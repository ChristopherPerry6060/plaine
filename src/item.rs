#![allow(dead_code)]
use std::ops::Deref;

/// An interface for searching and manipulating FBA cases containing items.
trait FbaCase {
    fn contents(&self) -> Vec<&SkuItem<u32>>;
    fn contains(&self, id: &Identifier) -> bool {
        self.contents().into_iter().any(|x| &x.id == id)
    }
}

impl FbaCase for SkuItem<u32> {
    fn contents(&self) -> Vec<&SkuItem<u32>> {
        vec![self]
    }
}

/// Any number type representing physical units.
#[derive(Clone, Debug)]
struct Units<T: num_traits::Num>(T);

impl From<u32> for Units<u32> {
    fn from(value: u32) -> Self {
        Units(value)
    }
}

/// A quantity of [`Units`] and an [`Identifier`]
#[derive(Clone, Debug)]
struct SkuItem<T>
where
    T: num_traits::Unsigned + num_traits::Bounded,
{
    id: Identifier,
    units: Units<T>,
}

impl SkuItem<u32> {
    fn new(id: Identifier, units: Units<u32>) -> Self {
        Self { id, units }
    }
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

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use super::{
        Identifier::{self, Asin, Fnsku, Msku, Upc},
        SkuItem,
    };

    // Sanity check the the Identifier variants for deref and equality.k
    #[test]
    fn deref_identifier() {
        let asin = Identifier::asin("random-asin");
        if let Identifier::Asin(exp) = asin {
            assert!(&*exp == "random-asin");
        };
    }
}
