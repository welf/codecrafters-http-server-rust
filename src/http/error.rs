use std::{fmt::Display, str::Utf8Error};

use super::method::MethodError;

#[derive(Debug)]
pub enum ParseRequestError {
    Encoding,
    Method,
    Protocol,
    Request,
    Network,
}

impl ParseRequestError {
    pub const fn message(&self) -> &'static str {
        match self {
            Self::Request => "Invalid Request",
            Self::Encoding => "Invalid Request Encoding",
            Self::Method => "Invalid Request Method",
            Self::Protocol => "Invalid Request Protocol",
            Self::Network => "Network I/O Error",
        }
    }
}

impl From<Utf8Error> for ParseRequestError {
    fn from(_: Utf8Error) -> Self {
        Self::Encoding
    }
}

impl From<MethodError> for ParseRequestError {
    fn from(_: MethodError) -> Self {
        Self::Method
    }
}

impl From<std::io::Error> for ParseRequestError {
    fn from(_: std::io::Error) -> Self {
        Self::Network
    }
}

impl Display for ParseRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request parsing error: {}", self.message())
    }
}
