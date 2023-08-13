extern crate derive_builder;

mod config;
mod connection_handler;
mod mongodbclient;

use async_std::net::TcpListener;
use async_std::task::spawn;
use clap::Parser;
use config::Config;
use futures::stream::StreamExt;
use serde::Deserialize;
use std::fs;

use crate::connection_handler::handle_connection;

#[derive(Parser, Debug, Deserialize)]
pub struct Args {
    #[arg(short, long, help = "Bind address")]
    address: Option<String>,

    #[arg(short, long, help = "Bind port")]
    port: Option<u16>,

    #[arg(
        short,
        long,
        help = "Maximum allowed timeout in seconds (max. 255 seconds)"
    )]
    max_timeout: Option<u8>,

    #[arg(short, long, help = "Path to configuration file")]
    config_file: Option<String>,
}

#[async_std::main]
async fn main() {
    let conf = configure();

    println!(
        "Starting BusyAPI server on http://{}:{}...",
        conf.address, conf.port
    );

    let listener = TcpListener::bind(format!("{}:{}", conf.address, conf.port))
        .await
        .unwrap();

    listener
        .incoming()
        .for_each_concurrent(None, |tcpstream| async move {
            let tcpstream = tcpstream.unwrap();
            let peer = tcpstream.peer_addr().unwrap();
            spawn(handle_connection(tcpstream, conf.max_timeout, peer));
        })
        .await;

    println!("Shutting down.");
}

fn configure() -> Config {
    let args = Args::parse();
    let mut conf = Config::new();

    conf.config_file = args.config_file.clone().unwrap_or(conf.config_file);

    let config_file_exists = match fs::metadata(&conf.config_file) {
        Ok(_) => true,
        Err(_) => false,
    };

    if config_file_exists {
        let contents =
            fs::read_to_string(&conf.config_file).expect("Should have been able to read the file");
        let file_conf: Args = toml::from_str(&contents).unwrap();
        conf.from_args(file_conf);
    }

    conf.from_args(args);

    conf
}
