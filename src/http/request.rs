use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

use super::{method::Method, ParseRequestError};

#[derive(Debug)]
pub struct Request<'a> {
    method: Method,
    path: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    body: Vec<u8>,
}

impl<'a> Request<'a> {
    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn path(&self) -> &str {
        self.path
    }

    pub fn headers(&self) -> &Vec<(&str, &str)> {
        &self.headers
    }
}

impl<'a> TryFrom<&'a str> for Request<'a> {
    type Error = ParseRequestError;

    fn try_from(request_str: &'a str) -> Result<Self, Self::Error> {
        let first_line = request_str
            .lines()
            .next()
            .ok_or(ParseRequestError::Request)?;

        let (method, first_line) = first_line
            .split_once(' ')
            .ok_or(ParseRequestError::Request)?;

        let method = method.parse::<Method>()?;

        let (path, protocol) = first_line
            .split_once(' ')
            .ok_or(ParseRequestError::Request)?;

        if protocol != "HTTP/1.1" {
            return Err(ParseRequestError::Protocol);
        }

        Ok(Self {
            method,
            path,
            headers: Vec::new(),
            body: Vec::new(),
        })
        // todo!()
    }
}

fn get_next_request_line(request_str: &str) -> Result<&str, ParseRequestError> {
    request_str.lines().next().ok_or(ParseRequestError::Request)
}

fn get_next_word(request_line: &str) -> Option<(&str, &str)> {
    for (i, c) in request_line.chars().enumerate() {
        if c == ' ' {
            return Some((&request_line[..i], &request_line[i + 1..]));
        }
    }

    None
}
