use std::str::Lines;

use super::{Method, ParseRequestError, ParseRequestErrorKind};

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub uri: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl TryFrom<&str> for Request {
    type Error = ParseRequestError;

    fn try_from(request_str: &str) -> Result<Self, Self::Error> {
        // Get the first line of the request
        let (first_line, mut rest) = get_next_request_line(request_str)?;

        // Split the first line into the method and the rest of the line
        let (method, rest_of_line) = get_next_word(first_line).ok_or(ParseRequestError {
            kind: ParseRequestErrorKind::InvalidRequest,
        })?;

        // Parse the method
        let method = method.parse::<Method>()?;

        // Split the rest of the first line into the URI and the protocol
        let (uri, protocol) = get_next_word(rest_of_line).ok_or(ParseRequestError {
            kind: ParseRequestErrorKind::InvalidRequest,
        })?;

        let uri = uri.to_string();

        if !uri.starts_with('/') {
            return Err(ParseRequestError {
                kind: ParseRequestErrorKind::InvalidRequest,
            });
        }

        // Ensure the protocol is HTTP/1.1
        if protocol != "HTTP/1.1" {
            // We can get an empty protocol if the method or URI are missing
            if protocol.is_empty() {
                return Err(ParseRequestError {
                    kind: ParseRequestErrorKind::InvalidRequest,
                });
            }
            return Err(ParseRequestError {
                kind: ParseRequestErrorKind::InvalidProtocol,
            });
        }

        let mut headers = Vec::new();

        // Consume iterator lines until we reach an empty line
        for line in rest.by_ref() {
            // If the line is empty, we've reached the end of the headers
            if line.is_empty() {
                break;
            }

            // Parse the header
            let (header_name, header_value) = parse_header(line).ok_or(ParseRequestError {
                kind: ParseRequestErrorKind::InvalidRequest,
            })?;

            // Add the header to the headers vector
            headers.push((header_name.to_string(), header_value.to_string()));
        }

        // The rest of the request is the body
        let body: Vec<u8> = rest.flat_map(|line| line.as_bytes().to_owned()).collect();

        Ok(Self {
            method,
            uri,
            headers,
            body,
        })
        // todo!()
    }
}

fn get_next_request_line(request_str: &str) -> Result<(&str, Lines), ParseRequestError> {
    let mut lines = request_str.lines();
    let first_line = lines.next().ok_or(ParseRequestError {
        kind: ParseRequestErrorKind::InvalidRequest,
    });

    Ok((first_line?, lines))
}

fn get_next_word(request_line: &str) -> Option<(&str, &str)> {
    if request_line.is_empty() {
        return None;
    }

    for (i, c) in request_line.chars().enumerate() {
        if c == ' ' {
            return Some((&request_line[..i], &request_line[i + 1..]));
        }
    }

    Some((request_line, ""))
}

