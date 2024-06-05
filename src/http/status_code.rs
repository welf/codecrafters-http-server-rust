use std::fmt::{Display, Result as FmtResult};

#[derive(Debug, PartialEq, Clone, Copy, Eq, Default)]
pub enum StatusCode {
    #[default]
    Ok = 200,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
}

impl StatusCode {
    pub fn message(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::NotFound => "Not Found",
            StatusCode::InternalServerError => "Internal Server Error",
        }
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> FmtResult {
        write!(f, "HTTP/1.1 {} {}\r\n", *self as u16, self.message())
    }
}
