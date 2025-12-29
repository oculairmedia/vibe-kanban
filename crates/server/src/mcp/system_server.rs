use std::{collections::HashMap, path::PathBuf, sync::Arc};

use turbomcp::prelude::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use services::services::{
    config::Config,
    filesystem::{DirectoryEntry, DirectoryListResponse, FilesystemService},
};

use crate::routes::config::Environment;

// Valid executor names (from executors::executors::BaseCodingAgent enum)
// Avoiding dependency on executors crate which has codex-protocol compilation issues
const VALID_EXECUTORS: &[&str] = &[
    "CLAUDE_CODE",
    "AMP",
    "GEMINI",
    "CODEX",
    "OPENCODE",
    "CURSOR",
    "QWEN_CODE",
    "COPILOT",
];

fn validate_executor(executor: &str) -> Result<(), String> {
    if VALID_EXECUTORS.contains(&executor) {
        Ok(())
    } else {
        Err(format!(
            "Unknown executor '{}'. Valid executors are: {}",
            executor,
            VALID_EXECUTORS.join(", ")
        ))
    }
}

// Minimal copy of ExecutorProfileId to avoid depending on executors crate
// which has codex-protocol compilation issues
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct McpExecutorProfileId {
    executor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<String>,
}

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

// Simplified version for MCP deserialization (uses String instead of enum)
#[derive(Debug, Serialize, Deserialize)]
struct ExecutorProfileInfoSimple {
    executor: String,
    variant: String,
    available: bool,
    capabilities: Vec<String>,
    supports_mcp: bool,
    config_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExecutorProfilesResponseSimple {
    profiles: Vec<ExecutorProfileInfoSimple>,
    count: usize,
}

// Request for getting a single executor profile
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetExecutorProfileRequest {
    #[schemars(description = "The executor type (e.g., 'CLAUDE_CODE', 'AMP', 'CODEX'). Case-insensitive.")]
    executor: String,
    #[schemars(description = "Optional variant name (e.g., 'ROUTER', 'PLAN'). Defaults to 'DEFAULT' if not specified.")]
    variant: Option<String>,
}

// Response for a single executor profile with full details
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct ExecutorProfileDetailSimple {
    executor: String,
    variant: String,
    available: bool,
    capabilities: Vec<String>,
    supports_mcp: bool,
    config_path: Option<String>,
    profile_id: String,
    display_name: String,
}

#[derive(Debug, Clone)]
pub struct SystemServer {
    client: Arc<reqwest::Client>,
    base_url: Arc<String>,
    filesystem_service: Arc<FilesystemService>,
    start_time: std::time::Instant,
}

impl SystemServer {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Arc::new(reqwest::Client::new()),
            base_url: Arc::new(base_url.to_string()),
            filesystem_service: Arc::new(FilesystemService::new()),
            start_time: std::time::Instant::now(),
        }
    }

    fn err_str(msg: &str, details: Option<&str>) -> McpError {
        let mut error_msg = msg.to_string();
        if let Some(d) = details {
            error_msg.push_str(&format!(": {}", d));
        }
        McpError::internal(error_msg)
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        rb: reqwest::RequestBuilder,
    ) -> Result<T, McpError> {
        let resp = rb
            .send()
            .await
            .map_err(|e| Self::err_str("Failed to connect to VK API", Some(&e.to_string())))?;

        if !resp.status().is_success() {
            let status = resp.status();
            return Err(Self::err_str(
                &format!("VK API returned error status: {}", status),
                None,
            ));
        }

        #[derive(Deserialize)]
        struct ApiResponseEnvelope<T> {
            success: bool,
            data: Option<T>,
            message: Option<String>,
        }

        let api_response = resp
            .json::<ApiResponseEnvelope<T>>()
            .await
            .map_err(|e| Self::err_str("Failed to parse VK API response", Some(&e.to_string())))?;

        if !api_response.success {
            let msg = api_response.message.as_deref().unwrap_or("Unknown error");
            return Err(Self::err_str("VK API returned error", Some(msg)));
        }

        api_response
            .data
            .ok_or_else(|| Self::err_str("VK API response missing data field", None))
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn get_config_from_api(&self) -> Result<Config, McpError> {
        let url = self.url("/api/info");
        
        // Deserialize only the fields we need from UserSystemInfo
        #[derive(Deserialize)]
        struct UserSystemInfoPartial {
            config: Config,
        }
        
        let user_info: UserSystemInfoPartial = self.send_json(self.client.get(&url)).await?;
        Ok(user_info.config)
    }

    async fn update_config_via_api(&self, updates: UpdateConfigRequest) -> Result<Config, McpError> {
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
            let variant = if parts.len() > 1 {
                let v = parts[1].trim();
                if v.is_empty() { None } else { Some(v.to_string()) }
            } else {
                None
            };

            // Validate executor name against known executors
            if let Err(err_msg) = validate_executor(&executor_str) {
                return Err(McpError::invalid_request(err_msg));
            }

            // Create a local ExecutorProfileId that matches the backend's JSON format
            let mcp_profile = McpExecutorProfileId {
                executor: executor_str,
                variant,
            };

            // Convert to JSON Value and then deserialize as the config's ExecutorProfileId type
            // This works because both types serialize to the same JSON structure
            let profile_json = serde_json::to_value(&mcp_profile)
                .map_err(|e| McpError::internal(format!("Failed to serialize executor profile: {}", e)))?;
            config.executor_profile = serde_json::from_value(profile_json)
                .map_err(|e| McpError::internal(format!("Failed to convert executor profile: {}", e)))?;
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
            return Err(McpError::invalid_request("Updating editor config is not yet supported"));
        }

        // Send update
        let url = self.url("/api/config");
        self.send_json(self.client.put(&url).json(&config)).await
    }
}

