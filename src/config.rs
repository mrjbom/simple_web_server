use std::{net, path};
use thiserror::Error;

/// Server config
#[derive(Debug)]
pub struct Config<'a> {
    socket_addr_v4: net::SocketAddrV4,
    root_folder_path: &'a path::Path,
    threads_number: u8,
}

impl<'a> Config<'a> {
    pub fn build_from_args(args: &'a crate::Args) -> Result<Self, ConfigError> {
        let socket_addr_v4 = args.socket_addr_v4.parse::<net::SocketAddrV4>()?;
        let root_folder_path = path::Path::new(args.root_folder_path.as_str());
        if !root_folder_path.is_dir() {
            // The path does not exist or does not point to the directory or cannot be accessed.
            return Err(ConfigError::WrongRootFolderPath);
        }
        let threads_number = args.threads_number;
        if threads_number == 0 {
            return Err(ConfigError::ZeroThreadsNumber)
        }

        Ok(Config {
            socket_addr_v4,
            root_folder_path,
            threads_number
        })
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Wrong address.")]
    WrongAddr(#[from] net::AddrParseError),
    #[error("Wrong root folder path.")]
    WrongRootFolderPath,
    #[error("Zero threads number.")]
    ZeroThreadsNumber,
}

#[cfg(test)]
mod tests {
    use crate::Args;
    use super::*;
    #[test]
    fn build_config_from_args_wrong_addr() {
        let args = Args {
            socket_addr_v4: "Wrong".to_string(),
            root_folder_path: "./".to_string(),
            threads_number: 4,
        };
        let config = Config::build_from_args(&args);
        assert!(matches!(config, Err(ConfigError::WrongAddr(_))));
    }

    #[test]
    fn build_config_from_args_wrong_path() {
        let args = Args {
            socket_addr_v4: "127.0.0.1:7878".to_string(),
            root_folder_path: "".to_string(),
            threads_number: 4,
        };
        let config = Config::build_from_args(&args);
        assert!(matches!(config, Err(ConfigError::WrongRootFolderPath)));
    }

    #[test]
    fn build_config_from_args_zero_threads_number() {
        let args = Args {
            socket_addr_v4: "127.0.0.1:7878".to_string(),
            root_folder_path: "./".to_string(),
            threads_number: 0,
        };
        let config = Config::build_from_args(&args);
        assert!(matches!(config, Err(ConfigError::ZeroThreadsNumber)));
    }

    #[test]
    fn build_config_from_args() {
        let args = Args {
            socket_addr_v4: "127.0.0.1:7878".to_string(),
            root_folder_path: "./".to_string(),
            threads_number: 4,
        };
        let config = Config::build_from_args(&args);
        assert!(config.is_ok());
    }
}
