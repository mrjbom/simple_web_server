use std::io::{BufRead, Read};
use std::{io, net, time};
use thiserror::Error;

const MAX_REQUEST_READ_SIZE: usize = 4096;
const READ_TIMEOUT_MILLIS: u64 = 5000;

/// HTTP connection.
/// Manages the connection, parses the request and generates a response.

pub struct HTTPConnection {
    tcp_stream: net::TcpStream,
}

impl HTTPConnection {
    pub fn new(tcp_stream: net::TcpStream) -> Self {
        Self { tcp_stream }
    }

    /// Checks and performs the HTTP connection
    pub fn perform(self) -> Result<(), Error> {
        let stream = self.tcp_stream;
        // Thread will wait for a suitable HTTP request or until the amount of data exceeds MAX_REQUEST_READ_SIZE for an unlimited amount of time.
        // I don't need it, so the connection should be terminated if the data doesn't arrive within READ_TIMEOUT_MILLIS milliseconds.
        // Although, the client can still send a small amount of data (for example, 1 byte once per READ_TIMEOUT_MILLIS - 1 millisecond) and occupy the thread.
        // I do not know how to deal with this (it may be worth limiting the connection time in general).
        // It doesn't matter in this project.
        let _ = stream.set_read_timeout(Some(time::Duration::from_millis(READ_TIMEOUT_MILLIS)));
        let mut buf_reader = io::BufReader::new(&stream);

        // Need to find out if the request is an HTTP request.
        // We are only interested in GET requests,
        // so we need to make sure that the first 3 chars are "GET".
        // "GET" in UTF-8 takes 3 bytes
        let mut buf: [u8; 3] = [0; 3];
        // Reading 3 bytes
        let result = buf_reader.read_exact(&mut buf);
        if let Err(error) = result {
            return Err(Error::RequestReadError(error));
        }
        // Contains GET?
        if buf != "GET".as_bytes() {
            return Err(Error::WrongRequest);
        }

        // This is a GET request.
        // Try to read him

        // Since there is a possibility that this request is formed incorrectly and has no end,
        // we must limit the number of bytes to be read.
        let mut request = String::with_capacity(MAX_REQUEST_READ_SIZE);
        request.push_str("GET");
        // Take guarantees that we will not be able to read more than MAX_REQUEST_READ_SIZE bytes,
        // it will always return EOF
        let mut take = buf_reader.take(MAX_REQUEST_READ_SIZE as u64);
        loop {
            // Read line from stream to the string
            let result = take.read_line(&mut request);
            match result {
                Err(error) => return Err(Error::RequestReadError(error)),
                // EOF reached, request is wrong or too large (> MAX_REQUEST_READ_SIZE)
                Ok(0) => {
                    println!("{}", request.len());
                    return Err(Error::WrongRequest);
                }
                Ok(n) => {
                    // Final line of the HTTP request is empty
                    let last_line_is_empty = request.lines().last().unwrap().is_empty();
                    if last_line_is_empty {
                        break;
                    }
                }
            }
        }
        // Request has been read
        //println!("request:\n\"{request}\"");
        //println!("request length: {}", request.len());

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read request from socket: {0}")]
    RequestReadError(io::Error),
    #[error("Wrong request")]
    WrongRequest,
}
