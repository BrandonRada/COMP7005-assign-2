use std::net::{TcpStream};
use std::io::{self, Read, Write};

/// Apply Vigenère cipher encryption or decryption.
/// `encrypt` = true for encryption, false for decryption.
pub fn vigenere(text: &str, key: &str, encrypt: bool) -> String {
    let key_bytes: Vec<u8> = key
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase() as u8 - b'a')
        .collect();
    if key_bytes.is_empty() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut j = 0;

    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            let base = if ch.is_ascii_uppercase() { b'A' } else { b'a' };
            let shift = key_bytes[j % key_bytes.len()];
            let offset = if encrypt {
                (ch as u8 - base + shift) % 26
            } else {
                (26 + ch as u8 - base - shift) % 26
            };
            out.push((base + offset) as char);
            j += 1;
        } else {
            out.push(ch);
        }
    }
    out
}


/// Send to stream: [u64 BE length][i64 BE shift][message bytes]
pub fn send_request(stream: &mut TcpStream, key: &str, message: &str) -> io::Result<()> {
    let msg_bytes = message.as_bytes();
    let key_bytes = key.as_bytes();
    let msg_len = msg_bytes.len() as u64;
    let key_len = key_bytes.len() as u64;

    stream.write_all(&key_len.to_be_bytes())?;
    stream.write_all(key_bytes)?;
    stream.write_all(&msg_len.to_be_bytes())?;
    stream.write_all(msg_bytes)?;
    Ok(())
}

/// Read request from stream: returns (shift, message)
pub fn read_request(stream: &mut TcpStream) -> io::Result<(String, String)> {
    let mut key_len_buf = [0u8; 8];
    stream.read_exact(&mut key_len_buf)?;
    let key_len = u64::from_be_bytes(key_len_buf) as usize;

    let mut key_buf = vec![0u8; key_len];
    stream.read_exact(&mut key_buf)?;
    let key = String::from_utf8_lossy(&key_buf).into_owned();

    let mut msg_len_buf = [0u8; 8];
    stream.read_exact(&mut msg_len_buf)?;
    let msg_len = u64::from_be_bytes(msg_len_buf) as usize;

    let mut msg_buf = vec![0u8; msg_len];
    stream.read_exact(&mut msg_buf)?;
    let message = String::from_utf8_lossy(&msg_buf).into_owned();

    Ok((key, message))
}

/// Send response: [u64 BE length][message bytes]
pub fn send_response(stream: &mut TcpStream, message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let len = bytes.len() as u64;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(bytes)?;
    Ok(())
}

/// Read response: returns message string
pub fn read_response(stream: &mut TcpStream) -> io::Result<String> {
    let mut len_buf = [0u8; 8];
    stream.read_exact(&mut len_buf)?;
    let len = u64::from_be_bytes(len_buf) as usize;

    let mut msg_buf = vec![0u8; len];
    stream.read_exact(&mut msg_buf)?;
    Ok(String::from_utf8_lossy(&msg_buf).into_owned())
}
