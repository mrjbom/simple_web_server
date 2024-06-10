use simple_web_server::{config, Server};
use std::{net, path, process};

fn main() -> process::ExitCode {
    // Arguments parsing
    let args: Args = clap::Parser::parse();
    println!(
        "Starting the server.\n\
        Current configuration:\n\
        Addr: {}\n\
        Root folder {}\n\
        Threads number: {}",
        args.socket_addr_v4, args.root_folder_path, args.threads_number
    );

    // Config building
    let config = args.build_config();
    if let Err(error) = config {
        eprintln!("Server configuration error:\n{error}");
        return process::ExitCode::FAILURE;
    }
    let config = config.unwrap();

    // Server Initialization
    println!("Initialization...");
    let server = Server::init(config);
    if let Err(error) = server {
        eprintln!("Server initialization error:\n{error}");
        return process::ExitCode::FAILURE;
    }
    let server = server.unwrap();
    println!("Initialized.");

    server.run();

    return process::ExitCode::SUCCESS;
}

/// Simple multithreaded web server
#[derive(clap::Parser, Debug)]
struct Args {
    /// IP address and port that the server is listening on. It must be in the format IP:PORT.
    /// 127.0.0.1:7878 for example.
    #[arg(id = "addr", short, long, default_value = "127.0.0.1:7878")]
    socket_addr_v4: String,
    /// Path to the folder that contains the site files.
    #[arg(id = "root_folder", short, long, default_value = "./www")]
    root_folder_path: String,
    /// Number of threads that serve connections. Max 255.
    #[arg(short, long, default_value_t = 8)]
    threads_number: u8,
}

impl Args {
    pub fn build_config(&self) -> Result<config::Config, config::Error> {
        let socket_addr_v4 = self.socket_addr_v4.parse::<net::SocketAddrV4>()?;
        let root_folder_path = path::Path::new(self.root_folder_path.as_str());
        if !root_folder_path.is_dir() {
            // The path does not exist or does not point to the directory or cannot be accessed.
            return Err(config::Error::WrongRootFolderPath);
        }
        let threads_number = self.threads_number;
        if threads_number == 0 {
            return Err(config::Error::ZeroThreadsNumber);
        }

        Ok(config::Config {
            socket_addr_v4,
            root_folder_path,
            threads_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Args;
    #[test]
    fn build_config_from_args_wrong_addr() {
        let args = Args {
            socket_addr_v4: "Wrong".to_string(),
            root_folder_path: "./".to_string(),
            threads_number: 4,
        };
        let config = args.build_config();
        assert!(matches!(config, Err(config::Error::WrongAddr(_))));
    }

    #[test]
    fn build_config_from_args_wrong_path() {
        let args = Args {
            socket_addr_v4: "127.0.0.1:7878".to_string(),
            root_folder_path: "".to_string(),
            threads_number: 4,
        };
        let config = args.build_config();
        assert!(matches!(config, Err(config::Error::WrongRootFolderPath)));
    }

    #[test]
    fn build_config_from_args_zero_threads_number() {
        let args = Args {
            socket_addr_v4: "127.0.0.1:7878".to_string(),
            root_folder_path: "./".to_string(),
            threads_number: 0,
        };
        let config = args.build_config();
        assert!(matches!(config, Err(config::Error::ZeroThreadsNumber)));
    }

    #[test]
    fn build_config_from_args() {
        let args = Args {
            socket_addr_v4: "127.0.0.1:7878".to_string(),
            root_folder_path: "./".to_string(),
            threads_number: 4,
        };
        let config = args.build_config();
        assert!(config.is_ok());
    }
}
