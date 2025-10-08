use caesar_ipc::{caesar, read_request, send_response};
use std::env;
use std::fs;
use std::io::{self, ErrorKind};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use ctrlc;

fn handle_client(mut stream: UnixStream) -> io::Result<()> {
    let (shift, message) = read_request(&mut stream)?;
    eprintln!("Received message: {:?}, shift: {}", message, shift);

    let encrypted = caesar(&message, shift);
    eprintln!("Encrypted message: {:?}", encrypted);

    send_response(&mut stream, &encrypted)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} /path/to/socket", args[0]);
        std::process::exit(1);
    }
    let socket_path = &args[1];

    // If socket exists, remove it to avoid bind error
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    listener.set_nonblocking(true)?;
    println!("Server listening on {}", socket_path);

    // Use an atomic flag to know when to shutdown and cleanup
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let sp = socket_path.clone();

    // ctrlc handler to cleanup the socket file
    ctrlc::set_handler(move || {
        eprintln!("SIGINT received, shutting down...");
        r.store(false, Ordering::SeqCst);
        // attempt best-effort cleanup of socket file
        let _ = std::fs::remove_file(&sp);
    }).expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _addr)) => {
                // Spawn a thread to handle the client
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Client handling error: {}", e);
                    }
                });
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // No client ready yet → sleep a bit and re-check running flag
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                eprintln!("Accept error: {}", e);
                break;
            }
        }
    }

    // final cleanup
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path)?;
    }
    println!("Server exited cleanly.");
    Ok(())
}
