use super::Mongo;
use anyhow::{anyhow, Result};
use mongodb::bson::Document;

/// An item that can be serialized to a [`Document`] and stored to Mongo.
///
/// [`Document`]:(bson:Document)
pub trait MongOne {
    // REFACTOR: There could be a better way to do this. Just be sure to
    // revisit it at some point.
    fn target_collection(&self) -> String;
    fn target_id(&self) -> String;
    fn doc(&self) -> Result<Document>;
}

/// Type state pattern used for [`Mongo`].
#[derive(Debug)]
pub struct Ready;

impl Mongo<Ready> {
    pub async fn count_docs_in_collection(&self, collection: &str) -> anyhow::Result<u64> {
        let count = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("No client"))?
            .database(&self.database)
            .collection::<Document>(collection)
            .count_documents(None, None)
            .await;
        Ok(count?)
    }

    /// Query the database for arbitrary data, checking if credentials are valid
    pub async fn check_credentials(&self) -> anyhow::Result<()> {
        println!("tried");
        if let Some(client) = &self.client {
            client
                .database(&self.database)
                .list_collection_names(None)
                .await?;
        };
        Ok(())
    }
    /// Write an `entry` using [`Self`]'s current configuration.
    ///
    /// # Errors
    ///
    /// This function can error due to any issues connecting to the
    /// MongoDb cluster. These errors might be caused by networking issues,
    /// authentication issues, and any other issues found within the underlying
    /// [`Client`].
    ///
    /// [`Client`]:(mongodb::Client::with_options)
    pub async fn write_one<T>(&self, entry: T) -> Result<()>
    where
        T: MongOne,
    {
        if let Some(client) = &self.client {
            let name = entry.target_collection();
            let doc = entry.doc()?;
            client
                .database(&self.database)
                .collection::<Document>(&name)
                .insert_one(doc, None)
                .await?;
            Ok(())
        } else {
            Err(anyhow!("Expected Client"))
        }
    }
}
