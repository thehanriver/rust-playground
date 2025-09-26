use std::io::{BufRead, BufReader, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use std::{fs, thread};

fn main() {
    println!("Hello, world!");
    // |----GAME PLAN----|
    // Single Client
    // Listen on a port
    // Accept a connection
    // Read segments in a loop
    // Write back a response
    // Graceful
    // Should be sync

    if let Err(err) = run() {
        eprintln!("Server error: {err}");
    }
}

fn run() -> Result<()> {
    let shut_down = Arc::new(AtomicBool::new(false));
    let sd = shut_down.clone();
    ctrlc::set_handler(move || sd.store(true, Ordering::SeqCst))
        .expect("failed to install crtlc handler");

    //Bind an address + port almost like new() -> How to know which ports to use? And how to error handle?
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    listener.set_nonblocking(true)?;

    //Stream processing
    while !shut_down.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, addr)) => {
                println!(
                    "Connection established: Client-{addr} and Server-{}",
                    stream.local_addr()?
                );
                handle_connection(stream)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(e),
        }
    }

    println!("Gracfully shutting down...");
    // for stream in listener.incoming() {
    //     let stream = stream?;
    //     println!(
    //         "Connection established: Cient - {} and Server - {}",
    //         stream.peer_addr()?,
    //         stream.local_addr()?
    //     );
    //     handle_connection(stream)?;
    // }

    Ok(())
}

// Just read stream first We use GET

/*
    Usually formatted like
    Method Request-URI HTTP version CRLF -> CRLF (carriage return line feed) meaning \r\n carriage return and line feed separates lines in request
    headers CRLF
    Message

    Headers usually have CLRF
    CLRF
    Body

    <status line>\r\n
    <header>\r\n
    <header>\r\n
    ... (more headers)\r\n
    \r\n
    <body>

    Syn syn ack ack is tcp level connection -> the Get is the http request after it succeeds

*/
fn handle_connection(mut stream: TcpStream) -> Result<()> {
    // 1. Read the request line into `request_line`; trim it; split whitespace.
    // 2. Guard: if method != "GET", return 405 with a tiny explanation body.
    // 3. match path {
    //        "/" => serve hello.html,
    //        _   => serve a 404 file or inline HTML
    //    }
    // 4. After the blank line, stop reading; ignore the body for now.

    let mut buf_reader = BufReader::new(&stream);
    let mut request_line = String::new();
    let mut headers = Vec::new();

    let response = match buf_reader.read_line(&mut request_line) {
        Ok(0) => {
            eprintln!("Client closed connection before sending a request line");
            return Ok(());
        }
        Ok(_) => {
            headers.push(request_line.clone());
            let request_line = request_line.trim_end_matches(['\r', '\n']);

            let mut parts = request_line.split_whitespace();
            let method = parts.next();
            let path = parts.next();
            let version = parts.next();

            match (method, path, version) {
                (Some("GET"), Some("/"), Some("HTTP/1.1")) => {
                    // hard code http response no real headers like token or anything
                    let (status, content_type) = (
                        "HTTP/1.1 200 OK\r\n",
                        "Content-Type: text/html; charset=utf-8
                    \r\n",
                    );
                    let contents = fs::read_to_string("src/hello.html")?;
                    let length = contents.len();
                    format!("{status}{content_type}Content-Length: {length}\r\n\r\n{contents}")
                }
                (_, _, _) => {
                    let (status, content_type) = (
                        "HTTP/1.1 404 NOT FOUND\r\n",
                        "Content-Type: text/html; charset=utf-8
                    \r\n",
                    );
                    let contents = fs::read_to_string("src/404.html")?;
                    let contents_length = contents.len();
                    format!(
                        "{status}{content_type}Content-Length:{contents_length}\r\n\r\n{contents}"
                    )
                }
            }
        }
        Err(e) => return Err(e),
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
                eprintln!("Reading error");
                return Err(e);
            }
        }
    }

    println!("Request: {headers:#?}");
    stream.write_all(response.as_bytes())?;

    Ok(())
}
