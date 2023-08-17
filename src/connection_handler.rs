use async_std::net::{SocketAddr, TcpStream};
use async_std::{prelude::*, task};
use httparse::Status::{Complete, Partial};
use httparse::{Header, Request, EMPTY_HEADER};
use mongodb::bson::{doc, DateTime};
use regex::Regex;
use std::env;
use std::time::Duration;

use crate::mongodbclient::MongoDbClient;
use crate::request_validator::RequestValidator;

pub(crate) struct ParsedRequest {
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ConnectionHandler {
    stream: TcpStream,
    peer: SocketAddr,
    buffer: [u8; 1024],
}

impl ConnectionHandler {
    pub fn new(stream: TcpStream) -> Self {
        let peer = stream.peer_addr().unwrap();

        ConnectionHandler {
            stream,
            peer,
            buffer: [0; 1024],
        }
    }

    pub async fn handle_connection(&mut self, max_timeout: u8) {
        // Try to read from stream or send en error
        if self.stream.read(&mut self.buffer).await.is_err() {
            self.send_reponse("500 Internal Server Error").await;
            return;
        }

        // Ignore empty requests
        if self.buffer[0] == 0 {
            return;
        }

        // Parse equest
        let request = self.parse_request().unwrap();
        match RequestValidator::validate(&request) {
            Ok(_) => println!("OK"),
            Err(_) => {
                self.send_reponse("400 Bad Request").await;
                return;
            }
        };

        let re = Regex::new(r"^/(?<timeout>\d*)$").unwrap();
        let Some(caps) = re.captures(request.path.as_str()) else {
            self.send_reponse("400 Bad Request").await;
            return;
        };

        let mut timeout: u8 = match caps["timeout"].parse() {
            Ok(n) => n,
            Err(_) => 0,
        };

        if timeout > max_timeout {
            timeout = max_timeout;
        }

        if timeout > 0 {
            task::sleep(Duration::from_secs(timeout.try_into().unwrap())).await;
        }

        self.send_reponse("204 No Content").await;

        let mongodb_user = env::var("BUSYAPI_MONGODB_USER").unwrap_or_default();
        let mongodb_password = env::var("BUSYAPI_MONGODB_PASSWORD").unwrap_or_default();
        let mongodb_host = env::var("BUSYAPI_MONGODB_HOST").unwrap_or_default();

        let mongo_client =
            match MongoDbClient::new(mongodb_user, mongodb_password, mongodb_host, "busyapi").await
            {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{:?}", err);
                    return;
                }
            };

        match mongo_client
            .insert(
                "requests",
                doc! {
                    "timestamp": DateTime::now(),
                    "ipAddress": self.peer.ip().to_string(),
                    "timeout": u32::from(timeout)
                },
            )
            .await
        {
            Ok(_) => (),
            Err(err) => {
                eprintln!("{:?}", err);
                return;
            }
        };
    }

    pub fn parse_request(&mut self) -> Result<ParsedRequest, ()> {
        let mut headers = [EMPTY_HEADER; 16];
        let mut request = Request::new(&mut headers);

        match request.parse(&self.buffer) {
            Ok(Complete(_)) => Ok(ParsedRequest {
                method: request.method.unwrap().to_string(),
                path: request.path.unwrap().to_string(),
            }),
            Ok(Partial) => Err(()),
            Err(_) => Err(()),
        }
    }

    async fn send_reponse(&mut self, status: &str) {
        let response = format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\n\r\n");
        self.stream
            .write_all(response.as_bytes())
            .await
            .unwrap_or_default();
        self.stream.flush().await.unwrap_or_default();
    }
}
