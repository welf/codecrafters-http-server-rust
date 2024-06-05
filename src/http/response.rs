use std::fmt::{Display, Write};

use super::status_code::StatusCode;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Response {
    pub status_code: StatusCode,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

impl Response {
    pub fn to_bytes_vec(&self) -> Vec<u8> {
        let mut response: Vec<u8> = Vec::new();

        let status_code: String = format!("{}", self.status_code);

        let mut headers: Vec<u8> = self.headers.iter().fold(vec![], |mut acc, (k, v)| {
            acc.extend_from_slice(k.as_bytes());
            acc.extend_from_slice(b": ");
            acc.extend_from_slice(v.as_bytes());
            acc.extend_from_slice(b"\r\n");
            acc
        });

        // Add additional CLRF after all headers
        headers.extend_from_slice(b"\r\n");

        if let Some(body) = &self.body {
            response.extend_from_slice(status_code.as_bytes());
            response.extend(headers);
            response.extend_from_slice(body);
        } else {
            response.extend_from_slice(status_code.as_bytes());
            response.extend(headers);
        }

        response
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let headers = self.headers.iter().fold(String::new(), |mut acc, (k, v)| {
            let _ = write!(acc, "{}: {}\r\n", k, v);
            acc
        });

        if let Some(body) = &self.body {
            write!(
                f,
                "{}{}\r\n{}",
                self.status_code,
                headers,
                String::from_utf8_lossy(body)
            )
        } else {
            write!(f, "{}{}\r\n", self.status_code, headers)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::http::ResponseBuilder;

    #[test]
    fn response_to_bytes_vec() {
        let response = ResponseBuilder::ok()
            .header("Content-Type", "text/plain")
            .body("Hello, World!")
            .build();
        let expected = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, World!".to_vec();

        assert_eq!(
            response.to_bytes_vec(),
            expected,
            "Response should be converted to bytes vector"
        );
    }

    #[test]
    fn response_to_string() {
        let response = ResponseBuilder::ok()
            .header("Content-Type", "text/plain")
            .body(b"Hello, World!".to_vec())
            .build();
        let expected = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, World!";

        assert_eq!(
            response.to_string(),
            expected,
            "Response should be converted to string"
        );
    }
}
