use std::{collections::HashMap, path::PathBuf};

use executors::{
    executors::{BaseCodingAgent, StandardCodingAgentExecutor},
    mcp_config::read_agent_config,
    profile::ExecutorConfigs,
};
use rmcp::{
    handler::server::tool::{Parameters, ToolRouter},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars, tool, tool_handler, tool_router, ErrorData, ServerHandler,
};
use serde::{Deserialize, Serialize};
use services::services::{
    config::Config,
    filesystem::{DirectoryEntry, DirectoryListResponse, FilesystemService},
};

use crate::routes::config::Environment;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SystemInfo {
    #[schemars(description = "Operating system type (e.g., Linux, Windows, macOS)")]
    pub os_type: String,
    #[schemars(description = "Operating system version")]
    pub os_version: String,
    #[schemars(description = "System architecture (e.g., x86_64, aarch64)")]
    pub os_architecture: String,
    #[schemars(description = "System bitness (e.g., 32, 64)")]
    pub bitness: String,
    #[schemars(description = "Home directory path")]
    pub home_directory: PathBuf,
    #[schemars(description = "Current working directory")]
    pub current_directory: PathBuf,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetSystemInfoResponse {
    pub system: SystemInfo,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetConfigResponse {
    pub config: Config,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateConfigRequest {
    #[schemars(description = "Git branch prefix for task branches")]
    pub git_branch_prefix: Option<String>,
    #[schemars(description = "Default executor profile")]
    pub executor_profile: Option<String>,
    #[schemars(description = "Enable analytics")]
    pub analytics_enabled: Option<bool>,
    #[schemars(description = "Enable telemetry")]
    pub telemetry_enabled: Option<bool>,
    #[schemars(description = "Preferred editor")]
    pub editor: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct UpdateConfigResponse {
    pub config: Config,
    pub message: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListMcpServersRequest {
    #[schemars(description = "The executor to list MCP servers for (e.g., 'CLAUDE_CODE')")]
    pub executor: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct McpServerInfo {
    #[schemars(description = "Name of the MCP server")]
    pub name: String,
    #[schemars(description = "MCP server configuration (varies by executor)")]
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListMcpServersResponse {
    pub servers: Vec<McpServerInfo>,
    pub executor: String,
    pub config_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateMcpServersRequest {
    #[schemars(description = "The executor to update MCP servers for (e.g., 'CLAUDE_CODE')")]
    pub executor: String,
    #[schemars(description = "The MCP servers configuration as a JSON object")]
    pub servers: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct UpdateMcpServersResponse {
    pub message: String,
    pub servers_count: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListGitReposRequest {
    #[schemars(description = "Optional path to search for git repositories")]
    pub path: Option<String>,
    #[schemars(description = "Timeout in milliseconds (default: 5000)")]
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Maximum depth to search (default: 5)")]
    pub max_depth: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListGitReposResponse {
    pub repositories: Vec<DirectoryEntry>,
    pub count: usize,
    pub search_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListDirectoryRequest {
    #[schemars(description = "Path to list (defaults to home directory)")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListDirectoryResponseWrapper {
    pub entries: Vec<DirectoryEntry>,
    pub current_path: String,
    pub count: usize,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SystemServer {
    client: reqwest::Client,
    base_url: String,
    tool_router: ToolRouter<SystemServer>,
    filesystem_service: FilesystemService,
    start_time: std::time::Instant,
}

impl SystemServer {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            tool_router: Self::tool_router(),
            filesystem_service: FilesystemService::new(),
            start_time: std::time::Instant::now(),
        }
    }

    fn success<T: Serialize>(data: &T) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(data)
                .unwrap_or_else(|_| "Failed to serialize response".to_string()),
        )]))
    }

    fn err<S: Into<String>>(msg: S, details: Option<S>) -> Result<CallToolResult, ErrorData> {
        let mut v = serde_json::json!({"success": false, "error": msg.into()});
        if let Some(d) = details {
            v["details"] = serde_json::json!(d.into());
        }
        Ok(CallToolResult::error(vec![Content::text(
            serde_json::to_string_pretty(&v)
                .unwrap_or_else(|_| "Failed to serialize error".to_string()),
        )]))
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn get_config_from_api(&self) -> Result<Config, String> {
        let url = self.url("/api/config/info");
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to API: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API returned error status: {}", resp.status()));
        }

        #[derive(Deserialize)]
        struct ApiResponse {
            success: bool,
            data: Option<Config>,
        }

        let api_response: ApiResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        if !api_response.success {
            return Err("API returned error".to_string());
        }

        api_response.data.ok_or_else(|| "Missing data field".to_string())
    }

    async fn update_config_via_api(&self, updates: UpdateConfigRequest) -> Result<Config, String> {
        // First get current config
        let mut config = self.get_config_from_api().await?;

        // Apply updates
        if let Some(prefix) = updates.git_branch_prefix {
            config.git_branch_prefix = prefix;
        }
        if let Some(profile_str) = updates.executor_profile {
            // Parse executor profile string (format: "EXECUTOR" or "EXECUTOR:VARIANT")
            let parts: Vec<&str> = profile_str.split(':').collect();
            let executor_str = parts[0].trim().replace('-', "_").to_ascii_uppercase();
            let executor = executor_str.parse::<BaseCodingAgent>()
                .map_err(|_| format!("Invalid executor: {}", parts[0]))?;

            let variant = if parts.len() > 1 {
                Some(parts[1].trim().to_string())
            } else {
                None
            };

            config.executor_profile = executors::profile::ExecutorProfileId { executor, variant };
        }
        if let Some(analytics) = updates.analytics_enabled {
            config.analytics_enabled = Some(analytics);
        }
        // Note: telemetry_enabled doesn't exist in Config v7, only telemetry_acknowledged
        if let Some(telemetry) = updates.telemetry_enabled {
            config.telemetry_acknowledged = telemetry;
        }
        // Note: editor field is EditorConfig, not a String - for now, skip this field
        // TODO: Implement proper editor config parsing
        if updates.editor.is_some() {
            return Err("Updating editor config is not yet supported".to_string());
        }

        // Send update
        let url = self.url("/api/config/config");
        let resp = self
            .client
            .put(&url)
            .json(&config)
            .send()
            .await
            .map_err(|e| format!("Failed to update config: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API returned error status: {}", resp.status()));
        }

        #[derive(Deserialize)]
        struct ApiResponse {
            success: bool,
            data: Option<Config>,
        }

        let api_response: ApiResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        if !api_response.success {
            return Err("API returned error".to_string());
        }

        api_response.data.ok_or_else(|| "Missing data field".to_string())
    }
}

#[tool_router]
impl SystemServer {
    #[tool(description = "Get system information including OS details and key directories")]
    async fn get_system_info(&self) -> Result<CallToolResult, ErrorData> {
        let env = Environment::new();
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let system_info = SystemInfo {
            os_type: env.os_type,
            os_version: env.os_version,
            os_architecture: env.os_architecture,
            bitness: env.bitness,
            home_directory: home_dir,
            current_directory: current_dir,
        };

        Self::success(&GetSystemInfoResponse {
            system: system_info,
        })
    }

    #[tool(description = "Get the current Vibe Kanban configuration")]
    async fn get_config(&self) -> Result<CallToolResult, ErrorData> {
        match self.get_config_from_api().await {
            Ok(config) => Self::success(&GetConfigResponse { config }),
            Err(e) => Self::err(format!("Failed to get configuration: {}", e), None),
        }
    }

    #[tool(
        description = "Update Vibe Kanban configuration settings. Only provided fields will be updated."
    )]
    async fn update_config(
        &self,
        Parameters(request): Parameters<UpdateConfigRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.update_config_via_api(request).await {
            Ok(config) => Self::success(&UpdateConfigResponse {
                config,
                message: "Configuration updated successfully".to_string(),
            }),
            Err(e) => Self::err(format!("Failed to update configuration: {}", e), None),
        }
    }

    #[tool(description = "List configured MCP servers for a specific executor")]
    async fn list_mcp_servers(
        &self,
        Parameters(ListMcpServersRequest { executor }): Parameters<ListMcpServersRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        // Parse executor
        let executor_trimmed = executor.trim();
        let normalized_executor = executor_trimmed.replace('-', "_").to_ascii_uppercase();
        let base_executor = match normalized_executor.parse::<BaseCodingAgent>() {
            Ok(exec) => exec,
            Err(_) => {
                return Self::err(
                    format!("Unknown executor '{}'", executor_trimmed),
                    None,
                );
            }
        };

        // Get executor config
        let profiles = ExecutorConfigs::get_cached();
        let coding_agent = match profiles.get_coding_agent(&executors::profile::ExecutorProfileId::new(base_executor)) {
            Some(agent) => agent,
            None => {
                return Self::err("Executor not found", None);
            }
        };

        if !coding_agent.supports_mcp() {
            return Self::err("This executor does not support MCP servers", None);
        }

        // Get MCP config path
        let config_path = match coding_agent.default_mcp_config_path() {
            Some(path) => path.to_path_buf(),
            None => {
                return Self::err("Could not determine MCP config path", None);
            }
        };

        // Read MCP config
        let mcpc = coding_agent.get_mcp_config();
        let raw_config = match read_agent_config(&config_path, &mcpc).await {
            Ok(cfg) => cfg,
            Err(e) => {
                return Self::err(format!("Failed to read MCP config: {}", e), None);
            }
        };

        // Extract servers
        let mut current = &raw_config;
        for part in &mcpc.servers_path {
            current = match current.get(part) {
                Some(val) => val,
                None => {
                    return Self::success(&ListMcpServersResponse {
                        servers: vec![],
                        executor: executor_trimmed.to_string(),
                        config_path: config_path.to_string_lossy().to_string(),
                    });
                }
            };
        }

        let servers: Vec<McpServerInfo> = match current.as_object() {
            Some(servers_obj) => servers_obj
                .iter()
                .map(|(name, config)| McpServerInfo {
                    name: name.clone(),
                    config: config.clone(),
                })
                .collect(),
            None => vec![],
        };

        Self::success(&ListMcpServersResponse {
            servers,
            executor: executor_trimmed.to_string(),
            config_path: config_path.to_string_lossy().to_string(),
        })
    }

    #[tool(description = "Update MCP server configuration for a specific executor")]
    async fn update_mcp_servers(
        &self,
        Parameters(UpdateMcpServersRequest { executor, servers }): Parameters<
            UpdateMcpServersRequest,
        >,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!(
            "/api/config/mcp-config?executor={}",
            urlencoding::encode(&executor)
        ));

        #[derive(Serialize)]
        struct Payload {
            servers: HashMap<String, serde_json::Value>,
        }

        let resp = match self
            .client
            .post(&url)
            .json(&Payload { servers: servers.clone() })
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return Self::err(format!("Failed to send request: {}", e), None),
        };

        if !resp.status().is_success() {
            return Self::err(
                format!("API returned error status: {}", resp.status()),
                None,
            );
        }

        #[derive(Deserialize)]
        struct ApiResponse {
            success: bool,
            data: Option<String>,
        }

        let api_response: ApiResponse = match resp.json().await {
            Ok(r) => r,
            Err(e) => return Self::err(format!("Failed to parse response: {}", e), None),
        };

        if !api_response.success {
            return Self::err("API returned error", None);
        }

        Self::success(&UpdateMcpServersResponse {
            message: api_response
                .data
                .unwrap_or_else(|| "MCP servers updated".to_string()),
            servers_count: servers.len(),
        })
    }

    #[tool(
        description = "List git repositories on the system. Searches common directories by default."
    )]
    async fn list_git_repos(
        &self,
        Parameters(ListGitReposRequest {
            path,
            timeout_ms,
            max_depth,
        }): Parameters<ListGitReposRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let timeout = timeout_ms.unwrap_or(5000);
        let hard_timeout = timeout + 2000;
        let depth = max_depth.unwrap_or(5);

        let repositories = match self
            .filesystem_service
            .list_git_repos(path.clone(), timeout, hard_timeout, Some(depth))
            .await
        {
            Ok(repos) => repos,
            Err(e) => {
                return Self::err(format!("Failed to list git repositories: {}", e), None);
            }
        };

        let search_path = path.unwrap_or_else(|| "home directory".to_string());

        Self::success(&ListGitReposResponse {
            count: repositories.len(),
            repositories,
            search_path,
        })
    }

    #[tool(description = "List files and directories in a path")]
    async fn list_directory(
        &self,
        Parameters(ListDirectoryRequest { path }): Parameters<ListDirectoryRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let response: DirectoryListResponse = match self.filesystem_service.list_directory(path).await {
            Ok(resp) => resp,
            Err(e) => {
                return Self::err(format!("Failed to list directory: {}", e), None);
            }
        };

        Self::success(&ListDirectoryResponseWrapper {
            count: response.entries.len(),
            entries: response.entries,
            current_path: response.current_path,
        })
    }

    #[tool(description = "Check if Vibe Kanban is healthy and get version information")]
    async fn health_check(&self) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/health");

        // Try to reach the health endpoint
        let is_healthy = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        let status = if is_healthy { "healthy" } else { "unhealthy" };
        let uptime = self.start_time.elapsed().as_secs();

        Self::success(&HealthCheckResponse {
            status: status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: Some(uptime),
        })
    }
}

#[tool_handler]
impl ServerHandler for SystemServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "vibe-kanban-system".to_string(),
                version: "1.0.0".to_string(),
            },
            instructions: Some(
                "System configuration and discovery tools for Vibe Kanban. \
                TOOLS: 'get_system_info', 'get_config', 'update_config', 'list_mcp_servers', \
                'update_mcp_servers', 'list_git_repos', 'list_directory', 'health_check'. \
                Use these tools to inspect system state, manage configuration, discover resources, \
                and monitor health."
                    .to_string(),
            ),
        }
    }
}