#[turbomcp::server(
    name = "vibe-kanban-system",
    version = "1.0.0",
    description = "System configuration and discovery tools for Vibe Kanban. TOOLS: 'get_system_info', 'get_config', 'update_config', 'list_mcp_servers', 'update_mcp_servers', 'list_executor_profiles', 'get_executor_profile', 'list_git_repos', 'list_directory', 'health_check'. Use these tools to inspect system state, manage configuration, discover resources, and monitor health."
)]
impl SystemServer {
    #[tool(description = "Get system information including OS details and key directories")]
    async fn get_system_info(&self) -> McpResult<String> {
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

        let response = GetSystemInfoResponse {
            system: system_info,
        };
        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(description = "Get the current Vibe Kanban configuration")]
    async fn get_config(&self) -> McpResult<String> {
        let config = self.get_config_from_api().await?;
        let response = GetConfigResponse { config };
        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "Update Vibe Kanban configuration settings. Only provided fields will be updated."
    )]
    async fn update_config(&self, request: UpdateConfigRequest) -> McpResult<String> {
        let config = self.update_config_via_api(request).await?;
        let response = UpdateConfigResponse {
            config,
            message: "Configuration updated successfully".to_string(),
        };
        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(description = "List configured MCP servers for a specific executor")]
    async fn list_mcp_servers(&self, request: ListMcpServersRequest) -> McpResult<String> {
        // Parse executor
        let executor_trimmed = request.executor.trim();
        let normalized_executor = executor_trimmed.replace('-', "_").to_ascii_uppercase();

        // Validate executor name
        if let Err(err_msg) = validate_executor(&normalized_executor) {
            return Err(McpError::invalid_request(err_msg));
        }

        // NOTE: This functionality requires access to executors crate which has
        // codex-protocol compilation issues. The MCP server reading logic depends on
        // ExecutorConfigs::get_cached() and read_agent_config() which are not available.
        // Users should access MCP configs directly via the filesystem or use the web UI.
        Err(McpError::internal(format!(
            "Listing MCP servers via system server is temporarily disabled due to build issues. \
             Please access MCP config files directly from the filesystem, or use the Vibe Kanban web UI. \
             Requested executor: {}. Config files are typically in ~/.config/{}/",
            executor_trimmed,
            executor_trimmed.to_lowercase()
        )))
    }

    #[tool(description = "Update MCP server configuration for a specific executor")]
    async fn update_mcp_servers(&self, request: UpdateMcpServersRequest) -> McpResult<String> {
        let url = self.url(&format!(
            "/api/config/mcp-config?executor={}",
            urlencoding::encode(&request.executor)
        ));

        #[derive(Serialize)]
        struct Payload {
            servers: HashMap<String, serde_json::Value>,
        }

        let _: serde_json::Value = self
            .send_json(self.client.post(&url).json(&Payload { servers: request.servers.clone() }))
            .await?;

        let response = UpdateMcpServersResponse {
            message: "MCP servers updated successfully".to_string(),
            servers_count: request.servers.len(),
        };
        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(
        description = "List git repositories on the system. Searches common directories by default."
    )]
    async fn list_git_repos(&self, request: ListGitReposRequest) -> McpResult<String> {
        let timeout = request.timeout_ms.unwrap_or(5000);
        let hard_timeout = timeout + 2000;
        let depth = request.max_depth.unwrap_or(5);

        // Default to /opt/stacks for better developer experience
        let default_path = "/opt/stacks".to_string();
        let path = request.path.clone().or(Some(default_path.clone()));

        let repositories = self
            .filesystem_service
            .list_git_repos(path.clone(), timeout, hard_timeout, Some(depth))
            .await
            .map_err(|e| McpError::internal(format!("Failed to list git repositories: {}", e)))?;

        let search_path = request.path.unwrap_or(default_path);

        let response = ListGitReposResponse {
            count: repositories.len(),
            repositories,
            search_path,
        };
        Ok(serde_json::to_string_pretty(&response).unwrap())
    }

    #[tool(description = "List files and directories in a path")]
    async fn list_directory(&self, request: ListDirectoryRequest) -> McpResult<String> {
        let response: DirectoryListResponse = self
            .filesystem_service
            .list_directory(request.path)
            .await
            .map_err(|e| McpError::internal(format!("Failed to list directory: {}", e)))?;

        let wrapper = ListDirectoryResponseWrapper {
            count: response.entries.len(),
            entries: response.entries,
            current_path: response.current_path,
        };
        Ok(serde_json::to_string_pretty(&wrapper).unwrap())
    }

    #[tool(description = "List all available executor profiles with their capabilities and availability status")]
    async fn list_executor_profiles(&self) -> McpResult<String> {
        let url = self.url("/api/executor-profiles");
        let data: ExecutorProfilesResponseSimple =
            self.send_json(self.client.get(&url)).await?;
        Ok(serde_json::to_string_pretty(&data).unwrap())
    }

    #[tool(description = "Get detailed information about a specific executor profile including availability, capabilities, and configuration. Use this to check if a specific coding agent is available before starting a task.")]
    async fn get_executor_profile(&self, request: GetExecutorProfileRequest) -> McpResult<String> {
        // Build URL based on whether variant is specified
        let url = match &request.variant {
            Some(variant) if variant != "DEFAULT" => {
                self.url(&format!("/api/executor-profiles/{}/{}", request.executor, variant))
            }
            _ => self.url(&format!("/api/executor-profiles/{}", request.executor)),
        };

        let data: ExecutorProfileDetailSimple =
            self.send_json(self.client.get(&url)).await?;
        Ok(serde_json::to_string_pretty(&data).unwrap())
    }

    #[tool(description = "Check if Vibe Kanban is healthy and get version information")]
    async fn health_check(&self) -> McpResult<String> {
        let url = self.url("/api/health");

        // Try to reach the health endpoint
        let is_healthy = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        let status = if is_healthy { "healthy" } else { "unhealthy" };
        let uptime = self.start_time.elapsed().as_secs();

        let response = HealthCheckResponse {
            status: status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: Some(uptime),
        };
        Ok(serde_json::to_string_pretty(&response).unwrap())
    }
}

// Custom HTTP runner implementation with permissive security for development
#[cfg(feature = "http")]
impl SystemServer {
    /// Run HTTP server with custom security configuration
    pub async fn run_http_custom(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        use turbomcp_transport::streamable_http::{StreamableHttpConfigBuilder};
        use std::time::Duration;

        // Create permissive HTTP config for development
        let config = StreamableHttpConfigBuilder::new()
            .with_bind_address(addr)
            .allow_any_origin(true) // Allow any origin in development mode
            .allow_localhost(true)
            .with_rate_limit(1_000_000, Duration::from_secs(60)) // Very high limit for development
            .build();

        // Run the HTTP server with custom config (v2.3 API uses method on server)
        self.run_http_with_config(addr, config).await?;
        Ok(())
    }
}
