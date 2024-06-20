use flate2::{write::GzEncoder, Compression};

use super::{Response, StatusCode};
use std::{default::Default, io::Write};

#[derive(Clone, Debug)]
pub struct ResponseBuilder<S> {
    status_code: S,
    headers: Option<Vec<(String, String)>>,
    body: Option<Vec<u8>>,
    set_content_length_header: bool,
}

impl ResponseBuilder<MissingStatusCode> {
    pub fn new() -> ResponseBuilder<MissingStatusCode> {
        ResponseBuilder {
            status_code: MissingStatusCode,
            headers: None,
            body: None,
            set_content_length_header: true,
        }
    }

    pub fn with_status_code(self, status_code: StatusCode) -> ResponseBuilder<StatusCode> {
        ResponseBuilder {
            status_code,
            headers: self.headers,
            body: self.body,
            set_content_length_header: self.set_content_length_header,
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
        let mut headers = self.headers.unwrap_or_default();

        // Check if the Content-Encoding header is set to "gzip"
        let content_encoding_header = headers
            .iter()
            .find(|(k, v)| k == "Content-Encoding" && v == "gzip");

        let encoded_body = match self.body {
            Some(body) => match content_encoding_header {
                // If the Content-Encoding header is set to "gzip", encode the body
                Some(_) => {
                    let mut new_body = Vec::new();
                    let mut encoder = GzEncoder::new(&mut new_body, Compression::default());
                    encoder.write_all(&body).unwrap();
                    encoder.finish().unwrap();
                    Some(new_body)
                }
                // If the Content-Encoding header is not set to "gzip", return the body as is
                None => Some(body),
            },
            // If there the body is None, return it as is
            None => None,
        };

        // Calculate the Content-Length header value
        let content_length = encoded_body.as_ref().map(|b| b.len()).unwrap_or(0);

        // Set the Content-Length header if the `without_content_length_header` method was not called
        match self.set_content_length_header {
            false => (), // No Content-Length header for empty bodies
            true => headers.push(("Content-Length".to_string(), content_length.to_string())),
        }

        Response {
            status_code: self.status_code,
            headers,
            body: encoded_body,
        }
    }
}

impl<S> ResponseBuilder<S> {
    /// # Set header(s) on the response.
    ///
    /// You can set a single header multiple times, or multiple headers at once.
    ///
    /// When you set a single header, pass a tuple `(header_key, header_value)` with
    /// values implementing the `Into<String>` trait.
    ///
    /// When you set multiple headers, pass a vector of tuples `Vec<(header_key, header_value)>`
    /// with values implementing the `Into<String>` trait.
    ///
    /// Do not set the `Content-Length` header manually. It is calculated automatically based on
    /// the body length.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::ResponseBuilder;
    /// let response = ResponseBuilder::ok()
    ///     .with(("Content-Type", "text/plain"))
    ///     .with(vec![("X-Custom-Header", "value"), ("Keep-Alive", "timeout=5, max=1000")])
    ///     .build();
    ///
    /// let expected_headers = vec![
    ///     ("Content-Type".to_string(), "text/plain".to_string()),
    ///     ("X-Custom-Header".to_string(), "value".to_string()),
    ///     ("Keep-Alive".to_string(), "timeout=5, max=1000".to_string()),
    ///     ("Content-Length".to_string(), "0".to_string()),
    /// ];
    ///
    /// let response_string = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Custom-Header: value\r\nKeep-Alive: timeout=5, max=1000\r\nContent-Length: 0\r\n\r\n";
    ///
    /// assert_eq!(response.headers.len(), expected_headers.len());
    /// assert_eq!(response.headers, expected_headers);
    /// assert_eq!(response.to_string(), response_string);
    /// ```
    pub fn with<T: Into<String>>(self, part: impl IntoResponsePart<T>) -> Self {
        match part.into_response_part() {
            ResponsePart::Header(key, value) => self.header(key, value),
            ResponsePart::Headers(headers) => self.headers(headers),
        }
    }

    fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
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

    fn headers(mut self, headers: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
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
    /// # Do not set the Content-Length header on the response.
    ///
    /// This method is used to not to set the Content-Length header.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::ResponseBuilder;
    /// let response = ResponseBuilder::ok()
    ///     .without_content_length_header()
    ///     .build();
    ///
    /// assert_eq!(response.headers.len(), 0);
    /// ```
    pub fn without_content_length_header(mut self) -> Self {
        self.set_content_length_header = false;
        self
    }

    /// # Set the body of the response.
    ///
    /// The body is a byte vector. To set the body, pass any value implementing the
    /// `Into<Vec<u8>>` trait.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::ResponseBuilder;
    /// let response = ResponseBuilder::ok()
    ///     .body("Hello, world!")
    ///     .build();
    ///
    /// assert_eq!(response.body, Some("Hello, world!".as_bytes().to_vec()));
    /// ```
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
            set_content_length_header: true,
        }
    }
}

