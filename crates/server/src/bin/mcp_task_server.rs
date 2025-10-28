//! Vibe Kanban MCP Server - Main Entry Point
//!
//! This binary starts the Vibe Kanban MCP server with the selected transport protocol.

use server::mcp::task_server::TaskServer;
use std::env;
use tracing_subscriber::{EnvFilter, prelude::*};
use utils::{
    port_file::read_port_file,
    sentry::{self as sentry_utils, SentrySource, sentry_layer},
};

// No additional imports needed for HTTP feature
// The TaskServer already has the run_http_custom method

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    sentry_utils::init_once(SentrySource::Mcp);

    // Initialize tracing/logging
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(EnvFilter::new(&log_level)),
        )
        .with(sentry_layer())
        .init();

    let version = env!("CARGO_PKG_VERSION");
    tracing::info!("╔══════════════════════════════════════╗");
    tracing::info!("║  Vibe Kanban MCP Server (TurboMCP)  ║");
    tracing::info!("╚══════════════════════════════════════╝");
    tracing::info!("Version: {}", version);

    // Get configuration from environment
    let transport = env::var("TRANSPORT").unwrap_or_else(|_| "stdio".to_string());

    // Read backend URL from environment variable or construct from port
    let base_url = if let Ok(url) = std::env::var("VIBE_BACKEND_URL") {
        tracing::info!("Using backend URL from VIBE_BACKEND_URL: {}", url);
        url
    } else {
        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        // Get port from environment variables or fall back to port file
        let port = match std::env::var("BACKEND_PORT").or_else(|_| std::env::var("PORT")) {
            Ok(port_str) => {
                tracing::info!("Using port from environment: {}", port_str);
                port_str.parse::<u16>().map_err(|e| {
                    anyhow::anyhow!("Invalid port value '{}': {}", port_str, e)
                })?
            }
            Err(_) => {
                let port = read_port_file("vibe-kanban").await?;
                tracing::info!("Using port from port file: {}", port);
                port
            }
        };

        let url = format!("http://{}:{}", host, port);
        tracing::info!("Using backend URL: {}", url);
        url
    };

    tracing::info!("Transport: {}", transport);
    tracing::info!("Backend API: {}", base_url);

    // Create server instance
    let server = TaskServer::new(&base_url);

    // Run with selected transport
    match transport.to_lowercase().as_str() {
        "http" => {
            #[cfg(feature = "http")]
            {
                let port: u16 = env::var("MCP_PORT")
                    .unwrap_or_else(|_| "3456".to_string())
                    .parse()
                    .expect("MCP_PORT must be a valid number");

                let addr = format!("0.0.0.0:{}", port);
                tracing::info!("🚀 Starting HTTP transport");
                tracing::info!("📡 Listening on: http://{}", addr);
                tracing::info!("🔗 Endpoint: http://{}/mcp", addr);
                tracing::info!("⚠️  CORS: Allowing all origins (development mode)");
                tracing::info!("Ready for MCP client connections");

                // Use custom HTTP runner with permissive security for development
                server.run_http_custom(&addr).await?;
            }
            #[cfg(not(feature = "http"))]
            {
                tracing::error!("HTTP transport requested but not compiled with 'http' feature");
                return Err("HTTP transport not available".into());
            }
        }
        "stdio" | _ => {
            tracing::info!("🚀 Starting stdio transport");
            tracing::info!("Ready for MCP client connections");

            server.run_stdio().await?;
        }
    }

    tracing::info!("Server shutdown complete");
    Ok(())
}
