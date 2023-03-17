use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

use http::{Request, Response};
use log::info;
use regex::Regex;

use crate::api::handlers;
use crate::Nature;

pub fn read_http_request(mut reader: BufReader<TcpStream>) -> Vec<u8> {
    let mut data = Vec::new();
    loop {
        let mut buf = Vec::new();
        reader.read_until(b'\n', &mut buf).unwrap();
        if buf.len() <= 2 {
            // \r\n end of request
            break;
        } else {
            data.extend(buf);
        }
    }
    data
}

type Handler = fn(&Nature, Vec<usize>) -> Result<Vec<u8>, serde_json::Error>;

fn route(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}

fn handle_request(
    path: &str,
    nature: &Nature,
    handlers: &Vec<(Regex, Handler)>,
) -> Result<Vec<u8>, serde_json::Error> {
    for (route, function) in handlers {
        for capture in route.captures_iter(path) {
            let resource: Vec<usize> = capture
                .iter()
                .skip(1)
                .map(|group| group.unwrap().as_str().parse().unwrap())
                .collect();
            return function(nature, resource);
        }
    }
    Ok(vec![])
}

pub fn serve_api(nature_lock: Arc<RwLock<Nature>>) {
    let listener = TcpListener::bind("0.0.0.0:9099").unwrap();

    let handlers: Vec<(Regex, Handler)> = vec![
        (
            route(r"/agents/creatures/(\d+)/thinking"),
            handlers::get_agent_thinking,
        ),
        (route(r"/agents"), handlers::get_agents),
        (route(r"/behaviours"), handlers::get_behaviours),
        (route(r"/views"), handlers::get_views),
    ];

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut http = String::new();
        reader.read_line(&mut http).unwrap();
        let path = http.split_whitespace().nth(1).unwrap();
        let nature = nature_lock.read().unwrap();
        let response = match handle_request(path, &nature, &handlers) {
            Ok(body) => Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
            Err(error) => Response::builder()
                .status(500)
                .body(format!("{:?}", error).as_bytes().to_vec())
                .unwrap(),
        };
        let status = response.status();
        let contents = String::from_utf8(response.body().clone()).unwrap();
        let length = contents.len();
        let response = format!(
            "HTTP/1.1 {status} OK\r\nContent-Length: {length}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{contents}"
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
}
