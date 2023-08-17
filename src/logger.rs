use mongodb::bson::{doc, DateTime};

use crate::mongodbclient::MongoDbClient;

pub(crate) struct Logger {
    mongo_client: MongoDbClient,
}

impl Logger {
    pub async fn new() -> Result<Self, String> {
        let mongo_client = match MongoDbClient::new("busyapi").await {
            Ok(v) => v,
            Err(err) => return Err(err),
        };

        Ok(Logger { mongo_client })
    }

    pub async fn log(&self, ip_address: String, timeout: u8) {
        let _ = self
            .mongo_client
            .insert(
                "requests",
                doc! {
                    "timestamp": DateTime::now(),
                    "ipAddress": ip_address,
                    "timeout": u32::from(timeout)
                },
            )
            .await;
    }
}
