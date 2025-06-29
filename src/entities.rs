use crate::ids::Id;

/// A trait representing an entity in a database.
/// An entity is a record that can be stored in a table.
pub trait Entity {
    fn get_id(&self) -> &Id<Self>;
}
