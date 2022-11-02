use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}
};
use regex::Regex;
use std::collections::HashMap;

extern crate urlencoding;

use urlencoding::decode;

struct Link {
    url: String,
    title: String,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let mut links: Vec<Link> = vec![];

    let links_csv_data = fs::read_to_string("data/links.csv").unwrap();
    let rows: Vec<&str> = links_csv_data.split("\n").collect();
    for row in rows {
        let cols: Vec<&str> = row.split(",").collect();
        links.push(Link {
            title: cols[0].to_string(),
            url: cols[1].to_string()
        });
    }

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, &mut links);
    }
}


fn handle_connection(mut stream: TcpStream, links: &mut Vec<Link>) {
    let mut buf_reader = BufReader::new(&mut stream);

    let mut request_line = "".to_string();
    buf_reader.read_line(&mut request_line);

    let mut header_line = "".to_string();
    let mut header_map: HashMap<String, String> = HashMap::new();
    loop {
        buf_reader.read_line(&mut header_line);

        if header_line.len() == 2 {
            break
        }

        let vec: Vec<&str> = header_line.trim().split(": ").collect();
        header_map.insert(vec[0].to_string(), vec[1].to_string());

        header_line = "".to_string();
    }

    let re = Regex::new(r"(GET|POST) ([\w/]*) ()").unwrap();
    if let Some(caps) = re.captures(&request_line) {
        let method = &caps[1];
        let path = &caps[2];

        if path == "/" {
            let status_line = "HTTP/1.1 200 OK";

            let mut links_html = String::new();
            for link in links {
                links_html = format!("{}<div><a href='{}'>{}</a></div>", links_html, &link.url, &link.title);
            }

            let contents = format!(r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="utf-8">
                    <title>dev/tails | News</title>
                </head>
                <body>
                    <div class="header">
                        <h1>News</h1>
                    </div>
                    {}
                </body>
                </html>
            "#, links_html);
            let length = contents.len();

            let response = format!(
                "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
            );

            stream.write_all(response.as_bytes()).unwrap();
        } else if path == "/submit" {
            if method == "POST" {
                if let Some(content_length) = header_map.get("Content-Length") {
                    let content_length_int: usize = content_length.parse().unwrap();
                    let mut read_buf_vec = vec![0; content_length_int];
            
                    buf_reader.read_exact(&mut read_buf_vec);
            
                    let body = String::from_utf8(read_buf_vec).unwrap().replace("+", " ");
                    
                    let mut parsed_body_map: HashMap<String, String> = HashMap::new();
            
                    let key_values: Vec<&str> = body.split("&").collect();
                    for key_val in key_values {
                        let actual_kv: Vec<&str> = key_val.split("=").collect();
                        parsed_body_map.insert(actual_kv[0].to_string(), actual_kv[1].to_string());
                    }

                    let title = parsed_body_map.get("title").unwrap();
                    let url = parsed_body_map.get("url").unwrap();

                    links.push(Link {
                        title: decode(title).unwrap(),
                        url: decode(url).unwrap()
                    });
                }
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