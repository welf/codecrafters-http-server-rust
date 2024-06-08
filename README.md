<!--toc:start-->
- [Challenge Status](#challenge-status)
- [Running the Server](#running-the-server)
<!--toc:end-->

[![progress-banner](https://backend.codecrafters.io/progress/http-server/86635622-46d8-4e71-bff5-ac8a170d13cb)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a repo for Rust solutions to the

# Challenge Status

The entry point for this `http-server` implementation is in `src/main.rs`. This simple `http-server`
implementation includes:
- [x] binding TCP listener to `127.0.0.1:4221`
- [x] responding with a `200 OK` status code to `GET` requests to `/`
- [x] responding with a `404 Not Found` status code to `GET` requests to not known paths
- [x] echoing in the body a string passed by user as the part of the path from `GET` requests to `/echo/<string_to_return>`
- [x] returning in the body the value of the `User-Agent` header in `GET` requests to `/user-agent`
- [x] implementing support for concurrent connections handling using multitheading
- [ ] implementing the `/files/{filename}` endpoint that returns on `GET` requests the content of the file with the name
`filename` in the specified directory
- [ ] implementing the `/files/{filename}` endpoint that saves on `POST` requests the content of the file to the file with
the name `filename` in the specified directory
- [ ] implementing compression support for the server

# Running the Server

1. Ensure you have `cargo (1.70)` installed locally
1. Run `./your_server.sh` to run your program, which is implemented in
   `src/main.rs`. This command compiles your Rust project, so it might be slow
   the first time you run the `http-server` on `http://127.0.0.1:4221`. Subsequent runs will be fast.
1. Open a new tab in the terminal and send requests to the server using `curl` or `netcat`. To test the performance of the server,
you can use the [`oha`](https://github.com/hatoo/oha) CLI tool to send a large number of concurrent requests to the server and
get the performance metrics in a nice TUI interface.
