use std::{
    fs,
    io::{BufRead, BufReader, Result, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};
use tcp_server_pool_sync::ThreadPool;

fn main() {
    println!("Hello, world!");
    if let Err(e) = run() {
        eprintln!("Server error: {e}");
    }
}

fn run() -> Result<()> {
    let binding = TcpListener::bind("127.0.0.1:7878").expect("port used");
    let pool = ThreadPool::new(4);

    for stream in binding.incoming() {
        let stream = stream?;

        pool.execute(move || {
            if let Err(err) = handle_connection(stream) {
                eprintln!("connection error: {err}");
            }
        });
    }
    println!("Shutting down");
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buf_reader = BufReader::new(&stream);
    let mut requests_line = String::new();
    let mut headers = Vec::new();
    let response = match buf_reader.read_line(&mut requests_line) {
        Ok(0) => {
            eprintln!("Client is closed");
            return Ok(());
        }
        Ok(_) => {
            headers.push(requests_line.clone());
            let mut request_line = requests_line
                .trim_end_matches(['\r', '\n'])
                .split_whitespace();
            let method = request_line.next();
            let path = request_line.next();
            let version = request_line.next();

            match (method, path, version) {
                (Some("GET"), Some("/"), Some("HTTP/1.1")) => {
                    let status = "HTTP/1.1 200 OK\r\n";
                    let content_type = "Content-Type: text/html; charset=utf-8\r\n";
                    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/hello.html");
                    let contents = fs::read_to_string(path)?;
                    let contents_length = contents.len();
                    format!(
                        "{status}{content_type}Content-Length: {contents_length}\r\n\r\n{contents}"
                    )
                }
                (_, _, _) => {
                    let status = "HTTP/1.1 404 NOT FOUND\r\n";
                    let content_type = "Content-Type: text/html; charset=utf-8\r\n";
                    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/404.html");
                    let contents = fs::read_to_string(path)?;
                    let contents_length = contents.len();
                    format!(
                        "{status}{content_type}Content-Length: {contents_length}\r\n\r\n{contents}"
                    )
                }
            }
        }
        Err(e) => {
            return Err(e);
        }
    };

    for lines in buf_reader.lines() {
        match lines {
            Ok(line) => {
                if line.is_empty() {
                    break;
                }
                headers.push(line);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    println!("Headers: {headers:#?}");

    stream.write_all(response.as_bytes())?;

    Ok(())
}
