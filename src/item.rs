#![allow(dead_code)]
use std::ops::Deref;

/// An interface for searching and manipulating FBA cases containing items.
trait Case {
    /// Return the contents of [`Self`] as a `Vec` of borrowed [`SkuItem`]s.
    fn contents(&self) -> Vec<&SkuItem<u32>>;

    /// Return true if [`Self`] contains the [`Identifier`].
    fn contains(&self, id: &Identifier) -> bool {
        self.contents().into_iter().any(|x| &x.id == id)
    }
    fn units(&self, id: &Identifier) -> u32 {
        self.contents()
            .into_iter()
            .map(|x| if &x.id == id { x.units.deref() } else { &0 })
            .sum()
    }
}

impl Case for SkuItem<u32> {
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

impl Deref for Units<u32> {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
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
    fn new<N>(id: Identifier, units: N) -> Self
    where
        N: Into<Units<u32>>,
    {
        Self {
            id,
            units: units.into(),
        }
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
        FbaCase,
        Identifier::{self, Asin, Fnsku, Msku, Upc},
        SkuItem,
    };

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

    #[test]
    // Sanity check Case trait, contains function.
    fn sku_items() {
        let id = Identifier::fnsku("fnsku1234");
        let false_id = Identifier::fnsku("not here");

        let si = SkuItem::new(id.clone(), 32);

        assert_eq!(si.contains(&false_id), false);
        assert_eq!(si.contains(&id), true);
    }
    #[test]
    // Sanity check Case trait, units function.
    fn units() {
        let id = Identifier::upc("sku123");
        let false_id = Identifier::asin("not here");
        let si = SkuItem::new(id.clone(), 32);

        assert_eq!(si.units(&id), 32);
        assert_ne!(si.units(&false_id), 32);
    }
}
