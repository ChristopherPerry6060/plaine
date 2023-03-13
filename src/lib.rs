pub mod db;
pub mod schema;

pub trait Table {
    fn headers(&self) -> Vec<&str>;
    fn row(&self) -> Vec<&str>;
}
