use anyhow::{anyhow, Result};
use mongodb::{
    bson::Document,
    options::{ClientOptions, ResolverConfig},
    Client,
};
use std::marker::PhantomData;

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
pub struct Building;

/// Type state pattern used for [`Mongo`].
#[derive(Debug)]
pub struct Ready;

/// A handle to our MongoDb cluster, implemented as  a type-state pattern.
///
/// initially, [`Self`] holds a marker in the [`Building`] state. Calling
/// [`build`] after configuring credentials will mutate the state to
/// [`Ready`]. Implementation for over the wire functions are only available
/// when [`Self`] is marked as [`Ready`].
///
/// [`build`]: (Self::build())
/// [`Building`]: (Building)
/// [`Ready`]: (Ready)
#[derive(Debug)]
pub struct Mongo<State = Building> {
    client: Option<Client>,
    database: String,
    password: String,
    state: PhantomData<State>,
    username: String,
}

impl<State> Default for Mongo<State> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> Mongo<State> {
    pub fn new() -> Self {
        Self {
            client: None,
            database: String::default(),
            password: String::default(),
            state: PhantomData,
            username: String::default(),
        }
    }
}
impl Mongo<Ready> {
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

impl Mongo<Building> {
    /// Set the database for [`Self`].
    pub fn set_database(&mut self, database: &str) -> &mut Self {
        self.database = database.to_string();
        self
    }

    /// Returns a built [`Self`] with the current configuration.
    ///
    /// # Errors
    ///
    /// This function can error due to any issues connecting to the
    /// MongoDb cluster. These errors might be caused by networking issues,
    /// authentication issues, and any other issues found within the underlying
    /// [`Client`].
    ///
    /// [`Client`]:(mongodb::Client::with_options)
    pub async fn build(&self) -> Result<Mongo<Ready>> {
        let user = &self.username;
        let pw = &self.password;
        let srv = format!(
            "mongodb+srv://{user}:{pw}\
                      @plaine-cluster.tqhag7f.mongodb.net\
                      /?retryWrites=true&w=majority"
        );

        let resolver = ResolverConfig::cloudflare();
        let opt = ClientOptions::parse_with_resolver_config(srv, resolver).await?;
        let client = Some(Client::with_options(opt)?);

        Ok(Mongo {
            client,
            database: self.database.to_string(),
            password: self.password.to_string(),
            state: PhantomData,
            username: self.username.to_string(),
        })
    }

    /// Set the password within [`Self`].
    pub fn set_password(&mut self, pw: &str) -> &mut Self {
        self.password = pw.to_string();
        self
    }

    /// Set the username within [`Self`].
    pub fn set_user(&mut self, user: &str) -> &mut Self {
        self.username = user.to_string();
        self
    }
}

