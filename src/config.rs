use derive_builder::Builder;
use serde::Deserialize;

use crate::Args;

#[derive(Default, Builder, Debug, Deserialize)]
#[builder(setter(into))]
pub struct Config {
    pub address: String,
    pub port: u16,
    pub max_timeout: u8,
    pub config_file: String,
}

impl Config {
    pub fn new() -> Config {
        ConfigBuilder::default()
            .address("localhost")
            .port::<u16>(7878)
            .max_timeout(60)
            .config_file("/etc/busyapi.conf")
            .build()
            .unwrap()
    }

    pub fn from_args(&mut self, args: Args) {
        if args.address.is_some() {
            self.address = args.address.unwrap();
        }

        if args.port.is_some() {
            self.port = args.port.unwrap();
        }

        if args.max_timeout.is_some() {
            self.max_timeout = args.max_timeout.unwrap();
        }
    }
}
