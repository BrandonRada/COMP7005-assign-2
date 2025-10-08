use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;

/// Apply a Caesar cipher to ASCII letters only (A-Z, a-z).
pub fn caesar(text: &str, shift: i64) -> String {
    // normalize shift to 0..25
    let s = ((shift % 26) + 26) % 26;
    let s_u8 = s as u8;

    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        // operate only on ASCII alpha characters as required
        if ch.is_ascii_alphabetic() {
            let base = if ch.is_ascii_uppercase() { b'A' } else { b'a' };
            let byte = ch as u8;
            let shifted = ((byte - base + s_u8) % 26) + base;
            out.push(shifted as char);
        } else {
            out.push(ch);
        }
    }
    out
}

/// Send to stream: [u64 BE length][i64 BE shift][message bytes]
pub fn send_request(stream: &mut UnixStream, shift: i64, message: &str) -> io::Result<()> {
    let msg_bytes = message.as_bytes();
    let len = msg_bytes.len() as u64;

    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&shift.to_be_bytes())?;
    stream.write_all(msg_bytes)?;
    Ok(())
}

/// Read request from stream: returns (shift, message)
pub fn read_request(stream: &mut UnixStream) -> io::Result<(i64, String)> {
    let mut len_buf = [0u8; 8];
    let mut shift_buf = [0u8; 8];

    // read length
    stream.read_exact(&mut len_buf)?;
    let len = u64::from_be_bytes(len_buf) as usize;

    // read shift
    stream.read_exact(&mut shift_buf)?;
    let shift = i64::from_be_bytes(shift_buf);

    // read message
    let mut msg_buf = vec![0u8; len];
    stream.read_exact(&mut msg_buf)?;
    let message = String::from_utf8_lossy(&msg_buf).into_owned();
    Ok((shift, message))
}

/// Send response: [u64 BE length][message bytes]
pub fn send_response(stream: &mut UnixStream, message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let len = bytes.len() as u64;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(bytes)?;
    Ok(())
}

/// Read response: returns message string
pub fn read_response(stream: &mut UnixStream) -> io::Result<String> {
    let mut len_buf = [0u8; 8];
    stream.read_exact(&mut len_buf)?;
    let len = u64::from_be_bytes(len_buf) as usize;

    let mut msg_buf = vec![0u8; len];
    stream.read_exact(&mut msg_buf)?;
    Ok(String::from_utf8_lossy(&msg_buf).into_owned())
}
