use clap::Parser;
use regex::Regex;
use rust_http_server::HTTP_RESPONSE_CODE::*;
use rust_http_server::*;
use std::fs;
use std::io::{BufReader, prelude::*};
use std::net::{SocketAddr, TcpListener, TcpStream};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 8383)]
    port: u16,
}

fn main() {
    let args = Args::parse();
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], args.port)))
        .expect(&format!("Failed to bind to port {}", args.port));
    println!("Successfully started server at port {}", args.port);
    let pool = ThreadPool::new(8);
    for stream in listener.incoming() {
        pool.execute(|| {
            handle_connection(stream.unwrap());
        });
    }
}

fn handle_connection(stream: TcpStream) {
    let contents = match parse_connection_to_header(&stream) {
        Ok(request) => {
            println!("Request: {request}");
            match parse_header_to_filepath(request) {
                Ok(filepath) => read_file(filepath),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    };
    respond(&stream, contents);
}

fn parse_connection_to_header(stream: &TcpStream) -> Result<String, HTTP_RESPONSE_CODE> {
    // read first line of buffer
    let buf_reader = BufReader::new(stream);
    match buf_reader.lines() {
        mut lines => match lines.next() {
            Some(Ok(line)) => Ok(line),
            _ => Err(HTTP_400_BAD_REQUEST),
        },
    }
}

fn parse_header_to_filepath(request: String) -> Result<String, HTTP_RESPONSE_CODE> {
    let regex = Regex::new(r"GET /(?<path>[^/].*|) HTTP/1\.1").unwrap();
    match regex.captures(&request) {
        Some(captures) => Ok(captures["path"].to_string()),
        None => Err(HTTP_400_BAD_REQUEST),
    }
}

fn read_file(filename: String) -> Result<String, HTTP_RESPONSE_CODE> {
    match fs::read_to_string(if filename == "" {
        "index.html"
    } else {
        &filename
    }) {
        Ok(file) => Ok(file),
        Err(_) => Err(HTTP_404_NOT_FOUND),
    }
}

fn respond(mut stream: &TcpStream, contents: Result<String, HTTP_RESPONSE_CODE>) {
    let (status, length, contents) = match contents {
        Ok(contents) =>
            ( format!("HTTP/1.1 {HTTP_200_OK}")
            , contents.len()
            , contents
            ),
        Err(err) =>
            ( format!("HTTP/1.1 {err}")
            , "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>404 NOT FOUND</title></head><body>Page not found. Check that you typed the address correctly, or contact the site owner.</body></html>".len()
            , "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>404 NOT FOUND</title></head><body>Page not found. Check that you typed the address correctly, or contact the site owner.</body></html>".to_string()
            ),
        };
    println!("Response: {status}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream
        .write_all(format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}").as_bytes())
        .expect("Error writing to stream");
}
