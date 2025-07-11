use thiserror::Error;

pub mod entities;
pub mod ids;
pub mod indices;
pub mod search;
pub mod tables;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Tried to insert row with existing ID: `{0}` for entity `{1}`")]
    EntityAlreadyExists(String, &'static str),
    #[error("Entity not found with ID: `{0}` for entity `{1}`")]
    EntityNotFound(String, &'static str),
}

pub mod prelude {
    pub use crate::Error;
    pub use crate::entities::Entity;
    pub use crate::ids::Id;
    pub use crate::tables::{Entry, Table};
    pub use codegen::Entity;
    pub use codegen::Searchable;
    pub use codegen::index;
}
