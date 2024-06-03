#![allow(unused)]
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use http::{ParseRequestError, Response};

mod http;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response_string = handle_connection(&mut stream).unwrap();

                stream
                    .write_all(response_string.as_bytes())
                    .expect("Failed to write to stream");

                stream.flush().expect("Failed to flush stream");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<String, ParseRequestError> {
    let mut buf_reader = BufReader::new(stream);

    let request_str = std::str::from_utf8(buf_reader.fill_buf()?)?;

    let request = http::Request::try_from(request_str)?;

    let response = match request.path() {
        "/" => format!("{}", Response::ok()),
        _ => format!("{}", Response::not_found()),
    };

    Ok(response)
}
