use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub github_client_id: String,
    pub github_client_secret: String,
    pub session_secret: String,
    pub database_path: String,
    pub port: u16,
    pub frontend_url: String,
    pub backend_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        Ok(Config {
            github_client_id: env::var("GITHUB_CLIENT_ID")
                .expect("GITHUB_CLIENT_ID must be set"),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET")
                .expect("GITHUB_CLIENT_SECRET must be set"),
            session_secret: env::var("SESSION_SECRET")
                .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
            database_path: env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "./data/leetcode.duckdb".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            backend_url: env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        })
    }
}
