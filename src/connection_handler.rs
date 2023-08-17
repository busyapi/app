use async_std::net::{SocketAddr, TcpStream};
use async_std::task::spawn;
use async_std::{prelude::*, task};
use httparse::Status::{Complete, Partial};
use httparse::{Request, EMPTY_HEADER};
use regex::Regex;
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::logger::Logger;
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
    config: Arc<Config>,
}

impl ConnectionHandler {
    pub fn new(config: Arc<Config>, stream: TcpStream) -> Self {
        let peer = stream.peer_addr().unwrap();

        ConnectionHandler {
            stream,
            peer,
            buffer: [0; 1024],
            config,
        }
    }

    pub async fn handle_connection(&mut self) {
        // Try to read from stream or send en error
        if self.stream.read(&mut self.buffer).await.is_err() {
            self.send_reponse("500 Internal Server Error").await;
            return;
        }

        // Ignore empty requests
        if self.buffer[0] == 0 {
            return;
        }

        // Parse request
        let request = self.parse_request().unwrap();

        // Validate the request
        if RequestValidator::validate(&request).is_err() {
            self.send_reponse("400 Bad Request").await;
            return;
        };

        // Get the timeout
        let re = Regex::new(r"^/(?<timeout>\d*)$").unwrap();
        let Some(caps) = re.captures(request.path.as_str()) else {
            self.send_reponse("400 Bad Request").await;
            return;
        };

        let mut timeout: u8 = match caps["timeout"].parse() {
            Ok(n) => n,
            Err(_) => 0,
        };

        if timeout > self.config.max_timeout {
            timeout = self.config.max_timeout;
        }

        // Log the request
        let ip_address = self.peer.ip().to_string();

        if self.config.can_log() {
            let config = self.config.clone();

            spawn(async move {
                let logger = Logger::new(config).await.unwrap();
                logger.log(ip_address, timeout).await;
            });
        }

        // Sleep if timeout > 0
        if timeout > 0 {
            task::sleep(Duration::from_secs(timeout.try_into().unwrap())).await;
        }

        // Send the response
        self.send_reponse("204 No Content").await;
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
