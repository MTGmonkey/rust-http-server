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
    let filepath = match parse_connection_to_header(&stream) {
        Ok(request) => {
            println!("Request: {request}");
            parse_header_to_filepath(request)
        }
        Err(err) => Err(err),
    };
    respond(&stream, filepath);
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
    let regex = Regex::new(r"^GET /(?<path>[^/].*|) HTTP/(1\.1|2)").unwrap();
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

fn get_file_extension(filename: String) -> Option<String> {
    let regex = Regex::new(r"^.*\.(?<extension>[a-z]+)$").unwrap();
    match regex.captures(&filename) {
        Some(captures) => Some(captures["extension"].to_string()),
        None => None,
    }
}

fn get_content_type(filename: String) -> Option<String> {
    let header = "Content-Type:";
    match &get_file_extension(filename) {
        Some(val) if val == "js" => Some(format!("{header} text/javascript")),
        Some(val) if val == "html" => Some(format!("{header} text/html")),
        Some(val) if val == "css" => Some(format!("{header} text/css")),
        Some(val) if val == "png" => Some(format!("{header} image/png")),
        Some(val) if val == "jpg" => Some(format!("{header} image/jpeg")),
        Some(val) if val == "jpeg" => Some(format!("{header} image/jpeg")),
        _ => None,
    }
}

fn respond(mut stream: &TcpStream, filename: Result<String, HTTP_RESPONSE_CODE>) {
    let (status, headers, contents) = match filename {
        Ok(filename) => {
            let filename = match filename {
                val if val == "".to_string() => "index.html".to_string(),
                filename => filename,
            };
            let contents = read_file(filename.clone());
            let content_type = match get_content_type(filename) {
                Some(header) => format!("{header}\r\n"),
                None => "".to_string(),
            };
            match contents {
                Ok(contents) =>
                    ( format!("HTTP/1.1 {HTTP_200_OK}")
                    , format!("Content-Length: {}\r\n{content_type}\r\n", contents.len())
                    , contents
                    ),
                Err(err) =>
                    ( format!("HTTP/1.1 {err}")
                    , format!("Content-Length: {}\r\nContent-Type: text/html\r\n", "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>404 NOT FOUND</title></head><body>Page not found. Check that you typed the address correctly, or contact the site owner.</body></html>".len())
                    , "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>404 NOT FOUND</title></head><body>Page not found. Check that you typed the address correctly, or contact the site owner.</body></html>".to_string()
                    ),
            }
        },
        Err(err) =>
            ( format!("HTTP/1.1 {err}")
            , format!("Content-Length: {}\r\nContent-Type: text/html\r\n", "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>404 NOT FOUND</title></head><body>Page not found. Check that you typed the address correctly, or contact the site owner.</body></html>".len())
            , "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>404 NOT FOUND</title></head><body>Page not found. Check that you typed the address correctly, or contact the site owner.</body></html>".to_string()
            ),
    };
    println!("Response: {status}\r\n{headers}\r\n{contents}");
    stream
        .write_all(format!("{status}\r\n{headers}\r\n{contents}").as_bytes())
        .expect("Error writing to stream");
}
