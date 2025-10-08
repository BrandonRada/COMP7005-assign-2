use vigenere_ipc::{read_response, send_request, vigenere};
use std::env;
use std::io;
use std::net::TcpStream;

fn usage(prog: &str) {
    eprintln!("Usage: {} \"Message\" \"KEY\" <IP> <PORT>", prog);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        usage(&args[0]);
        std::process::exit(1);
    }

    let message = &args[1];
    let key = &args[2];
    let ip = &args[3];
    let port = &args[4];

    let addr = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect(addr)?;

    send_request(&mut stream, key, message)?;

    let encrypted = read_response(&mut stream)?;
    println!("Encrypted (from server): {}", encrypted);

    let decrypted = vigenere(&encrypted, key, false);
    println!("Decrypted (locally): {}", decrypted);

    Ok(())
}
