use std::{net, path};
use std::net::SocketAddrV4;
use thiserror::Error;

/// Server config
#[derive(Debug)]
pub struct Config<'a> {
    socket_addr_v4: net::SocketAddrV4,
    root_folder_path: &'a path::Path,
    threads_number: u8,
}

impl<'a> Config<'a> {
    pub fn new(socket_addr_v4: SocketAddrV4, root_folder_path: &'a path::Path, threads_number: u8) -> Self {
        Self {
            socket_addr_v4,
            root_folder_path,
            threads_number,
        }
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
