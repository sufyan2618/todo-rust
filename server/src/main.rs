mod config;
mod error;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{
        HeaderValue, Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    routing::{get, patch, post},
};
use config::Config;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "todo_server=info,tower_http=info".into()),
        )
        .init();
    let config = Config::load().await?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_with(config.database_options.clone())
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let cors = CorsLayer::new()
        .allow_origin(config.client_origin.parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
    let port = config.port;
    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/api/auth/signup", post(routes::auth::signup))
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/me", get(routes::auth::me))
        .route(
            "/api/todos",
            get(routes::todos::list).post(routes::todos::create),
        )
        .route(
            "/api/todos/{id}",
            patch(routes::todos::update).delete(routes::todos::delete),
        )
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            pool: pool.clone(),
            config,
        });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(port, "Todo API listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    pool.close().await;
    Ok(())
}
async fn shutdown() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl-C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
