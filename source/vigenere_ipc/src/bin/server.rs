use vigenere_ipc::{vigenere, read_request, send_response};
use std::env;
use std::io::{self, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use ctrlc;

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let (key, message) = read_request(&mut stream)?;
    eprintln!("Received message: {:?}, key: {:?}", message, key);

    let encrypted = vigenere(&message, &key, true);
    eprintln!("Encrypted message: {:?}", encrypted);

    send_response(&mut stream, &encrypted)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <IP> <PORT>", args[0]);
        std::process::exit(1);
    }

    let ip = &args[1];
    let port = &args[2];
    let addr = format!("{}:{}", ip, port);

    let listener = TcpListener::bind(&addr)?;
    listener.set_nonblocking(true)?;
    println!("Server listening on {}", addr);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        eprintln!("SIGINT received, shutting down...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, addr)) => {
                eprintln!("Client connected: {}", addr);
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Client error: {}", e);
                    }
                });
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                eprintln!("Accept error: {}", e);
                break;
            }
        }
    }

    println!("Server exited cleanly.");
    Ok(())

}
