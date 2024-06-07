// Server

use std::{io, net};
use thiserror::Error;

pub mod config;

pub struct Server<'a> {
    config: config::Config<'a>,
    tcp_listener: net::TcpListener,
}

impl<'a> Server<'a> {
    /// Creates and initializes the server
    pub fn init(config: config::Config<'a>) -> Result<Self, ServerError> {
        // Binding TCP listener
        let tcp_listener = net::TcpListener::bind(config.socket_addr_v4)?;
        Ok(Server {
            config,
            tcp_listener,
        })
    }
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("TCP listener binding error: {0}")]
    TcpListenerBindingError(#[from] io::Error),
}
