use std::{
    env,
    net::IpAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Deserialize;
use url::Url;

use crate::error::CoreError;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub bind_address: IpAddr,
    pub port: u16,
    pub static_dir: PathBuf,
    pub home_url: Url,
    pub chromium_binary: PathBuf,
    pub profile_dir: PathBuf,
    pub pid_file: PathBuf,
    pub token_store: PathBuf,
    pub pairing_code: String,
    pub system_backend: SystemBackendKind,
    pub input_backend: InputBackendKind,
    pub wtype_binary: PathBuf,
    pub xdotool_binary: PathBuf,
    pub ydotool_binary: PathBuf,
    pub unclutter_binary: Option<PathBuf>,
    #[serde(with = "duration_seconds")]
    pub command_timeout: Duration,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SystemBackendKind {
    Auto,
    Simulated,
    Linux,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputBackendKind {
    Auto,
    Wtype,
    Xdotool,
    Disabled,
}

impl Default for Config {
    fn default() -> Self {
        let data = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("suica-tv");
        Self {
            bind_address: "0.0.0.0".parse().expect("valid default IP"),
            port: 3030,
            static_dir: PathBuf::from("apps/tv/dist"),
            home_url: Url::parse("http://127.0.0.1:3030/").expect("valid default URL"),
            chromium_binary: PathBuf::from("/usr/bin/chromium"),
            profile_dir: data.join("chromium-profile"),
            pid_file: data.join("suica-tv.pid"),
            token_store: data.join("tokens.json"),
            pairing_code: String::new(),
            system_backend: SystemBackendKind::Auto,
            input_backend: InputBackendKind::Auto,
            wtype_binary: PathBuf::from("/usr/bin/wtype"),
            xdotool_binary: PathBuf::from("/usr/bin/xdotool"),
            ydotool_binary: PathBuf::from("/usr/bin/ydotool"),
            unclutter_binary: None,
            command_timeout: Duration::from_secs(10),
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self, CoreError> {
        let mut config = match path {
            Some(path) => toml::from_str(&std::fs::read_to_string(path)?)?,
            None => Self::default(),
        };
        if let Ok(value) = env::var("SUICA_CORE_BIND_ADDRESS") {
            config.bind_address = value
                .parse()
                .map_err(|_| CoreError::Config("SUICA_CORE_BIND_ADDRESS is invalid".into()))?;
        }
        if let Ok(value) = env::var("SUICA_CORE_PORT") {
            config.port = value
                .parse()
                .map_err(|_| CoreError::Config("SUICA_CORE_PORT is invalid".into()))?;
        }
        if let Ok(value) = env::var("SUICA_CORE_STATIC_DIR") {
            config.static_dir = value.into();
        }
        if let Ok(value) = env::var("SUICA_CORE_HOME_URL") {
            config.home_url = Url::parse(&value)
                .map_err(|_| CoreError::Config("SUICA_CORE_HOME_URL is invalid".into()))?;
        }
        if let Ok(value) = env::var("SUICA_CORE_CHROMIUM_BINARY") {
            config.chromium_binary = value.into();
        }
        if let Ok(value) = env::var("SUICA_CORE_PROFILE_DIR") {
            config.profile_dir = value.into();
        }
        if let Ok(value) = env::var("SUICA_CORE_PID_FILE") {
            config.pid_file = value.into();
        }
        if let Ok(value) = env::var("SUICA_CORE_TOKEN_STORE") {
            config.token_store = value.into();
        }
        if let Ok(value) = env::var("SUICA_CORE_PAIRING_CODE") {
            config.pairing_code = value;
        }
        if let Ok(value) = env::var("SUICA_CORE_SYSTEM_BACKEND") {
            config.system_backend = match value.as_str() {
                "auto" => SystemBackendKind::Auto,
                "simulated" => SystemBackendKind::Simulated,
                "linux" => SystemBackendKind::Linux,
                _ => {
                    return Err(CoreError::Config(
                        "SUICA_CORE_SYSTEM_BACKEND is invalid".into(),
                    ));
                }
            };
        }
        if let Ok(value) = env::var("SUICA_CORE_INPUT_BACKEND") {
            config.input_backend = match value.as_str() {
                "auto" => InputBackendKind::Auto,
                "wtype" => InputBackendKind::Wtype,
                "xdotool" => InputBackendKind::Xdotool,
                "disabled" => InputBackendKind::Disabled,
                _ => {
                    return Err(CoreError::Config(
                        "SUICA_CORE_INPUT_BACKEND is invalid".into(),
                    ));
                }
            };
        }
        if let Ok(value) = env::var("SUICA_CORE_WTYPE_BINARY") {
            config.wtype_binary = value.into();
        }
        if let Ok(value) = env::var("SUICA_CORE_XDOTOOL_BINARY") {
            config.xdotool_binary = value.into();
        }
        if let Ok(value) = env::var("SUICA_CORE_YDOTOOL_BINARY") {
            config.ydotool_binary = value.into();
        }
        if let Ok(value) = env::var("SUICA_CORE_UNCLUTTER_BINARY") {
            config.unclutter_binary = if value.is_empty() {
                None
            } else {
                Some(value.into())
            };
        }
        if let Ok(value) = env::var("SUICA_CORE_COMMAND_TIMEOUT") {
            config.command_timeout =
                Duration::from_secs(value.parse().map_err(|_| {
                    CoreError::Config("SUICA_CORE_COMMAND_TIMEOUT is invalid".into())
                })?);
        }
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        if self.pairing_code == "CHANGE_ME"
            || self.pairing_code.len() != 6
            || !self.pairing_code.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(CoreError::Config(
                "pairing_code must be a six digit value and must not be CHANGE_ME".into(),
            ));
        }
        if self.home_url.scheme() != "http" && self.home_url.scheme() != "https" {
            return Err(CoreError::Config("home_url must use http or https".into()));
        }
        Ok(())
    }
}

mod duration_seconds {
    use serde::{Deserialize, Deserializer};
    use std::time::Duration;
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        Ok(Duration::from_secs(u64::deserialize(d)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_default_pairing_code() {
        let c = Config {
            pairing_code: "CHANGE_ME".into(),
            ..Config::default()
        };
        assert!(c.validate().is_err());
    }
    #[test]
    fn accepts_six_digits() {
        let c = Config {
            pairing_code: "123456".into(),
            ..Config::default()
        };
        assert!(c.validate().is_ok());
    }
}
