mod api;
mod auth;
mod config;
mod db;
mod models;

use auth::AppState;
use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Middleware to extract session from cookie or query parameter
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try to get session from cookie or query parameter
    let session_id = request
        .headers()
        .get("Cookie")
        .and_then(|cookie| cookie.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find(|c| c.trim().starts_with("session="))
                .and_then(|c| c.split('=').nth(1))
        })
        .or_else(|| {
            request
                .uri()
                .query()
                .and_then(|q| {
                    q.split('&')
                        .find(|p| p.starts_with("session="))
                        .and_then(|p| p.split('=').nth(1))
                })
        })
        .map(|s| s.to_string());

    // Verify session exists in database (for protected routes)
    if let Some(ref session_id) = session_id {
        if db::get_session(&state.db, session_id)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .is_none()
        {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // Store session_id in extensions for handlers to use
    request.extensions_mut().insert(session_id);

    Ok(next.run(request).await)
}

// Extract session from request extensions
async fn extract_session(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let session_id = request.extensions().get::<Option<String>>().cloned().flatten();

    // Add session_id to request for handlers
    request.extensions_mut().insert(session_id);

    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "leetcode_tracker_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::from_env()?;
    let port = config.port;

    // Initialize database
    let db = db::init_database(&config.database_path)?;

    // Create app state
    let state = Arc::new(AppState::new(config, db));

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);

    // Protected API routes (require authentication)
    let protected_api = Router::new()
        .route("/lists", get(api::get_lists))
        .route("/lists/:name", get(api::get_list_questions))
        .route("/intersections", get(api::get_intersections))
        .route("/intersections/:id", get(api::get_intersection_questions))
        .route("/questions/:number", put(api::update_question))
        .route("/tags", get(api::get_tags).post(api::create_tag))
        .route("/tags/:name", delete(api::delete_tag))
        .route("/questions/:number/tags", get(api::get_question_tags))
        .route("/questions/:number/tags", put(api::update_question_tags))
        .route("/metrics/:list", get(api::get_metrics))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Auth routes (public)
    let auth_routes = Router::new()
        .route("/github", get(auth::github_login))
        .route("/callback", get(auth::github_callback))
        .route(
            "/me",
            get(|State(state): State<Arc<AppState>>, request: Request| async move {
                let session_id = request.extensions().get::<Option<String>>().cloned().flatten();
                auth::get_current_user(State(state), session_id).await
            }),
        )
        .route(
            "/logout",
            post(|State(state): State<Arc<AppState>>, request: Request| async move {
                let session_id = request.extensions().get::<Option<String>>().cloned().flatten();
                auth::logout(State(state), session_id).await
            }),
        );

    // Combine all routes
    let api_routes = Router::new()
        .nest("/auth", auth_routes)
        .merge(protected_api)
        .with_state(state.clone());

    // Serve static files (frontend) at root, API at /api
    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(
            ServeDir::new("/app/static")
                .not_found_service(ServeFile::new("/app/static/index.html"))
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await?;

    tracing::info!("Server listening on port {}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
