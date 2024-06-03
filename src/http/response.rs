use std::fmt::Display;

use super::status_code::StatusCode;

pub struct Response {
    status_code: StatusCode,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Response {
    pub fn new(status_code: StatusCode) -> Self {
        Response {
            status_code,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::Ok)
    }

    pub fn bad_request() -> Self {
        Self::new(StatusCode::BadRequest)
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NotFound)
    }

    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::InternalServerError)
    }

    pub fn set_status_code(&mut self, status_code: StatusCode) -> &mut Self {
        self.status_code = status_code;
        self
    }

    pub fn set_header(&mut self, key: &str, value: &str) -> &mut Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn set_body(&mut self, body: Vec<u8>) -> &mut Self {
        self.body = body;
        self
    }

    pub fn status_code(&self) -> &StatusCode {
        &self.status_code
    }

    pub fn headers(&self) -> &Vec<(String, String)> {
        &self.headers
    }

    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut headers = String::new();

        for (key, value) in self.headers.iter() {
            headers.push_str(&format!("{}: {}\r\n", key, value));
        }

        write!(
            f,
            "HTTP/1.1 {} {}\r\n\r\n",
            self.status_code as u16,
            self.status_code.message()
        )
    }
}