// Auxiliary type to represent a missing status code in the builder
#[derive(Clone, Default)]
pub struct MissingStatusCode;

// Auxiliary enum to represent a part of a response
pub enum ResponsePart<T> {
    Header(T, T),
    Headers(Vec<(T, T)>),
}

// Trait to convert header value(s) into a response part
pub trait IntoResponsePart<T> {
    fn into_response_part(self) -> ResponsePart<T>;
}

// Implement the IntoResponsePart trait for tuples of two elements to represent a single header
impl<T: Into<String>> IntoResponsePart<T> for (T, T) {
    fn into_response_part(self) -> ResponsePart<T> {
        ResponsePart::Header(self.0, self.1)
    }
}

// Implement the IntoResponsePart trait for vectors of tuples of two elements to represent multiple headers
impl<T: Into<String>> IntoResponsePart<T> for Vec<(T, T)> {
    fn into_response_part(self) -> ResponsePart<T> {
        ResponsePart::Headers(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_builder_default() {
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
    fn test_response_builder_ok() {
        let response = ResponseBuilder::ok().build();
        assert_eq!(
            response.status_code,
            StatusCode::Ok,
            "Status code should be 200 OK"
        );
    }

    #[test]
    fn test_response_builder_not_found() {
        let response = ResponseBuilder::not_found().build();
        assert_eq!(
            response.status_code,
            StatusCode::NotFound,
            "Status code should be 404 Not Found"
        );
    }

    #[test]
    fn test_response_builder_bad_request() {
        let response = ResponseBuilder::bad_request().build();
        assert_eq!(
            response.status_code,
            StatusCode::BadRequest,
            "Status code should be 400 Bad Request"
        );
    }

    #[test]
    fn test_response_builder_internal_server_error() {
        let response = ResponseBuilder::internal_server_error().build();
        assert_eq!(
            response.status_code,
            StatusCode::InternalServerError,
            "Status code should be 500 Internal Server Error"
        );
    }

    #[test]
    fn test_without_content_length_header() {
        let response = ResponseBuilder::ok()
            .without_content_length_header()
            .build();

        dbg!(&response.headers);
        assert_eq!(
            response.headers,
            vec![],
            "No headers should be set when the `without_content_length_header` method is called"
        );

        let response = ResponseBuilder::ok().build();

        assert_eq!(
            response.headers,
            vec![("Content-Length".to_string(), "0".to_string())],
            "Headers should contain Content-Length: 0"
        );

        // Manually set Content-Length header and then call without_content_length_header
        let response = ResponseBuilder::ok()
            .with(("Content-Length", "0"))
            .without_content_length_header()
            .build();

        assert_eq!(response.headers.len(), 0, "No headers should be set");
        assert_eq!(response.headers, vec![], "No headers should be set");
    }

    #[test]
    fn test_response_builder_with() {
        // Set a single header
        let response = ResponseBuilder::ok()
            .with(("Content-Type", "text/html"))
            .without_content_length_header()
            .build();

        let headers = vec![("Content-Type".to_string(), "text/html".to_string())];

        assert_eq!(response.headers.len(), 1, "One header should be set");
        assert_eq!(
            response.headers, headers,
            "Headers should contain Content-Type: text/html only"
        );

        // Set multiple headers at once
        let response = ResponseBuilder::ok()
            .with(vec![("Content-Type", "text/html"), ("X-Custom", "value")])
            .build();

        let headers = vec![
            ("Content-Type".to_string(), "text/html".to_string()),
            ("X-Custom".to_string(), "value".to_string()),
            ("Content-Length".to_string(), "0".to_string()),
        ];

        assert_eq!(response.headers.len(), 3, "Three headers should be set");
        assert_eq!(
            response.headers, headers,
            "Headers should contain Content-Type: text/html, X-Custom: value, and Content-Length: 4"
        );
    }

    #[test]
    fn test_response_builder_body() {
        let body = "Hello, world!";
        let response = ResponseBuilder::ok().body(body).build();
        assert_eq!(
            response.body.unwrap(),
            body.as_bytes().to_vec(),
            "Body should be set to the given value"
        );
    }
}
