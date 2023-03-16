use http::{Request, Response};
use log::info;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

fn deserialize(req: Request<Vec<u8>>) -> Request<String> {
    let (parts, body) = req.into_parts();
    let body = String::from_utf8(body).unwrap();
    Request::from_parts(parts, body)
}

fn serialize<T>(res: Response<T>) -> serde_json::Result<Response<Vec<u8>>>
where
    T: serde::Serialize,
{
    let (parts, body) = res.into_parts();
    let body = serde_json::to_vec(&body)?;
    Ok(Response::from_parts(parts, body))
}

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

pub fn serve() {
    let listener = TcpListener::bind("0.0.0.0:9099").unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let request_data = read_http_request(reader);
        // info!("HE: {}", String::from_utf8(request_data).unwrap());
        let req = deserialize(Request::new(request_data));
        info!(
            "REQ interop M:{} URI:{} BODY:{}",
            req.method(),
            req.uri(),
            req.body()
        );

        let status_line = "HTTP/1.1 200 OK";
        let mut contents = "Hello, how are you?";
        let length = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();
    }
}
