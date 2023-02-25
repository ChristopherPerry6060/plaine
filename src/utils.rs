use anyhow::Result;
use parity_wordlist::random_phrase;
use std::path::Path;

/// Generate a phrase with two parity words and a Uuid, delimited by a '-'.
///
/// See [`gen_pw`] for a version without the Uuid.
pub fn gen_pw_uuid() -> String {
    let word = random_phrase(1);
    let word2 = random_phrase(1);
    let uuid = uuid::Uuid::new_v4();
    format!("{word}-{word2}-{uuid}")
}

/// Generate a phrase with two parity words.
///
/// See [`gen_pw_uuid`] for a version with the Uuid.
pub fn gen_pw() -> String {
    let word = random_phrase(1);
    let word2 = random_phrase(1);
    format!("{word}-{word2}")
}

/// A `String` which represents the path to a serialized [`Trunk`].
///
/// [`Trunk`]:(plaine::Trunk)
type TrunkFileName = String;

/// Returns [`TrunkFileName`]s within the given `path`.
///
/// # Errors
///
/// The underlying file system fails at reading the given path.
///
/// [`TrunkFileName`]:(TrunkFileName)
/// [`Trunk`]:(plaine::Trunk)
pub fn gather_records<P>(path: P) -> Result<Vec<TrunkFileName>>
where
    P: AsRef<Path>,
{
    let entries = std::fs::read_dir(path)?.map(|res| res.map(|e| e.file_name()));
    let strings = entries
        .flat_map(|os_str| os_str.map(|x| x.into_string()))
        .flatten()
        .filter(|x| x.ends_with(".json"))
        .collect::<Vec<_>>();
    Ok(strings)
}
