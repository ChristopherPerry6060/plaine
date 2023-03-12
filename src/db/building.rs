use super::{ready::Ready, Mongo};
use anyhow::Result;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};
use std::marker::PhantomData;

/// Type state pattern used for [`Mongo`].
#[derive(Debug)]
pub struct Building;

impl Mongo<Building> {
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
