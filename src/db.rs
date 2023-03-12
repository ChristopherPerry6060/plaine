mod building;
mod ready;

pub use self::building::Building;
pub use self::ready::Ready;
use mongodb::Client;
use std::marker::PhantomData;

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
    pub fn set_database(&mut self, database: &str) -> &mut Self {
        self.database = database.to_string();
        self
    }

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
