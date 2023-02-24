pub mod read;
pub mod write;
pub mod plan;
pub mod utils;

/// A [`Tree`] that has been serialized to a Json string.
pub type TreeJson = String;

/// A universally unique identifier that is pinned to the [`Tree`].
///
/// Generally, not displayed to the user.
pub type TreeUuid = String;

/// The leftmost [`Group`] of the [`Tree`].
pub type Trunk = String;

/// A pair of words, connected by a single `-`.
///
/// # Examples
///
/// * `endearing-shredder`.
/// * `flirty-bucket`.
pub type Group = String;

/// One of more [`Group`]s that are connected to describe an item's path.
pub type Tree = String;

pub type Branch = String;
