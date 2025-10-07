use std::{io::Result, net::TcpListener};

fn main() {
    println!("Hello, world!");
    if let Err(e) = run() {
        eprintln!("server error: {e}");
    }
}

fn run() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    Ok(())
}
