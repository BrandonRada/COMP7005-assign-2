use caesar_ipc::{read_response, send_request, caesar};
use std::env;
use std::io;
use std::os::unix::net::UnixStream;

fn usage(prog: &str) {
    eprintln!("Usage: {} \"Message\" <shift> /path/to/socket", prog);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        usage(&args[0]);
        std::process::exit(1);
    }

    let message = &args[1];
    // parse shift as i64
    let shift: i64 = match args[2].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid shift value: {}", args[2]);
            std::process::exit(1);
        }
    };
    let socket_path = &args[3];

    // connect
    let mut stream = UnixStream::connect(socket_path)?;
    // send request
    send_request(&mut stream, shift, message)?;

    // read response
    let encrypted = read_response(&mut stream)?;
    println!("Encrypted (from server): {}", encrypted);

    // locally decrypt using same shift (i.e., apply -shift)
    let decrypted = caesar(&encrypted, -shift);
    println!("Decrypted (locally): {}", decrypted);

    Ok(())
}