fn parse_header(header: &str) -> Option<(&str, &str)> {
    header.split_once(": ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_next_word() {
        assert_eq!(
            get_next_word("GET / HTTP/1.1"),
            Some(("GET", "/ HTTP/1.1")),
            "String slice incremental parsing"
        );
        assert_eq!(
            get_next_word("GET /"),
            Some(("GET", "/")),
            "String slice incremental parsing"
        );
        assert_eq!(
            get_next_word("GET"),
            Some(("GET", "")),
            "String slice incremental parsing"
        );
        assert_eq!(
            get_next_word(""),
            None,
            "String slice incremental parsing termiantion"
        );
    }

    #[test]
    fn test_get_next_request_line() {
        let request = "GET / HTTP/1.1\r\n";
        let line = "GET / HTTP/1.1";
        let (req, mut lines) = get_next_request_line(request).unwrap();
        assert_eq!(req, line, "First line of the request");
        assert!(lines.next().is_none(), "No more lines");

        let request = "GET /\r\nHost: localhost:8080\r\n\r\n";
        let line = "GET /";
        let (req, mut lines) = get_next_request_line(request).unwrap();
        assert_eq!(req, line, "First line of the request");
        assert_eq!(
            lines.next().unwrap(),
            "Host: localhost:8080",
            "Second line of the request"
        );
        assert_eq!(lines.next().unwrap(), "", "Third line of the request");
        assert!(lines.next().is_none(), "No more lines");

        let request = "GET";
        let line = "GET";
        let (req, mut lines) = get_next_request_line(request).unwrap();
        assert_eq!(req, line, "First line of the request");
        assert!(lines.next().is_none(), "No more lines");

        let request = "";
        let err = get_next_request_line(request).unwrap_err();
        let err_kind = ParseRequestErrorKind::InvalidRequest;
        assert_eq!(err.kind, err_kind, "Empty request");
    }

    #[test]
    fn test_try_from_request() {
        let request =
            Request::try_from("POST / HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/8.6.0\r\nAccept: */*\r\nContent-Type: application/json\r\nContent-Length: 11\r\n\r\nHello world").unwrap();
        assert_eq!(request.method, Method::Post, "Method");
        assert_eq!(request.uri, "/".to_owned(), "URI");
        assert!(request.headers.len() == 5, "Headers");

        let header_0 = ("Host".to_owned(), "localhost:4221".to_owned());
        let header_1 = ("User-Agent".to_owned(), "curl/8.6.0".to_owned());
        let header_2 = ("Accept".to_owned(), "*/*".to_owned());
        let header_3 = ("Content-Type".to_owned(), "application/json".to_owned());
        let header_4 = ("Content-Length".to_owned(), "11".to_owned());

        assert_eq!(
            request.headers[0], header_0,
            "Host header with the value localhost:4221"
        );
        assert_eq!(
            request.headers[1], header_1,
            "User-Agent header with the value curl/8.6.0"
        );
        assert_eq!(
            request.headers[2], header_2,
            "Accept header with the value */*"
        );
        assert_eq!(
            request.headers[3], header_3,
            "Content-Type header with the value application/json"
        );
        assert_eq!(
            request.headers[4], header_4,
            "Content-Length header with the value 11"
        );
        assert_eq!(request.body, b"Hello world", "Request body");

        let request = Request::try_from("POST /abc/def HTTP/1.1\r\n\r\n").unwrap();
        assert_eq!(request.method, Method::Post, "Method");
        assert_eq!(request.uri, "/abc/def", "URI");
        assert!(request.headers.is_empty(), "Headers are empty");
        assert!(request.body.is_empty(), "Request body is empty");

        let request = Request::try_from("GET / HTTP/1.0\r\n\r\n");
        let err_kind = ParseRequestErrorKind::InvalidProtocol;
        assert_eq!(
            request.unwrap_err().kind,
            err_kind,
            "Invalid protocol error"
        );

        let request = Request::try_from("GET abc HTTP/1.0\r\n\r\n");
        let err_kind = ParseRequestErrorKind::InvalidRequest;
        assert_eq!(request.unwrap_err().kind, err_kind, "Invalid request error");

        let request = Request::try_from("GETT / HTTP/1.1\r\n\r\n");
        let err_kind = ParseRequestErrorKind::InvalidMethod;
        assert_eq!(request.unwrap_err().kind, err_kind, "Invalid method error");

        let request = Request::try_from("GET HTTP/1.1\r\n\r\n");
        let err_kind = ParseRequestErrorKind::InvalidRequest;
        assert_eq!(request.unwrap_err().kind, err_kind, "Invalid request error");
    }

    #[test]
    fn test_parse_header() {
        let arg = "Host: localhost:4221";
        let expected = Some(("Host", "localhost:4221"));
        assert_eq!(parse_header(arg), expected, "Host header");

        let arg = "User-Agent: curl/8.6.0";
        let expected = Some(("User-Agent", "curl/8.6.0"));
        assert_eq!(parse_header(arg), expected, "User-Agent header");

        let arg = "Accept: */*";
        let expected = Some(("Accept", "*/*"));
        assert_eq!(parse_header(arg), expected, "Accept header");

        let arg = "Content-Type: application/json";
        let expected = Some(("Content-Type", "application/json"));
        assert_eq!(parse_header(arg), expected, "Content-Type header");

        let arg = "Content-Length: 11";
        let expected = Some(("Content-Length", "11"));
        assert_eq!(parse_header(arg), expected, "Content-Length header");
    }
}
