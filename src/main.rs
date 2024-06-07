mod config;

use std::process;
use clap::Parser;

fn main() -> process::ExitCode {
    // Arguments parsing
    let args: Args = clap::Parser::parse();
    eprintln!(
        "Starting the server.\n\
        Current configuration:\n\
        Addr: {}\n\
        Root folder {}\n\
        Threads number: {}",
        args.socket_addr_v4,
        args.root_folder_path,
        args.threads_number
    );

    // Config building
    let config = config::Config::build_from_args(&args);
    if let Err(error) = config {
        eprintln!("Server configuration error:\n{error}");
        return process::ExitCode::FAILURE;
    }

    return process::ExitCode::SUCCESS;
}

/// Simple multithreaded web server
#[derive(Parser, Debug)]
struct Args {
    /// IP address and port that the server is listening on. It must be in the format IP:PORT.
    /// 127.0.0.1:7878 for example.
    #[arg(id = "addr", short, long, default_value = "127.0.0.1:7878")]
    socket_addr_v4: String,
    /// Path to the folder that contains the site files.
    #[arg(id = "root_folder", short, long, default_value = "./www")]
    root_folder_path: String,
    /// Number of threads that serve connections. Max 255.
    #[arg(short, long, default_value_t = 4)]
    threads_number: u8,
}
