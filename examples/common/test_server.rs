use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::result::Result;

use bytes::Bytes;
use httparse::{Header, Request};

type Error = Box<dyn std::error::Error>;

const MAX_BODY_SIZE: usize = 4096;

pub fn test_server(addr: &str) -> std::thread::JoinHandle<()> {
    let listener = TcpListener::bind(addr).unwrap();
    std::thread::spawn(move || {
        let mut buf = [0; 512];
        let mut resources = HashMap::<String, String>::default();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut headers = [httparse::EMPTY_HEADER; 16];
                    match parse_stream(&mut stream, &mut headers, &mut buf) {
                        Ok((req, body)) => {
                            if let Some(response_str) = response_body(req, body, &mut resources) {
                                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                                for line in response_str.lines() {
                                    stream.write_all(line.as_bytes()).unwrap();
                                }
                            } else {
                                stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                            }
                            stream.flush().unwrap();
                        }
                        Err(e) => {
                            eprintln!("test_server encountered stream parsing error: {e:?}")
                        }
                    }
                }
                Err(e) => eprintln!("test_server encountered IO error: {e:?}"),
            }
        }
    })
}

fn parse_stream<'st, 'bf>(
    stream: &'st mut TcpStream,
    headers: &'st mut [Header<'bf>],
    buf: &'bf mut [u8],
) -> Result<(Request<'st, 'bf>, Option<Bytes>), Error> {
    let stream_count = stream.read(buf)?;
    let mut req = Request::new(headers);
    let parse_count = req.parse(buf)?.unwrap();

    let mut body: Option<Bytes> = None;
    for Header { name, value } in &mut *req.headers {
        if *name == "content-length" {
            let content_length: usize = String::from_utf8_lossy(value).as_ref().parse()?;
            if content_length > 0 {
                let mut body_owned = Vec::with_capacity(content_length);
                if parse_count < stream_count {
                    body_owned.extend_from_slice(&buf[parse_count..stream_count]);
                } else {
                    assert!(content_length <= MAX_BODY_SIZE);
                    let mut body_buf = [0; MAX_BODY_SIZE];
                    #[allow(clippy::unused_io_amount)]
                    stream.read(&mut body_buf)?;
                    body_owned.extend_from_slice(&body_buf[..content_length]);
                }
                body = Some(Bytes::from(body_owned));
            }
            break;
        }
    }

    Ok((req, body))
}

fn response_body<'rs>(
    req: Request,
    body: Option<Bytes>,
    resources: &'rs mut HashMap<String, String>,
) -> Option<&'rs str> {
    match (req.method, req.path, body) {
        (Some("GET"), Some(path), _) => resources.get(path).map(|r| r.as_str()),
        (Some("PUT"), Some(path), Some(body)) => {
            let body_string = String::from_utf8_lossy(&body[..]).to_string();
            resources.insert(path.to_string(), body_string);
            Some("") // 200 OK
        }
        _ => unimplemented!(),
    }
}
