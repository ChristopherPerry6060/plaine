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

