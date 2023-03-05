#![allow(dead_code)]
use std::ops::Deref;
use thiserror::Error;

/// A Fulfillment Network Sku.
struct Fnsku(String);
impl Deref for Fnsku {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An Amazon Standard Identification Number.
struct Asin(String);
impl Deref for Asin {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A Universal Product Code.
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
