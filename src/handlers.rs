use tokio::{
    fs::OpenOptions,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::http::{Method, ParseRequestError, Request, Response, ResponseBuilder, StatusCode};

pub async fn handle_connection(
    mut stream: TcpStream,
    files_dir: &String,
) -> Result<(), ParseRequestError> {
    let mut buf_reader = BufReader::new(&mut stream);

    let request_str = std::str::from_utf8(buf_reader.fill_buf().await?)?;

    let request = Request::try_from(request_str)?;

    let path = request.uri.as_str();
    let method = request.method;

    let response = match path {
        "/" => ResponseBuilder::ok()
            .with(vec![
                ("Connection", "Keep-Alive"),
                ("Keep-Alive", "timeout=5, max=1000"),
            ])
            // Disable Content-Length header generation to pass codecrafters tests
            .without_content_length_header()
            .build(),

        "/user-agent" => get_user_agent_response(&request),

        other => {
            if other.starts_with("/echo/") {
                get_echo_response(other.trim_start_matches("/echo/"))
            } else if path.starts_with("/files/") {
                match method {
                    Method::Post => post_file_response(&request, files_dir).await,
                    _ => get_file_response(other.trim_start_matches("/files/"), files_dir).await,
                }
            } else {
                ResponseBuilder::not_found().build()
            }
        }
    };

    stream
        .write_all(response.to_bytes_vec().as_slice())
        .await
        .expect("Failed to write to stream");

    stream.flush().await.expect("Failed to flush stream");

    Ok(())
}

async fn post_file_response(request: &Request, files_dir: &String) -> Response {
    let file_name = request.uri.as_str().trim_start_matches("/files/");

    let path = format!("{}/{}", files_dir, file_name);
    let mut file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .await
    {
        Ok(file) => file,
        Err(_) => {
            return ResponseBuilder::internal_server_error()
                .without_content_length_header()
                .build()
        }
    };

    let file_content = request.body.as_ref();

    match file.write(file_content).await {
        Ok(_) => ResponseBuilder::new()
            .with_status_code(StatusCode::Created)
            .without_content_length_header()
            .build(),
        Err(_) => ResponseBuilder::bad_request().build(),
    }
}

async fn get_file_response(file_name: &str, files_dir: &String) -> Response {
    let path = format!("{}/{}", files_dir, file_name);
    let file = match tokio::fs::read(path).await {
        Ok(file) => file,
        Err(_) => {
            return ResponseBuilder::not_found()
                .without_content_length_header()
                .build()
        }
    };

    ResponseBuilder::ok()
        .with(("Content-Type", "application/octet-stream"))
        .body(file)
        .build()
}

fn get_user_agent_response(request: &Request) -> Response {
    let user_agent = request
        .headers
        .iter()
        .find(|(k, _)| k == "User-Agent")
        .map(|(_, v)| v);

    match user_agent {
        Some(user_agent) => ResponseBuilder::ok()
            .with(("Content-Type", "text/plain"))
            .body(user_agent.as_bytes().to_vec())
            .build(),
        None => ResponseBuilder::bad_request().build(),
    }
}

fn get_echo_response(content: &str) -> Response {
    let response_builder = ResponseBuilder::ok().with(("Content-Type", "text/plain"));

    if content.is_empty() {
        response_builder.build()
    } else {
        response_builder.body(content).build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{Request, StatusCode};

    #[test]
    fn test_get_user_agent_response() {
        //======================================================================
        // Test for user agent
        let request =
            Request::try_from("GET /user-agent HTTP/1.1\r\nUser-Agent: curl/7.68.0\r\n\r\n")
                .unwrap();

        let response = get_user_agent_response(&request);

        let body_len_str = "curl/7.68.0".len().to_string();
        let headers: Vec<(String, String)> = vec![
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Content-Length".to_string(), body_len_str),
        ];

        assert_eq!(response.status_code, StatusCode::Ok);
        assert_eq!(response.headers, headers);
        assert_eq!(response.body, Some(b"curl/7.68.0".to_vec()));

        //======================================================================
        // Test for no user agent
        let request = Request::try_from("GET /user-agent HTTP/1.1\r\n\r\n").unwrap();

        let response = get_user_agent_response(&request);

        assert_eq!(response.status_code, StatusCode::BadRequest);
        assert_eq!(
            response.headers,
            vec![("Content-Length".to_string(), "0".to_string())]
        );
        assert_eq!(response.body, None);
    }

    #[test]
    fn test_get_echo_response() {
        //======================================================================
        // Test for non-empty content
        let request = Request::try_from("GET /echo/Hello%20World HTTP/1.1\r\n\r\n").unwrap();

        let path = request.uri.as_str().trim_start_matches("/echo/");

        let response = get_echo_response(path);

        let body_len_str = b"Hello%20World".len().to_string();
        let headers: Vec<(String, String)> = vec![
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Content-Length".to_string(), body_len_str),
        ];

        assert_eq!(response.status_code, StatusCode::Ok);
        assert_eq!(response.headers, headers);
        assert_eq!(response.body, Some(b"Hello%20World".to_vec()));

        //======================================================================
        // Test for empty content
        let request = Request::try_from("GET /echo/ HTTP/1.1\r\n\r\n").unwrap();

        let path = request.uri.as_str().trim_start_matches("/echo/");

        let response = get_echo_response(path);
        let headers: Vec<(String, String)> = vec![
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Content-Length".to_string(), "0".to_string()),
        ];

        assert_eq!(response.status_code, StatusCode::Ok);
        assert_eq!(response.headers, headers);
        assert_eq!(response.body, None);
    }

    #[tokio::test]
    async fn test_get_file_response() {
        //======================================================================
        // Test for file found
        let root_dir = env!("CARGO_MANIFEST_DIR");
        let tmp_dir = format!("{}/tmp", root_dir);
        let files_dir = format!("{}/files", tmp_dir);
        let file_name = "test.txt";
        let file_content = "Hello World";

        // Create files directory if it doesn't exist
        tokio::fs::create_dir_all(&files_dir).await.unwrap();

        // Create temporary file
        let file_path = format!("{}/{}", files_dir, file_name);

        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .await
            .unwrap();

        tokio::fs::write(file_path, file_content).await.unwrap();

        let response = get_file_response(file_name, &files_dir).await;

        assert_eq!(response.status_code, StatusCode::Ok);
        assert_eq!(
            response.headers,
            vec![
                (
                    "Content-Type".to_string(),
                    "application/octet-stream".to_string()
                ),
                ("Content-Length".to_string(), file_content.len().to_string())
            ]
        );
        assert_eq!(response.body, Some(file_content.as_bytes().to_vec()));

        // Remove temporary directory and its contents
        tokio::fs::remove_dir_all(tmp_dir).await.unwrap();

        //======================================================================
        // Test file not found
        let response = get_file_response(file_name, &files_dir).await;

        assert_eq!(response.status_code, StatusCode::NotFound);
        assert!(response.headers.is_empty());
        assert_eq!(response.body, None);
    }

    #[tokio::test]
    async fn test_post_file_response() {
        //======================================================================
        // Test for file created
        let root_dir = env!("CARGO_MANIFEST_DIR");
        let tmp_dir = format!("{}/tmp", root_dir);
        let files_dir = format!("{}/files", tmp_dir);
        let file_name = "test.txt";
        let file_content = "Hello World";

        // Create files directory if it doesn't exist
        tokio::fs::create_dir_all(&files_dir).await.unwrap();

        let request = Request::try_from(
            format!(
                "POST /files/{} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
                file_name,
                file_content.len(),
                file_content
            )
            .as_str(),
        )
        .unwrap();

        let response = post_file_response(&request, &files_dir).await;

        assert_eq!(response.status_code, StatusCode::Created);
        assert!(response.headers.is_empty());
        assert_eq!(response.body, None);

        // Check if file was created
        let file_path = format!("{}/{}", files_dir, file_name);
        let file_content = tokio::fs::read_to_string(file_path).await.unwrap();

        assert_eq!(file_content, "Hello World");

        // Remove temporary directory and its contents
        tokio::fs::remove_dir_all(tmp_dir).await.unwrap();

        //======================================================================
        // Test for file not created
        let request = Request::try_from("POST /files/test.txt HTTP/1.1\r\n\r\n").unwrap();

        let response = post_file_response(&request, &files_dir).await;

        assert_eq!(response.status_code, StatusCode::InternalServerError);
        assert!(response.headers.is_empty());
        assert_eq!(response.body, None);
    }
}
