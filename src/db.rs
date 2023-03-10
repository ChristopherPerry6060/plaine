use anyhow::{anyhow, Result};
use mongodb::{
    bson::Document,
    options::{ClientOptions, ResolverConfig},
    Client,
};
use std::marker::PhantomData;

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

