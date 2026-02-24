mod app;
mod cli;
mod mcp;

use crate::app::App;
use crate::cli::{Cli, Command, TransportType};
use crate::mcp::{BnfgenMCP, BnfgenSettings};
use anyhow::Result;
use clap::Parser;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{stdio, StreamableHttpServerConfig, StreamableHttpService};
use rmcp::ServiceExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Check {
            grammar,
            start,
            strict,
        } => {
            let grammar = std::fs::read_to_string(grammar)?;
            let app = App::new(grammar);
            let raw = app.parse()?;

            let mut pass = true;

            if strict {
                pass = app.strict_lint(
                    &raw,
                    start.expect("starting non-terminal is required when --strict is set"),
                );
            }

            let _checked = app.lint(raw)?;

            if !pass {
                return Err(app.diagnostics());
            }

            Ok(())
        }

        Command::Gen {
            grammar,
            start,
            count,
            seed,
            max_steps,
            max_attempts,
        } => {
            let grammar = std::fs::read_to_string(grammar)?;
            let app = App::new(grammar);

            // perform parsing and linting
            let raw = app.parse()?;
            let checked = app.lint(raw)?;

            // generate output
            let outputs = app.generate(checked, start, count, seed, max_steps, max_attempts)?;
            for output in outputs {
                println!("{}", output);
            }
            Ok(())
        }

        Command::Mcp {
            transport,
            port,
            host,
        } => match transport {
            TransportType::Stdio => {
                let service = BnfgenMCP::new(BnfgenSettings::builder().build())
                    .serve(stdio())
                    .await?;
                service.waiting().await?;
                Ok(())
            }
            TransportType::StreamableHttp => {
                tracing_subscriber::registry()
                    .with(
                        tracing_subscriber::EnvFilter::try_from_default_env()
                            .unwrap_or_else(|_| "debug".to_string().into()),
                    )
                    .with(tracing_subscriber::fmt::layer())
                    .init();

                let ct = tokio_util::sync::CancellationToken::new();
                let bind_address = format!(
                    "{}:{}",
                    host.expect("host is required when --transport is set to http"),
                    port.expect("port is required when --transport is set to http")
                );

                let service = StreamableHttpService::new(
                    || {
                        let service = BnfgenMCP::new(BnfgenSettings::builder().build());
                        Ok(service)
                    },
                    LocalSessionManager::default().into(),
                    StreamableHttpServerConfig {
                        cancellation_token: ct.child_token(),
                        stateful_mode: false,
                        ..Default::default()
                    },
                );

                let router = axum::Router::new().nest_service("/mcp", service);
                let tcp_listener = tokio::net::TcpListener::bind(bind_address).await?;

                tracing::info!("listening on {}", tcp_listener.local_addr()?);

                let _ = axum::serve(tcp_listener, router)
                    .with_graceful_shutdown(async move {
                        tokio::signal::ctrl_c().await.unwrap();
                        ct.cancel();
                    })
                    .await;
                Ok(())
            }
        },
    }
}
