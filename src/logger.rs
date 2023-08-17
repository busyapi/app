use std::sync::Arc;

use mongodb::bson::{doc, DateTime};

use crate::{config::Config, mongodbclient::MongoDbClient};

pub(crate) struct Logger {
    mongo_client: MongoDbClient,
    config: Arc<Config>,
}

impl Logger {
    pub async fn new(config: Arc<Config>) -> Result<Logger, String> {
        let mongo_client = match MongoDbClient::new(config.clone()).await {
            Ok(v) => v,
            Err(err) => return Err(err),
        };

        Ok(Logger {
            config,
            mongo_client,
        })
    }

    pub async fn log(&self, ip_address: String, timeout: u8) {
        let _ = self
            .mongo_client
            .insert(
                self.config.mongo_collection.as_ref().unwrap(),
                doc! {
                    "timestamp": DateTime::now(),
                    "ipAddress": ip_address,
                    "timeout": u32::from(timeout)
                },
            )
            .await;
    }
}
