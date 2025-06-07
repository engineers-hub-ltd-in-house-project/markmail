use std::env;

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub api_port: u16,
    pub environment: Environment,
}

pub enum Environment {
    Development,
    Production,
    Test,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        // 環境変数の読み込み
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://markmail:markmail_password@localhost:5432/markmail_dev".to_string()
        });

        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your_jwt_secret_key_here_please_change_in_production".to_string());

        let jwt_expiration = env::var("JWT_EXPIRATION")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .unwrap_or(3600);

        let api_port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        let environment = match env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .as_str()
        {
            "production" => Environment::Production,
            "test" => Environment::Test,
            _ => Environment::Development,
        };

        Self {
            database_url,
            redis_url,
            jwt_secret,
            jwt_expiration,
            api_port,
            environment,
        }
    }

    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }

    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    pub fn is_test(&self) -> bool {
        matches!(self.environment, Environment::Test)
    }
}
