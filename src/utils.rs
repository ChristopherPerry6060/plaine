//! Internal utilities for convenience.
use parity_wordlist::random_phrase;

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
