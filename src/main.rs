#![allow(unused)]
use http::{ParseRequestError, Response, ResponseBuilder, ThreadPool};
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

mod http;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => pool.execute(|| handle_connection(stream).unwrap()),
            Err(e) => println!("error: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), ParseRequestError> {
    let mut buf_reader = BufReader::new(&mut stream);

    let request_str = std::str::from_utf8(buf_reader.fill_buf()?)?;

    let request = http::Request::try_from(request_str)?;

    let response = match request.uri.as_str() {
        "/" => ResponseBuilder::ok()
            // Disable Content-Length header generation to pass codecrafters tests
            .content_length_header_on_empty_body(false)
            .build(),

        "/user-agent" => get_user_agent_response(&request),

        path => {
            if path.starts_with("/echo/") {
                get_echo_response(&request)
            } else {
                ResponseBuilder::not_found().build()
            }
        }
    };

    stream
        .write_all(response.to_bytes_vec().as_slice())
        .expect("Failed to write to stream");

    stream.flush().expect("Failed to flush stream");

    Ok(())
}

fn get_user_agent_response(request: &http::Request) -> Response {
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

fn get_echo_response(request: &http::Request) -> Response {
    let content = request
        .uri
        .as_str()
        .replacen("/echo/", "", 1)
        .as_bytes()
        .to_vec();

    let response_builder = ResponseBuilder::ok().header("Content-Type", "text/plain");

    if content.is_empty() {
        response_builder.build()
    } else {
        response_builder.body(content).build()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        rc::Rc,
        sync::{mpsc, Arc, Mutex},
        thread,
        time::Duration,
    };

    use http::{Request, StatusCode};

    use super::*;

    #[test]
    fn test_get_user_agent_response() {
        let request =
            Request::try_from("GET /user-agent HTTP/1.1\r\nUser-Agent: curl/7.68.0\r\n\r\n")
                .unwrap();

        let response = get_user_agent_response(&request);

        let body_len_str = "curl/7.68.0".len().to_string();
        let headers: Vec<(String, String)> = vec![
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Content-Length".to_string(), body_len_str),
        ];

        assert_eq!(response.status_code, StatusCode::Ok);
        assert_eq!(response.headers, headers);
        assert_eq!(response.body, Some(b"curl/7.68.0".to_vec()));
    }

    #[test]
    fn test_get_user_agent_response_no_user_agent() {
        let request = Request::try_from("GET /user-agent HTTP/1.1\r\n\r\n").unwrap();

        let response = get_user_agent_response(&request);

        assert_eq!(response.status_code, StatusCode::BadRequest);
        assert_eq!(
            response.headers,
            vec![("Content-Length".to_string(), "0".to_string())]
        );
        assert_eq!(response.body, None);
    }

    #[test]
    fn test_get_echo_response() {
        let request = Request::try_from("GET /echo/Hello%20World HTTP/1.1\r\n\r\n").unwrap();

        let response = get_echo_response(&request);

        let body_len_str = b"Hello%20World".len().to_string();
        let headers: Vec<(String, String)> = vec![
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Content-Length".to_string(), body_len_str),
        ];

        assert_eq!(response.status_code, StatusCode::Ok);
        assert_eq!(response.headers, headers);
        assert_eq!(response.body, Some(b"Hello%20World".to_vec()));
    }

    #[test]
    fn test_get_echo_response_empty() {
        let request = Request::try_from("GET /echo/ HTTP/1.1\r\n\r\n").unwrap();

        let response = get_echo_response(&request);
        let headers: Vec<(String, String)> = vec![
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Content-Length".to_string(), "0".to_string()),
        ];

        assert_eq!(response.status_code, StatusCode::Ok);
        assert_eq!(response.headers, headers);
        assert_eq!(response.body, None);
    }

    #[test]
    fn test_threads() {
        thread::spawn(|| {
            main();
        });
    }
}
