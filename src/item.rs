#![allow(dead_code)]
use std::ops::Deref;

/// An interface for searching and manipulating FBA cases containing items.
trait Case {
    /// Return the contents of [`Self`] as a `Vec` of borrowed [`SkuItem`]s.
    fn contents(&self) -> Vec<&SkuItem>;

    /// Return true if [`Self`] contains the [`Identifier`].
    fn contains(&self, id: &Identifier) -> bool {
        self.contents().into_iter().any(|x| &x.id == id)
    }
}

type Units = u32;

/// A quantity of [`Units`] and an [`Identifier`]
#[derive(Clone, Debug)]
struct SkuItem {
    id: Identifier,
    units: Units,
}

/// A Fulfillment Network Sku.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Fnsku(String);
impl Deref for Fnsku {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An Amazon Standard Identification Number.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Asin(String);
impl Deref for Asin {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A Universal Product Code.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Upc(String);
impl Deref for Upc {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Msku(String);
impl Deref for Msku {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Various codes and skus used to identify physical items.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Identifier {
    Asin(Asin),
    Fnsku(Fnsku),
    Msku(Msku),
    Upc(Upc),
}

impl Identifier {
    /// Construct an [`Asin`] from a `&str` clone.
    fn asin(asin: &str) -> Self {
        let asin = Asin(asin.to_string());
        Identifier::Asin(asin)
    }

    /// Construct an [`Fnsku`] from a `&str` clone.
    fn fnsku(fnsku: &str) -> Self {
        let fnsku = Fnsku(fnsku.to_string());
        Identifier::Fnsku(fnsku)
    }

    /// Construct an [`Msku`] from a `&str` clone.
    fn msku(msku: &str) -> Self {
        let msku = Msku(msku.to_string());
        Identifier::Msku(msku)
    }

    /// Construct an [`Upc`] from a `&str` clone.
    fn upc(upc: &str) -> Self {
        let upc = Upc(upc.to_string());
        Identifier::Upc(upc)
    }
}

#[cfg(test)]
mod tests {
    use super::Identifier;

    #[test]
    // Sanity check the Identifier variants for deref and equality.
    fn deref_identifier() {
        let asin = Identifier::asin("random-asin");
        if let Identifier::Asin(exp) = asin {
            assert!(&*exp == "random-asin");
        } else {
            panic!()
        };
    }
}

