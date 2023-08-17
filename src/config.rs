use derive_builder::Builder;
use serde::Deserialize;

use crate::Args;

#[derive(Default, Builder, Debug, Deserialize, Clone)]
#[builder(setter(into))]
pub struct Config {
    pub address: String,
    pub port: u16,
    pub max_timeout: u8,
    pub config_file: String,
    pub mongo_user: Option<String>,
    pub mongo_password: Option<String>,
    pub mongo_host: Option<String>,
    pub mongo_database: Option<String>,
    pub mongo_collection: Option<String>,
}

impl Config {
    pub fn new() -> Config {
        ConfigBuilder::default()
            .address("localhost")
            .port::<u16>(7878)
            .max_timeout(60)
            .config_file("/etc/busyapi.conf")
            .mongo_user(None)
            .mongo_password(None)
            .mongo_host(None)
            .mongo_database(None)
            .mongo_collection(None)
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

        self.mongo_user = args.mongo_user;
        self.mongo_password = args.mongo_password;
        self.mongo_host = args.mongo_host;
        self.mongo_database = args.mongo_database;
        self.mongo_collection = args.mongo_collection;
    }

    pub fn can_log(&self) -> bool {
        self.mongo_user.is_some()
            && self.mongo_password.is_some()
            && self.mongo_host.is_some()
            && self.mongo_host.is_some()
            && self.mongo_collection.is_some()
    }
}
