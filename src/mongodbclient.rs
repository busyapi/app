use mongodb::bson::Document;
use mongodb::{options::ClientOptions, Client, Database};

pub struct MongoDbClient {
    db: Database,
}

impl MongoDbClient {
    pub async fn new(
        username: String,
        password: String,
        host: String,
        db: &str,
    ) -> Result<MongoDbClient, String> {
        let mut client_options = match ClientOptions::parse(format!(
            "mongodb+srv://{}:{}@{}/?retryWrites=true&w=majority",
            username, password, host
        ))
        .await
        {
            Ok(v) => v,
            Err(err) => return Err(format!("Failed to set MongoDB options, err = {:?}", err)),
        };

        // Manually set an option.
        client_options.app_name = Some("BusyAPI".to_string());

        // Get a handle to the deployment.
        let client = match Client::with_options(client_options) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to create MongoDB client, err = {:?}", e)),
        };

        let db = client.database(db);

        Ok(MongoDbClient { db })
    }

    pub async fn insert(self: &Self, collection: &str, doc: Document) -> Result<bool, String> {
        let collection = self.db.collection::<Document>(collection);
        let res = match collection.insert_one(doc, None).await {
            Ok(v) => true,
            Err(e) => return Err(format!("Failed inserting doc in MongoDB, err = {:?}", e)),
        };

        Ok(res)
    }
}
