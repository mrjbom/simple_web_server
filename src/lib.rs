/// Server
use std::{io, net, sync::mpsc};

pub mod config;
mod http_connection;
pub mod thread_pool;

pub struct Server<'a> {
    config: config::Config<'a>,
    tcp_listener: net::TcpListener,
    thread_pool: thread_pool::ThreadPool,

    ctrl_c_receiver: mpsc::Receiver<()>,
}

impl<'a> Server<'a> {
    /// Creates and initializes the server
    pub fn init(config: config::Config<'a>) -> Result<Self, Error> {
        // Binding TCP listener
        let tcp_listener = net::TcpListener::bind(config.socket_addr_v4)?;

        // Without this, when receiving Ctrl-C, the program will immediately shut down, without correctly terminating the threads
        let (ctrl_c_sender, ctrl_c_receiver) = mpsc::channel::<()>();
        ctrlc::set_handler(move || {
            ctrl_c_sender
                .send(())
                .expect("Could not send CTRL-C signal on channel.");
        })
        .expect("Error setting Ctrl-C handler");

        // Create thread pool
        let thread_pool = thread_pool::ThreadPool::new(config.threads_number);

        Ok(Server {
            config,
            tcp_listener,
            thread_pool,
            ctrl_c_receiver,
        })
    }

    /// Handles incoming connections in loop
    pub fn run(&self) {
        loop {
            // Ctrl-C handling
            let result = self.ctrl_c_receiver.try_recv();
            match result {
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("Ctrl-C signal handler disconnect his channel");
                }
                Ok(_) => {
                    // Ctrl-C is received, shutting down the server.
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }

            // Service incoming connections
            let result = self.tcp_listener.set_nonblocking(true);
            if result.is_err() {
                return;
            }
            // Try to accept connection
            let stream = self.tcp_listener.accept();
            if let Ok((stream, _)) = stream {
                let result = stream.set_nonblocking(false);
                if result.is_err() {
                    return;
                }
                let peer_addr = stream.peer_addr();
                match peer_addr {
                    Ok(addr) => println!("{addr}"),
                    Err(error) => eprintln!("Failed to get remote address: {error}"),
                }

                // Perform connection serving
                let http_connection =
                    http_connection::HTTPConnection::new(stream, self.config.root_folder_path);
                let result = http_connection.perform();

                // Threads error serving
                if let Err(error) = result {
                    // I don't want to print timeout-related errors
                    if let http_connection::Error::RequestReadError(ref io_error) = error {
                        if io_error.kind() == io::ErrorKind::TimedOut {
                            continue;
                        }
                    }
                    eprintln!("Error in HTTP connection: {error}");
                }
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("TCP listener binding error: {0}")]
    TcpListenerBindingError(#[from] io::Error),
}
