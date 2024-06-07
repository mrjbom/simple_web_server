/// Server
use std::{io, net};
use thiserror::Error;

pub mod config;
mod http_connection;

pub struct Server<'a> {
    config: config::Config<'a>,
    tcp_listener: net::TcpListener,
}

impl<'a> Server<'a> {
    /// Creates and initializes the server
    pub fn init(config: config::Config<'a>) -> Result<Self, Error> {
        // Binding TCP listener
        let tcp_listener = net::TcpListener::bind(config.socket_addr_v4)?;
        Ok(Server {
            config,
            tcp_listener,
        })
    }

    /// Handles incoming connections
    pub fn run(&self) {
        for stream in self.tcp_listener.incoming() {
            print!("New connection from ");
            if let Ok(stream) = stream {
                let peer_addr = stream.peer_addr();
                match peer_addr {
                    Ok(addr) => println!("{addr}"),
                    Err(error) => eprintln!("Failed to get remote address: {error}"),
                }
                // Perform connection serving
                let http_connection = http_connection::HTTPConnection::new(stream);
                let result = http_connection.perform();
                if let Err(error) = result {
                    eprintln!("Error in HTTP connection: {error}");
                }
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("TCP listener binding error: {0}")]
    TcpListenerBindingError(#[from] io::Error),
}
