use std::{fmt::Display, str::Utf8Error};

use super::method::MethodError;

#[derive(Debug)]
pub struct ParseRequestError {
    pub kind: ParseRequestErrorKind,
}

#[derive(Debug, PartialEq)]
pub enum ParseRequestErrorKind {
    EncodingError,
    InvalidMethod,
    InvalidProtocol,
    InvalidRequest,
    NetworkError,
}

impl ParseRequestError {
    pub const fn message(&self) -> &'static str {
        match self.kind {
            ParseRequestErrorKind::InvalidRequest => "Invalid Request",
            ParseRequestErrorKind::EncodingError => "Invalid Request Encoding",
            ParseRequestErrorKind::InvalidMethod => "Invalid Request Method",
            ParseRequestErrorKind::InvalidProtocol => "Invalid Request Protocol",
            ParseRequestErrorKind::NetworkError => "Network I/O Error",
        }
    }
}

impl From<Utf8Error> for ParseRequestError {
    fn from(_: Utf8Error) -> Self {
        Self {
            kind: ParseRequestErrorKind::EncodingError,
        }
    }
}

impl From<MethodError> for ParseRequestError {
    fn from(_: MethodError) -> Self {
        Self {
            kind: ParseRequestErrorKind::InvalidMethod,
        }
    }
}

impl From<std::io::Error> for ParseRequestError {
    fn from(_: std::io::Error) -> Self {
        Self {
            kind: ParseRequestErrorKind::NetworkError,
        }
    }
}

impl Display for ParseRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request parsing error: {}", self.message())
    }
}
