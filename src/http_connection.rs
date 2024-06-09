use std::path::PathBuf;
use std::{fs, io, io::BufRead, net, path, string, time};

const MAX_REQUEST_READ_SIZE: usize = 4096;
const READ_TIMEOUT_MILLIS: u64 = 5000;

/// HTTP connection.
/// Manages the connection, parses the request and generates a response.
pub struct HTTPConnection<'a> {
    tcp_stream: net::TcpStream,
    root_folder_path: &'a path::Path,
}

impl<'a> HTTPConnection<'a> {
    pub fn new(tcp_stream: net::TcpStream, root_folder_path: &'a path::Path) -> Self {
        Self {
            tcp_stream,
            root_folder_path,
        }
    }

    /// Checks and performs the HTTP connection
    pub fn perform(self) -> Result<(), Error> {
        let mut stream = self.tcp_stream;
        // Thread will wait for a suitable HTTP request or until the amount of data exceeds MAX_REQUEST_READ_SIZE for an unlimited amount of time.
        // I don't need it, so the connection should be terminated if the data doesn't arrive within READ_TIMEOUT_MILLIS milliseconds.
        // Although, the client can still send a small amount of data (for example, 1 byte once per READ_TIMEOUT_MILLIS - 1 millisecond) and occupy the thread.
        // I do not know how to deal with this (it may be worth limiting the connection time in general).
        // It doesn't matter in this project.
        let _ = stream.set_read_timeout(Some(time::Duration::from_millis(READ_TIMEOUT_MILLIS)));
        let mut buf_reader = io::BufReader::new(&stream);

        // Check and read request
        let request = read_http_request(&mut buf_reader)?;
        // HTTP request has been read
        //println!("request:\n\"{request}\"");
        //println!("request length: {}", request.len());
        //println!("{path:?}");

        // Prepare requested file path
        // Root path + path from HTTP request
        // Get root folder
        let root_folder: PathBuf = self.root_folder_path.into();
        // Get path from HTTP request
        let mut http_requested_path = get_requested_path(&request)?;
        // If a folder is requested, it should be returned index.html from this folder
        if http_requested_path.is_dir() {
            http_requested_path.push("index.html");
        }
        // Remove prefix "/" from http requested path
        let http_requested_path = http_requested_path.strip_prefix("/");
        if let Err(_error) = http_requested_path {
            return Err(Error::WrongRequest);
        }
        let http_requested_path = http_requested_path.unwrap();
        // Root folder + path from HTTP
        let full_path = root_folder.join(http_requested_path);

        // Try to read requested file content
        let requested_file_content: Option<String> = get_file_content(&full_path);
        // Forms HTTP answer
        let answer = form_http_answer(requested_file_content.as_ref());
        //println!("answer:\n\"{answer}\"");

        // Create BufWriter
        let mut buf_writer = io::BufWriter::new(&mut stream);
        // Write HTTP answer
        use std::io::Write;

        let result = buf_writer.write_all(answer.as_bytes());
        if let Err(error) = result {
            return Err(Error::AnswerWriteError(error));
        }
        drop(buf_writer);

        let result = stream.shutdown(net::Shutdown::Both);
        if let Err(error) = result {
            return Err(Error::ShutdownFailed(error));
        }

        Ok(())
    }
}

/// Reads the HTTP request, returns Ok(String) if it is an HTTP request, otherwise it returns an error.
fn read_http_request(mut buf_reader: impl BufRead) -> Result<String, Error> {
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
    let mut line = String::new();
    loop {
        // Read line from stream to the string
        let result = take.read_line(&mut line);
        match result {
            Err(error) => return Err(Error::RequestReadError(error)),
            // EOF reached, request is wrong or too large (> MAX_REQUEST_READ_SIZE + "GET".len())
            Ok(0) => {
                return Err(Error::WrongRequest);
            }
            Ok(_) => {
                // Final line of the HTTP request is empty
                request += line.as_str();
                if line == "\r\n" || line == "\n" {
                    break;
                }
                line.clear();
            }
        }
    }
    Ok(request)
}

fn get_requested_path(request: &String) -> Result<path::PathBuf, Error> {
    let first_line = request.lines().next().unwrap();
    // First line is "GET PATH HTTP..."
    // It is necessary to find the PATH
    let path_string: String = first_line
        .chars()
        .skip_while(|&ch| ch != ' ') // Skips first word
        .skip(1) // Skips space before PATH
        .take_while(|&ch| ch != ' ') // Takes PATH until space before HTTP...
        .collect();
    // Decode URI string from "percent-encoding"
    let path_string = urlencoding::decode(path_string.as_str())?;
    Ok(path_string.to_string().into())
}

/// Tries to get the required file, returns None if it failed to do so.
// In a good way, I should have moved the actions related to reading server files to a separate module, but right now there is too little code.
fn get_file_content(path: &path::Path) -> Option<String> {
    match path.try_exists() {
        Ok(is_exist) => {
            if !is_exist {
                return None;
            }
        }
        Err(_error) => {
            return None;
        }
    }
    // Read requested file
    let result = fs::read_to_string(path);
    result.ok()
}

/// Forms HTTP answer
/// If the requested file was unavailable, then requested_file_content should be None
fn form_http_answer(requested_file_content: Option<&String>) -> String {
    let mut answer = String::new();
    // Adds first line
    match requested_file_content {
        None => {
            answer.push_str("HTTP/1.1 404 Not Found\r\n");
        }
        Some(_content) => {
            answer.push_str("HTTP/1.1 200 OK\r\n");
        }
    }
    // Adds Server header
    answer.push_str("Server: Simple Web Server\r\n");
    // Adds Connection header
    answer.push_str("Connection: close\r\n");
    // Adds Content-Type header
    answer.push_str("Content-Type: text/html\r\n");
    // Select content
    let content: &str;
    match requested_file_content {
        None => {
            content = NOT_FOUND_HTML_PAGE_CODE;
        }
        Some(requested_file_content) => content = requested_file_content.as_str(),
    }
    // Adds Content-Length header
    use std::fmt::Write;
    write!(&mut answer, "Content-Length: {}\r\n", content.len())
        .expect("write! macro error, so bad...");
    // Adds empty line
    answer.push_str("\r\n");
    // Adds content
    answer.push_str(content);
    answer
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to read request from socket: {0}")]
    RequestReadError(io::Error),
    #[error("Wrong request")]
    WrongRequest,
    #[error("Wrong URI in request {0}")]
    WrongUri(#[from] string::FromUtf8Error),
    #[error("Failed to write HTTP answer to socket {0}")]
    AnswerWriteError(io::Error),
    #[error("Failed to shutdown TCP connection {0}")]
    ShutdownFailed(io::Error),
}

static NOT_FOUND_HTML_PAGE_CODE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Simple Web Server</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            background-color: #f0f0f0;
            margin: 0;
            padding: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            flex-direction: column;
        }
        h1 {
            color: #333;
        }
        p {
            color: #666;
            text-align: center;
            max-width: 600px;
            margin: 0;
            font-size: 1.2em;
        }
    </style>
</head>
<body>
    <h1>404</h1>
    <p>Page Not Found</p>
</body>
</html>
"#;
