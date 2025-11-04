use crate::config::Config;
use crate::db::DbConnection;
use crate::models::User;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: DbConnection,
    pub oauth_client: BasicClient,
}

impl AppState {
    pub fn new(config: Config, db: DbConnection) -> Self {
        let oauth_client = BasicClient::new(
            ClientId::new(config.github_client_id.clone()),
            Some(ClientSecret::new(config.github_client_secret.clone())),
            AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap()),
        )
        .set_redirect_uri(
            RedirectUrl::new(format!("{}/api/auth/callback", config.backend_url)).unwrap(),
        );

        Self {
            config,
            db,
            oauth_client,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
}

// Start OAuth flow
pub async fn github_login(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (auth_url, _csrf_token) = state
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    Redirect::temporary(auth_url.as_str())
}

// OAuth callback handler
pub async fn github_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuthCallbackQuery>,
) -> Result<Response, StatusCode> {
    // Exchange code for token
    let token_result = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get user info from GitHub
    let client = reqwest::Client::new();
    let user_info: GitHubUser = client
        .get("https://api.github.com/user")
        .header("User-Agent", "leetcode-tracker")
        .header(
            "Authorization",
            format!("Bearer {}", token_result.access_token().secret()),
        )
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create session
    let session_id = uuid::Uuid::new_v4().to_string();
    crate::db::create_session(
        &state.db,
        &session_id,
        user_info.id,
        &user_info.login,
        &user_info.avatar_url,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Redirect to frontend with session cookie
    let redirect_url = format!("{}?session={}", state.config.frontend_url, session_id);

    Ok((
        StatusCode::FOUND,
        [("Location", redirect_url.as_str())],
        format!("Redirecting to {}", redirect_url),
    )
        .into_response())
}

// Get current user
pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    session_id: Option<String>,
) -> Result<Json<User>, StatusCode> {
    let session_id = session_id.ok_or(StatusCode::UNAUTHORIZED)?;

    let session = crate::db::get_session(&state.db, &session_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(Json(User {
        github_id: session.github_id,
        username: session.username,
        avatar_url: session.avatar_url,
    }))
}

// Logout
pub async fn logout(
    State(state): State<Arc<AppState>>,
    session_id: Option<String>,
) -> Result<StatusCode, StatusCode> {
    if let Some(session_id) = session_id {
        crate::db::delete_session(&state.db, &session_id)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(StatusCode::OK)
}
