use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: String,
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();
        
        let port = env::var("PORT").unwrap_or_else(|_| {
            println!("PORT environment variable not set, using default port 24528");
            "24528".to_string()
        });
        
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable must be set"))?;
        
        Ok(AppConfig {
            port,
            database_url,
        })
    }
    
    pub fn server_address(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}
