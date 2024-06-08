use std::default::Default;

use super::{Response, StatusCode};

#[derive(Clone, Default)]
pub struct MissingStatusCode;

#[derive(Clone, Debug)]
pub struct ResponseBuilder<S> {
    status_code: S,
    headers: Option<Vec<(String, String)>>,
    body: Option<Vec<u8>>,
    content_length_header_on_empty_body: bool,
}

impl ResponseBuilder<MissingStatusCode> {
    pub fn new() -> ResponseBuilder<StatusCode> {
        ResponseBuilder::default()
    }

    pub fn status_code(self, status_code: StatusCode) -> ResponseBuilder<StatusCode> {
        ResponseBuilder {
            status_code,
            headers: self.headers,
            body: self.body,
            content_length_header_on_empty_body: self.content_length_header_on_empty_body,
        }
    }

    pub fn ok() -> ResponseBuilder<StatusCode> {
        ResponseBuilder {
            status_code: StatusCode::Ok,
            ..Default::default()
        }
    }

    pub fn not_found() -> ResponseBuilder<StatusCode> {
        ResponseBuilder {
            status_code: StatusCode::NotFound,
            ..Default::default()
        }
    }

    pub fn bad_request() -> ResponseBuilder<StatusCode> {
        ResponseBuilder {
            status_code: StatusCode::BadRequest,
            ..Default::default()
        }
    }

    pub fn internal_server_error() -> ResponseBuilder<StatusCode> {
        ResponseBuilder {
            status_code: StatusCode::InternalServerError,
            ..Default::default()
        }
    }
}

impl ResponseBuilder<StatusCode> {
    pub fn build(self) -> Response {
        // Calculate the Content-Length header value
        let content_length = self.body.as_ref().map(|b| b.len()).unwrap_or(0);
        let mut headers = self.headers.unwrap_or_default();

        match content_length {
            0 if !self.content_length_header_on_empty_body => (), // No Content-Length header for empty bodies
            _ => headers.push(("Content-Length".to_string(), content_length.to_string())),
        }

        Response {
            status_code: self.status_code,
            headers,
            body: self.body,
        }
    }
}

impl<S> ResponseBuilder<S> {
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key: String = key.into();

        if key.as_str() != "Content-Length" {
            if let Some(ref mut headers) = self.headers {
                headers.push((key, value.into()));
            } else {
                self.headers = Some(vec![(key, value.into())]);
            }
        }

        self
    }

    pub fn headers(mut self, headers: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        // Filter out the Content-Length header and convert headers' keys and values to owned strings
        let headers = headers
            .into_iter()
            .map(|(key, value)| -> (String, String) { (key.into(), value.into()) })
            .filter(|(key, _)| key.clone().as_str() != "Content-Length")
            .collect::<Vec<_>>();

        if let Some(ref mut existing_headers) = self.headers {
            existing_headers.extend(headers); // Extend existing headers
        } else {
            self.headers = Some(headers); // Set the headers
        }

        self
    }

    // This method is used to not to set the Content-Length header on empty bodies to pass codecrafters tests
    pub fn content_length_header_on_empty_body(mut self, value: bool) -> Self {
        self.content_length_header_on_empty_body = value;
        self
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }
}

impl Default for ResponseBuilder<StatusCode> {
    fn default() -> Self {
        ResponseBuilder {
            status_code: StatusCode::Ok,
            headers: None,
            body: None,
            content_length_header_on_empty_body: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_builder_default() {
        let response = ResponseBuilder::default().build();

        assert_eq!(
            response.status_code,
            StatusCode::Ok,
            "Status code should be 200 OK"
        );
        assert_eq!(
            response.headers,
            vec![("Content-Length".to_string(), "0".to_string())],
            "Headers should contain Content-Length: 0"
        );
        assert!(response.body.is_none(), "No body should be set");
        assert_eq!(
            response.to_string(),
            "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n",
            "Response string should be 200 OK"
        );
    }

    #[test]
    fn test_content_length_header_on_empty_body() {
        let response = ResponseBuilder::ok()
            .content_length_header_on_empty_body(false)
            .build();

        assert_eq!(
            response.headers,
            vec![],
            "No headers should be set when Content-Length header is disabled"
        );

        let response = ResponseBuilder::ok().build();

        assert_eq!(
            response.headers,
            vec![("Content-Length".to_string(), "0".to_string())],
            "Headers should contain Content-Length: 0"
        );
    }

    #[test]
    fn response_builder_status_code() {
        let response = ResponseBuilder::not_found()
            .content_length_header_on_empty_body(false)
            .build();

        assert_eq!(response.headers.len(), 0, "No headers should be set");
        assert_eq!(response.headers, vec![], "No headers should be set");
        assert_eq!(
            response.status_code,
            StatusCode::NotFound,
            "Status code should be 404 Not Found"
        );
    }

    #[test]
    fn response_builder_header() {
        let response = ResponseBuilder::ok()
            .content_length_header_on_empty_body(false)
            .header("Content-Type", "text/html")
            .build();

        let headers = vec![("Content-Type".to_string(), "text/html".to_string())];

        assert_eq!(response.headers.len(), 1, "One header should be set");
        assert_eq!(
            response.headers, headers,
            "Headers should contain Content-Type: text/html only"
        );
    }

    #[test]
    fn response_builder_no_content_length_header() {
        // Test that the Content-Length header is not added when the body is empty
        let response = ResponseBuilder::ok()
            .content_length_header_on_empty_body(false)
            .header("Content-Length", "0")
            .build();

        assert_eq!(response.headers.len(), 0, "No headers should be set");
        assert_eq!(response.headers, vec![], "No headers should be set");
    }

    #[test]
    fn response_builder_headers() {
        let response = ResponseBuilder::ok()
            .headers(vec![("Content-Type", "text/html"), ("X-Custom", "value")])
            .body("body".as_bytes().to_vec())
            .build();

        let headers = vec![
            ("Content-Type".to_string(), "text/html".to_string()),
            ("X-Custom".to_string(), "value".to_string()),
            ("Content-Length".to_string(), "4".to_string()),
        ];

        assert_eq!(response.headers.len(), 3, "Three headers should be set");
        assert_eq!(
            response.headers, headers,
            "Headers should contain Content-Type: text/html, X-Custom: value, and Content-Length: 4"
        );
    }

    #[test]
    fn response_builder_body() {
        let body = "Hello, world!";
        let response = ResponseBuilder::ok().body(body).build();
        assert_eq!(
            response.body.unwrap(),
            body.as_bytes().to_vec(),
            "Body should be set to the given value"
        );
    }

    #[test]
    fn response_builder_ok() {
        let response = ResponseBuilder::ok().build();
        assert_eq!(
            response.status_code,
            StatusCode::Ok,
            "Status code should be 200 OK"
        );
    }

    #[test]
    fn response_builder_not_found() {
        let response = ResponseBuilder::not_found().build();
        assert_eq!(
            response.status_code,
            StatusCode::NotFound,
            "Status code should be 404 Not Found"
        );
    }

    #[test]
    fn response_builder_bad_request() {
        let response = ResponseBuilder::bad_request().build();
        assert_eq!(
            response.status_code,
            StatusCode::BadRequest,
            "Status code should be 400 Bad Request"
        );
    }

    #[test]
    fn response_builder_internal_server_error() {
        let response = ResponseBuilder::internal_server_error().build();
        assert_eq!(
            response.status_code,
            StatusCode::InternalServerError,
            "Status code should be 500 Internal Server Error"
        );
    }
}
