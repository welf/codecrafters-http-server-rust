<!--toc:start-->
- [Challenge Status](#challenge-status)
- [Running the Server](#running-the-server)
<!--toc:end-->

[![progress-banner](https://backend.codecrafters.io/progress/http-server/86635622-46d8-4e71-bff5-ac8a170d13cb)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a repo for Rust solutions to the [http-server challenge on Codecrafters](https://codecrafters.io/challenges/http-server).

# Challenge Status

The entry point for this `http-server` implementation is in `src/main.rs`. This simple `http-server`
implementation includes:
- [x] user-friendly API for [parsing HTTP requests](./src/http/request.rs)
- [x] user-friendly, elegant, and type-safe API for [building HTTP responses](./src/http/response_builder.rs)
- [x] support for concurrent connections handling [using thread pool](./src/http/thread_pool.rs) with
a configurable number of threads *(deprecated in favor of async/await)*
- [x] support for concurrent connections handling using multi-threading with async/await
- [ ] compression support for the server
- support for the following endpoints:
  - [x] `/` - returns `200 OK` status code
  - [x] `/echo/<string_to_return>` - echoes the string passed by user in the URL
  - [x] `/user-agent` - echoes the `User-Agent` header value
  - [ ] `/files/{filename}` - returns the content of the file with the name `filename` in the specified directory
  - [ ] `/files/{filename}` - saves the content of the response to the file with the name `filename` in the specified directory

# Running the Server

1. Ensure you have `cargo (>=1.70)` installed locally
1. Run `./your_server.sh` to run your program, which is implemented in `src/main.rs`. This command compiles your Rust project,
so it might be slow the first time you run the `http-server` on `http://127.0.0.1:4221`. Subsequent runs will be fast.
1. Open a new tab in the terminal and send requests to the server using `curl` or `netcat`. To test the performance of the server,
you can use the [`oha` CLI tool](https://github.com/hatoo/oha) to send a large number of concurrent requests to the server and
get the performance metrics in a nice TUI interface. An example of how to use `oha` is shown below:
```bash
# Send 10000 requests with 10 concurrent connections
$ oha -n 10000 -c 10 http://127.0.0.1:4221/echo/hello
```
