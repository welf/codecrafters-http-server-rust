use std::default::Default;

use super::{Response, StatusCode};

#[derive(Clone, Default)]
pub struct MissingStatusCode;

#[derive(Clone, Default)]
pub struct ResponseBuilder<S> {
    status_code: S,
    headers: Option<Vec<(String, String)>>,
    body: Option<Vec<u8>>,
}

impl ResponseBuilder<MissingStatusCode> {
    pub fn new() -> Self {
        ResponseBuilder::default()
    }

    pub fn status_code(self, status_code: StatusCode) -> ResponseBuilder<StatusCode> {
        ResponseBuilder {
            status_code,
            headers: self.headers,
            body: self.body,
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
            0 => (), // No Content-Length header for empty bodies
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

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
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
        assert_eq!(response.headers.len(), 0, "No headers should be set");
        assert!(response.body.is_none(), "No body should be set");
        assert_eq!(
            response.to_string(),
            "HTTP/1.1 200 OK\r\n\r\n",
            "Response string should be 200 OK"
        );
        assert_eq!(
            response,
            Response::default(),
            "Response should be the default response"
        );
    }

    #[test]
    fn response_builder_status_code() {
        let response = ResponseBuilder::not_found().build();
        assert_eq!(response.headers.len(), 0, "No headers should be set");
        assert_eq!(
            response.status_code,
            StatusCode::NotFound,
            "Status code should be 404 Not Found"
        );
    }

    #[test]
    fn response_builder_header() {
        let response = ResponseBuilder::ok()
            .header("Content-Type", "text/html")
            .build();

        let header = ("Content-Type".to_string(), "text/html".to_string());

        assert_eq!(response.headers.len(), 1, "One header should be set");
        assert_eq!(
            response.headers[0], header,
            "Header should be Content-Type: text/html"
        );
    }

    #[test]
    fn response_builder_no_content_length_header() {
        // Test that the Content-Length header is not added when the body is empty
        let response = ResponseBuilder::ok().header("Content-Length", "0").build();

        assert_eq!(response.headers.len(), 0, "No headers should be set");
    }

    #[test]
    fn response_builder_headers() {
        let response = ResponseBuilder::ok()
            .headers(vec![("Content-Type", "text/html"), ("X-Custom", "value")])
            .body("body".as_bytes().to_vec())
            .build();

        let header_0 = ("Content-Type".to_string(), "text/html".to_string());
        let header_1 = ("X-Custom".to_string(), "value".to_string());
        let content_length_header = ("Content-Length".to_string(), "4".to_string());

        assert_eq!(response.headers.len(), 3, "Three headers should be set");
        assert_eq!(
            response.headers[0], header_0,
            "First header should be Content-Type: text/html"
        );
        assert_eq!(
            response.headers[1], header_1,
            "Second header should be X-Custom: value"
        );
        assert_eq!(
            response.headers[2], content_length_header,
            "Third header should be Content-Length: 4"
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
