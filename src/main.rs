use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}
};
use regex::Regex;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}


fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let re = Regex::new(r"(GET|POST) ([\w/]*) ()").unwrap();
    if let Some(caps) = re.captures(&request_line) {
        let method = &caps[1];
        let path = &caps[2];

        if path == "/" {
            let status_line = "HTTP/1.1 200 OK";
            let contents = fs::read_to_string("static/index.html").unwrap();
            let length = contents.len();

            let response = format!(
                "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
            );

            stream.write_all(response.as_bytes()).unwrap();
        } else if path == "/submit" {
            if method == "POST" {
                println!("{}", request_line);
            }
            write_file(stream, "static/submit.html");
        }
    } else {
        write_file(stream, "static/404.html");
    }
}

fn write_file(mut stream: TcpStream, filename: &str) {
    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );

    stream.write_all(response.as_bytes()).unwrap();
}