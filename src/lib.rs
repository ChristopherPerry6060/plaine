pub mod plan;
pub mod read;
pub mod utils;
pub mod write;
mod item;
pub mod db;

/// A [`Tree`] that has been serialized to a Json string.
pub type TreeJson = String;

/// An owned `String` representing left side of a [`Tree`]'s name.
///
/// [`Tree`]:(crate::Tree)
pub type RootName = String;

/// A borrowed slice of a [`RootName`].
/// 
/// [`RootName`]:(crate::RootName)
pub type Rut<'a> = &'a str;

pub type Branch = String;

pub type Brn<'a> = &'a str;

/// An owned Fulfillment Network Sku
pub type Fnsku = String;
