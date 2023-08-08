mod mongodbclient;

use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task::{self, spawn};
use clap::Parser;
use futures::stream::StreamExt;
use mongodb::bson::{doc, DateTime};
use mongodbclient::MongoDbClient;
use regex::Regex;
use std::env;
use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = String::from("0.0.0.0"), help = "Bind address")]
    address: String,

    #[arg(short, long, default_value_t = 7878, help = "Bind port")]
    port: u16,

    #[arg(
        short,
        long,
        default_value_t = 60,
        help = "Maximum allowed timeout in seconds"
    )]
    max_timeout: u8,
}

#[async_std::main]
async fn main() {
    let args = Args::parse();
    let bind = format!("{}:{}", args.address, args.port);

    println!("Starting BusyAPI server on {bind}...");

    let listener = TcpListener::bind(format!("{}:{}", args.address, args.port))
        .await
        .unwrap();

    listener
        .incoming()
        .for_each_concurrent(None, |tcpstream| async move {
            let tcpstream = tcpstream.unwrap();
            let peer = tcpstream.peer_addr().unwrap();
            spawn(handle_connection(tcpstream, args.max_timeout, peer));
        })
        .await;

    println!("Shutting down.");
}

async fn handle_connection(mut stream: TcpStream, max_timeout: u8, peer: SocketAddr) {
    let mut buffer = [0; 20];
    let status = stream.read(&mut buffer).await;

    if status.is_err() {
        send_reponse(stream, "500 Internal Server Error").await;
        return;
    }

    let req = std::str::from_utf8(&buffer[..]).unwrap_or("");

    if req.eq("") {
        send_reponse(stream, "400 Bad Request").await;
        return;
    }

    let re =
        Regex::new(r"^(GET|POST|PUT|PATCH|DELETE|OPTIONS) /(?<timeout>\d*) HTTP/1.(0|1)").unwrap();
    let Some(caps) = re.captures(req) else {
        send_reponse(stream, "400 Bad Request").await;
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

    send_reponse(stream, "204 No Content").await;

    let mongodb_user = env::var("BUSYAPI_MONGODB_USER").unwrap_or_default();
    let mongodb_password = env::var("BUSYAPI_MONGODB_PASSWORD").unwrap_or_default();
    let mongodb_host = env::var("BUSYAPI_MONGODB_HOST").unwrap_or_default();

    let mongo_client =
        match MongoDbClient::new(mongodb_user, mongodb_password, mongodb_host, "busyapi").await {
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
                "ipAddress": peer.ip().to_string(),
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

async fn send_reponse(mut stream: TcpStream, status: &str) {
    let response = format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\n\r\n");
    stream
        .write_all(response.as_bytes())
        .await
        .unwrap_or_default();
    stream.flush().await.unwrap_or_default();
}
