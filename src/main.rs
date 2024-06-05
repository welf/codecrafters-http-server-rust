#![allow(unused)]
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use http::{ParseRequestError, Response, ResponseBuilder};

mod http;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response = handle_connection(&mut stream).unwrap();

                stream
                    .write_all(response.to_bytes_vec().as_slice())
                    .expect("Failed to write to stream");

                stream.flush().expect("Failed to flush stream");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<Response, ParseRequestError> {
    let mut buf_reader = BufReader::new(stream);

    let request_str = std::str::from_utf8(buf_reader.fill_buf()?)?;

    let request = http::Request::try_from(request_str)?;

    let response = match request.uri.as_str() {
        "/" => ResponseBuilder::ok().build(),
        "/user-agent" => {
            let user_agent = request
                .headers
                .iter()
                .find(|(k, _)| k == "User-Agent")
                .map(|(_, v)| v);

            match user_agent {
                Some(user_agent) => ResponseBuilder::ok()
                    .header("Content-Type", "text/plain")
                    .body(user_agent.as_bytes().to_vec())
                    .build(),
                None => ResponseBuilder::bad_request().build(),
            }
        }
        path => {
            if path.starts_with("/echo/") {
                // Remove the "/echo/" prefix from the path
                let content = path.replacen("/echo/", "", 1).as_bytes().to_vec();

                let response_builder = ResponseBuilder::ok().header("Content-Type", "text/plain");

                if content.is_empty() {
                    response_builder.build()
                } else {
                    response_builder.body(content).build()
                }
            } else {
                ResponseBuilder::not_found().build()
            }
        }
    };

    Ok(response)
}
