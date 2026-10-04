use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use api::routes::{auth, v1};
use axum::{Router, http::Method};
use composition::AppRegistryImpl;
use infra::database::connect_database_with;
use infra::redis::RedisClient;
use shared::config::AppConfig;
use tokio::net::TcpListener;

use anyhow::Context;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use shared::env::{Environment, which};
use tower_http::LatencyUnit;
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[cfg(debug_assertions)]
use api::openapi::ApiDoc;
#[cfg(debug_assertions)]
use utoipa::OpenApi;
#[cfg(debug_assertions)]
use utoipa_redoc::{Redoc, Servable};
#[cfg(debug_assertions)]
use utoipa_swagger_ui::{Config, SwaggerUi};

// The API is called from the frontend dev server, which is a different origin.
fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_headers(cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(cors::Any)
}

#[tokio::main]
async fn main() -> Result<()> {
    let tracer_provider = build_tracer_provider()?;
    init_logger(&tracer_provider)?;
    bootstrap(tracer_provider).await
}

fn build_tracer_provider() -> Result<SdkTracerProvider> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")?;
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        // Cap one export so the flush (5 s) and `stop_grace_period` (15 s)
        // always cover it. The OTLP default is 10 s.
        .with_timeout(Duration::from_secs(3))
        .build()?;

    // Batches go out every 5 s (the SDK default).
    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("book-manager")
                .build(),
        )
        .build())
}

fn init_logger(tracer_provider: &SdkTracerProvider) -> Result<()> {
    let log_level = match which() {
        Environment::Development => "debug",
        Environment::Production => "info",
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());

    // JSON logs are for a log collector; they are unreadable in a terminal.
    let format: Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync> = match which()
    {
        Environment::Development => {
            Box::new(tracing_subscriber::fmt::layer().with_target(false))
        }
        Environment::Production => {
            Box::new(tracing_subscriber::fmt::layer().with_target(false).json())
        }
    };

    let tracer = tracer_provider.tracer("book-manager");

    tracing_subscriber::registry()
        .with(format)
        .with(env_filter)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;

    Ok(())
}

async fn bootstrap(tracer_provider: SdkTracerProvider) -> Result<()> {
    let app_config = AppConfig::new()?;
    let pool = connect_database_with(&app_config.database);
    let kv = Arc::new(RedisClient::new(&app_config.redis)?);

    let registry = Arc::new(AppRegistryImpl::new(pool, kv, app_config));

    let router = Router::new().merge(v1::routes()).merge(auth::routes());
    #[cfg(debug_assertions)]
    let router = router
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
                .config(Config::default().default_models_expand_depth(-1)),
        )
        .merge(Redoc::with_url("/docs", ApiDoc::openapi()));

    let app = router
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(cors())
        .with_state(registry);

    // 0.0.0.0 so the container is reachable from outside it.
    let addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 8080);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://{addr}");

    let result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Unexpected error happened in server")
        .inspect_err(|e| {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Unexpected error"
            )
        });

    // Drain is done; flush with a 5 s timeout.
    if let Err(e) = tracer_provider.shutdown() {
        tracing::error!(error = %e, "failed to shut down the tracer provider");
    }

    result
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install the ctrl-c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install the SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received ctrl-c"),
        _ = terminate => tracing::info!("received SIGTERM"),
    }
}
