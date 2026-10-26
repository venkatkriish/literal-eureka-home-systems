use std::{env, net::SocketAddr, str::FromStr};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read BIND_ADDR: {0}")]
    BindAddrEnv(#[source] env::VarError),
    #[error("invalid bind address '{value}': {source}")]
    InvalidBindAddr {
        value: String,
        #[source]
        source: std::net::AddrParseError,
    },
}

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_addr = match env::var("BIND_ADDR") {
            Ok(value) => parse_bind_addr(&value)?,
            Err(env::VarError::NotPresent) => parse_default_bind_addr()?,
            Err(err) => return Err(ConfigError::BindAddrEnv(err)),
        };
        Ok(Self { bind_addr })
    }
}

fn parse_bind_addr(value: &str) -> Result<SocketAddr, ConfigError> {
    SocketAddr::from_str(value).map_err(|source| ConfigError::InvalidBindAddr {
        value: value.to_string(),
        source,
    })
}

fn parse_default_bind_addr() -> Result<SocketAddr, ConfigError> {
    parse_bind_addr(DEFAULT_BIND_ADDR)
}
