use std::env;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ConfigError {
    message: String
}
impl ConfigError {
    pub(crate) fn new(message: String) -> Self {ConfigError{message}}
}
impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "configuration error: {}", self.message)
    }
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) branding: String,
    pub(crate) s3_region: String,
    pub(crate) s3_endpoint_url: String,
    pub(crate) s3_bucket: String
}

impl Default for Config {
    fn default() -> Self {
        panic!("No default config")
    }
}

impl Config {
    pub(crate) fn load() -> Result<Config, ConfigError> {
        let mut config_path = env::current_dir()
            .map_err(|e| ConfigError::new(format!("failed to get current directory: {}", e)))?;
        config_path.push("Config.toml");
        info!("Loading config from path {}", &config_path.display());
        let config: Self = confy::load_path(config_path).map_err(|e| ConfigError::new(e.to_string()))?;

        Ok(config)
    }

}

#[derive(Deserialize)]
pub(crate) struct AppEnv {
    pub(crate) database_url: String,
    pub(crate) access_token_key: String,
    pub(crate) refresh_token_key: String,
    pub(crate) smtp_username: String,
    pub(crate) smtp_password: String,
    pub(crate) _cors_allowed: String,
    pub(crate) _rocket_secret_key: String,
    pub(crate) _static_path: String,
}

impl AppEnv {
    pub(crate) fn new_from_env() -> Result<AppEnv, String> {
        Ok(Self {
            database_url: field_from_env("DATABASE_URL")?,
            access_token_key: field_from_env("ACCESS_TOKEN_KEY")?,
            refresh_token_key: field_from_env("REFRESH_TOKEN_KEY")?,
            smtp_username: field_from_env("SMTP_USERNAME")?,
            smtp_password: field_from_env("SMTP_PASSWORD")?,
            _cors_allowed: field_from_env("CORS_ALLOWED")?,
            _rocket_secret_key: field_from_env("ROCKET_SECRET_KEY")?,
            _static_path: field_from_env("STATIC_PATH")?,
        })
    }
}

fn field_from_env(field_name: &str) -> Result<String, String> {
    env::var(field_name).map_err(|e| format!("{}: {}", e, field_name))
}

#[cfg(test)]
impl Default for AppEnv {
    fn default() -> Self {
        Self { database_url: Default::default(), access_token_key: Default::default(), refresh_token_key: Default::default(), smtp_username: Default::default(), smtp_password: Default::default(), _cors_allowed: Default::default(), _rocket_secret_key: Default::default(), _static_path: Default::default() }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn test_load() {
        let config = Config::load().unwrap();
        assert_eq!("Pokébargains", config.branding);
    }

}