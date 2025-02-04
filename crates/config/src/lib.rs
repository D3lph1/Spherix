use std::env;
use std::env::VarError;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Error as IOError, Write};

use config::{Config as ConfigLib, ConfigError as ConfigErrorLib, File as ConfigFile, ValueKind};
use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer, Serialize};

use spherix_macro::config;

use crate::r#macro::config_enum_case_ignore;

mod r#macro;

const CONFIG_PATHNAME_ENV: &str = "CONFIG_PATHNAME";
const CONFIG_PATHNAME_DEFAULT: &str = "config.yaml";

const FILE_PREAMBLE: &str = r"#
# Primary Spherix configuration file
# Read about parameters at: https://github.com/D3lph1/Spherix
#

";

config!(
    struct Config {
        network: struct Network {
            port: usize 25565,
            compression: struct Compression {
                enabled: bool true,
                threshold: usize 256
            }
        },
        auth: struct Auth {
            enabled: bool false,
            session_host: PathBuf PathBuf::from("http://127.0.0.1:25585/authlib-injector/sessionserver")
        },
        world: struct World {
            seed: i32 1,
            strategy: WorldStrategy WorldStrategy::GENERATE,
            path: PathBuf PathBuf::from("./world"),
            view_distance: u8 8
        },
        chat: struct Chat {
            secure: bool true
        },
        log: struct Log {
            terminal: struct LogTerminal {
                ansi: bool true,
                target: bool false,
                level: LogLevel LogLevel::TRACE
            },
            file: struct LogFile {
                enabled: bool true,
                target: bool false,
                level: LogLevel LogLevel::TRACE,
                path: PathBuf PathBuf::from("./log"),
                file_prefix: String "spherix"
            }
        }
    }
);

config_enum_case_ignore! (
    pub enum LogLevel {
        TRACE,
        DEBUG,
        INFO,
        WARN,
        ERROR
    }
);

config_enum_case_ignore!(
    pub enum WorldStrategy {
        GENERATE,
        LOAD
    }
);

pub enum ConfigResult {
    Presented(Config),
    Created(Config),
}

impl ConfigResult {
    pub fn unwrap(self) -> Config {
        match self {
            Self::Presented(cfg) => cfg,
            Self::Created(cfg) => cfg
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IO(IOError),
    Config(ConfigErrorLib),
    Yaml(serde_yaml::Error),
    Env(VarError),
    Enum(EnumError)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(e) => Display::fmt(e, f),
            Error::Config(e) => Display::fmt(e, f),
            Error::Yaml(e) => Display::fmt(e, f),
            Error::Env(e) => Display::fmt(e, f),
            Error::Enum(e) => Display::fmt(e, f),
        }
    }
}

#[derive(Debug)]
pub struct EnumError {
    ty: String,
    expected: Vec<String>,
    given: String
}

impl EnumError {
    pub fn new(ty: String, expected: Vec<String>, given: String) -> Self {
        Self {
            ty,
            expected,
            given
        }
    }
}

impl Display for EnumError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Expected one of enum {} values {} (case insensitive), but \"{}\" given",
            self.ty,
            self
                .expected
                .iter()
                .map(|v| format!("\"{}\"", v))
                .collect::<Vec<String>>()
                .join(", "),
            self.given
        )
    }
}

pub fn build_config(config_pathname: std::path::PathBuf) -> Result<ConfigResult, Error> {
    let presented = config_pathname.exists();
    if !presented {
        File::create(config_pathname.clone()).unwrap();
    }

    let builder = ConfigLib::builder()
        .add_source(ConfigFile::from(config_pathname.clone()));
    let builder = Config::defaults(builder);

    let config = builder
        .build()
        .unwrap();

    let config_entity: Config = config
        .try_deserialize()
        .map_err(|e| Error::Config(e))?;

    if !presented {
        let mut file = File::create(config_pathname.clone()).unwrap();
        file.write(FILE_PREAMBLE.as_bytes()).map_err(|e| Error::IO(e))?;

        serde_yaml::to_writer(
            file,
            &config_entity
        ).map_err(|e| Error::Yaml(e))?;
    }

    if presented {
        Ok(ConfigResult::Presented(config_entity))
    } else {
        Ok(ConfigResult::Created(config_entity))
    }
}

pub fn build_config_from_env() -> Result<ConfigResult, Error> {
    match env::var(CONFIG_PATHNAME_ENV) {
        Ok(val) => build_config(std::path::PathBuf::from(val)),
        Err(e) => match e {
            VarError::NotPresent => build_config(std::path::PathBuf::from(CONFIG_PATHNAME_DEFAULT.to_owned())),
            VarError::NotUnicode(e) => Err(Error::Env(VarError::NotUnicode(e)))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathBuf(pub std::path::PathBuf);

impl PathBuf {
    pub fn inner(&self) -> std::path::PathBuf {
        self.clone().into()
    }

    pub fn into_inner(self) -> std::path::PathBuf {
        self.into()
    }
}

impl From<PathBuf> for std::path::PathBuf {
    #[inline]
    fn from(value: PathBuf) -> Self {
        value.0
    }
}

impl From<std::path::PathBuf> for PathBuf {
    #[inline]
    fn from(value: std::path::PathBuf) -> Self {
        Self(value)
    }
}

impl From<&str> for PathBuf {
    fn from(value: &str) -> Self {
        PathBuf(std::path::PathBuf::from(value))
    }
}

impl From<PathBuf> for ValueKind {
    fn from(value: PathBuf) -> Self {
        ValueKind::String(value.0.to_str().unwrap().to_string())
    }
}
