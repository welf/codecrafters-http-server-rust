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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_code_message() {
        assert_eq!(
            StatusCode::Ok.message(),
            "OK",
            "Status code 200 should be OK"
        );
        assert_eq!(
            StatusCode::BadRequest.message(),
            "Bad Request",
            "Status code 400 should be Bad Request"
        );
        assert_eq!(
            StatusCode::NotFound.message(),
            "Not Found",
            "Status code 404 should be Not Found"
        );
        assert_eq!(
            StatusCode::InternalServerError.message(),
            "Internal Server Error",
            "Status code 500 should be Internal Server Error"
        );
    }

    #[test]
    fn status_code_display() {
        assert_eq!(
            format!("{}", StatusCode::Ok),
            "HTTP/1.1 200 OK\r\n",
            "Status code string 200 should be OK"
        );
        assert_eq!(
            format!("{}", StatusCode::BadRequest),
            "HTTP/1.1 400 Bad Request\r\n",
            "Status code string 400 should be Bad Request"
        );
        assert_eq!(
            format!("{}", StatusCode::NotFound),
            "HTTP/1.1 404 Not Found\r\n",
            "Status code string 404 should be Not Found"
        );
        assert_eq!(
            format!("{}", StatusCode::InternalServerError),
            "HTTP/1.1 500 Internal Server Error\r\n",
            "Status code string 500 should be Internal Server Error"
        );
    }
}
