use std::{net, path};
use thiserror::Error;

/// Server config
#[derive(Debug)]
pub struct Config<'a> {
    pub socket_addr_v4: net::SocketAddrV4,
    pub root_folder_path: &'a path::Path,
    pub threads_number: u8,
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
