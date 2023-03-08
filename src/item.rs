#![allow(dead_code)]
use std::ops::Deref;
use thiserror::Error;

/// An interface for searching and manipulating FBA cases containing items.
trait FbaCase {
    fn contents(&self) -> Vec<&SkuItem<u32>>;
    fn contains(&self, id: Identifier) -> bool {
        self.contents().into_iter().any(|x| x.id == id)
    }
}

impl FbaCase for SkuItem<u32> {
    fn contents(&self) -> Vec<&SkuItem<u32>> {
        vec![&self]
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

/// A Monsoon Sku.
struct MonSku(String);
impl Deref for MonSku {
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Units for FbaItem {
    type Num = u32;

    fn quantity(&self) -> Self::Num {
        self.quantity
    }

    fn adjust(&mut self, adj: i32) -> Option<Self::Num> {
        if adj.is_negative() {
            let checked: u32 = adj.try_into().ok()?;
            self.quantity.checked_sub(checked)?;
        } else {
            let checked: u32 = adj.try_into().ok()?;
            self.quantity.checked_add(checked)?;
        };
        Some(self.quantity())
    }
}

struct FbaItem {
    quantity: u32,
    identifier: Identifier,
}

enum Identifier {
    Asin(Asin),
    Upc(Upc),
    Fnsku(Fnsku),
    MonSku(MonSku),
}

/// Used with an [`Identifier`] to describe a physical quantity.
trait Units
where
    Self::Num: Eq,
{
    type Num;
    fn quantity(&self) -> Self::Num;
    fn adjust(&mut self, adj: i32) -> Option<Self::Num>;
    fn cases(&self, per_case: u32) -> Result<u32> {
        self.quantity()
    }
}

type Result<T> = std::result::Result<T, Error>;
#[derive(Error, Debug)]
enum Error {
    #[error("OverflowAdjustment")]
    OverflowAdjustment,
}

#[cfg(test)]
mod tests {
    #[test]
    fn overflow() {
        let _x: i32 = 199;
    }
}
