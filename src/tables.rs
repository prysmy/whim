use crate::Error;
use crate::entities::Entity;
use crate::ids::Id;
use crate::indices::{IndexStorage, Indexer};
use crate::search::{SearchConfig, SearchEngine, SearchResult, Searchable};
use std::any::TypeId;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::sync::Arc;

/// A table that stores entities in a BTreeMap.
/// It provides basic CRUD operations and supports fuzzy text search through a search engine.
pub struct Table<T: Entity> {
    /// For now, we use a BTreeMap for simplicity.
    entities: BTreeMap<Id<T>, Entry<T>>,
    search_engine: Option<SearchEngine<T>>,
    indices: HashMap<TypeId, IndexStorage<T>>,
}

impl<T: Entity> Table<T> {
    /// Inserts a new entity into the table, returning a reference to the entry.
    pub fn insert(&mut self, entity: T) -> Result<&Entry<T>, Error> {
        let entry = Entry {
            entity: Arc::new(entity),
        };

        let id = entry.get_id().clone();

        if self.entities.contains_key(&id) {
            return Err(Error::EntityAlreadyExists(
                id.value().to_string(),
                std::any::type_name::<T>(),
            ));
        }

        for storage in self.indices.values_mut() {
            storage.index(&entry);
        }

        self.entities.insert(id.clone(), entry);
        self.search_engine = None; // Reset search engine on insert

        Ok(self.entities.get(&id).unwrap())
    }

    /// Returns an iterator over all entries in the table.
    pub fn iter(&self) -> impl Iterator<Item = &Entry<T>> {
        self.entities.values()
    }

    /// Finds an entry in the table by its ID.
    pub fn find(&self, id: &Id<T>) -> Option<&Entry<T>> {
        self.entities.get(id)
    }

    /// Updates an existing entity in the table, returning a reference to the updated entry.
    pub fn update(&mut self, entity: T) -> Result<&Entry<T>, Error> {
        let id = entity.get_id().clone();
        let Some(existing_entry) = self.entities.get(&id) else {
            return Err(Error::EntityNotFound(
                id.value().to_string(),
                std::any::type_name::<T>(),
            ));
        };

        // Remove the old entry from indices
        for storage in self.indices.values_mut() {
            storage.forget(existing_entry);
        }

        let entry = Entry {
            entity: Arc::new(entity),
        };

        // Re-index the new entry
        for storage in self.indices.values_mut() {
            storage.index(&entry);
        }

        self.entities.insert(id.clone(), entry);
        self.search_engine = None; // Reset search engine on update

        Ok(self.entities.get(&id).unwrap())
    }

    /// Deletes an entity from the table by its ID.
    pub fn delete(&mut self, id: &Id<T>) -> Result<(), Error> {
        let Some(existing_entry) = self.entities.remove(id) else {
            return Err(Error::EntityNotFound(
                id.value().to_string(),
                std::any::type_name::<T>(),
            ));
        };

        // Remove the entry from all indices
        for storage in self.indices.values_mut() {
            storage.forget(&existing_entry);
        }

        self.search_engine = None; // Reset search engine on delete
        Ok(())
    }

    /// Adds an indexer to the table, allowing for indexed queries.
    pub fn add_index<I: Indexer<Entity = T> + 'static>(&mut self, indexer: I) {
        let type_id = TypeId::of::<I>();

        let storage = self
            .indices
            .entry(type_id)
            .or_insert_with(|| IndexStorage::new(indexer));

        for entry in self.entities.values() {
            storage.index(entry);
        }
    }

    /// Finds entries in the table by a specific index key.
    pub fn find_by_index<I: Indexer<Entity = T> + 'static>(&self, key: &str) -> Vec<&Entry<T>> {
        let type_id = TypeId::of::<I>();

        if let Some(storage) = self.indices.get(&type_id) {
            storage.get(key)
        } else {
            vec![]
        }
    }
}

impl<T: Entity + Searchable> Table<T> {
    /// Searches for entities in the table based on a query string (fuzzy text search).
    pub fn search(&mut self, query: &str) -> Vec<SearchResult<T>> {
        if self.search_engine.is_none() {
            // Init search engine if it isn't set up yet
            self.search_engine = Some(SearchEngine::new(
                self.entities.values().cloned().collect(),
                SearchConfig::default(),
            ));
        }

        self.search_engine.as_ref().unwrap().search(query)
    }
}

impl<T: Entity> Default for Table<T> {
    fn default() -> Self {
        Table {
            entities: BTreeMap::new(),
            search_engine: None,
            indices: HashMap::new(),
        }
    }
}

#[cfg(feature = "bincode")]
impl<T: Entity> bincode::Encode for Table<T>
where
    T: bincode::Encode,
{
    fn encode<__E: bincode::enc::Encoder>(
        &self,
        encoder: &mut __E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.entities, encoder)?;
        Ok(())
    }
}

#[cfg(feature = "bincode")]
impl<T: Entity, __Context> bincode::Decode<__Context> for Table<T>
where
    T: bincode::Decode<__Context>,
{
    fn decode<__D: bincode::de::Decoder<Context = __Context>>(
        decoder: &mut __D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            entities: bincode::Decode::decode(decoder)?,
            search_engine: None,
            indices: HashMap::new(),
        })
    }
}

#[cfg(feature = "bincode")]
impl<'__de, T: Entity, __Context> bincode::BorrowDecode<'__de, __Context> for Table<T>
where
    T: bincode::de::BorrowDecode<'__de, __Context>,
{
    fn borrow_decode<__D: bincode::de::BorrowDecoder<'__de, Context = __Context>>(
        decoder: &mut __D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            entities: bincode::BorrowDecode::<'_, __Context>::borrow_decode(decoder)?,
            search_engine: None,
            indices: HashMap::new(),
        })
    }
}

/// A read-only entry in a table, wrapping an entity.
/// This has a cheap clone, as it will only clone the Arc.
/// For mutability, you can call `into_owned` to get an owned version of the entity,
/// update it and call `update` on the table / database to persist changes.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entry<T> {
    entity: Arc<T>,
}

impl<T> Deref for Entry<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<T> Clone for Entry<T> {
    fn clone(&self) -> Self {
        Entry {
            entity: Arc::clone(&self.entity),
        }
    }
}

impl<T: Clone> Entry<T> {
    /// Clone the internal value, returning an owned version of the entity.
    pub fn into_owned(self) -> T {
        Arc::try_unwrap(self.entity).unwrap_or_else(|arc| (*arc).clone())
    }
}

#[cfg(feature = "bincode")]
impl<T> bincode::Encode for Entry<T>
where
    T: bincode::Encode,
{
    fn encode<__E: bincode::enc::Encoder>(
        &self,
        encoder: &mut __E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.entity, encoder)?;
        Ok(())
    }
}

#[cfg(feature = "bincode")]
impl<T, __Context> bincode::Decode<__Context> for Entry<T>
where
    T: bincode::Decode<__Context>,
{
    fn decode<__D: bincode::de::Decoder<Context = __Context>>(
        decoder: &mut __D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            entity: bincode::Decode::decode(decoder)?,
        })
    }
}

#[cfg(feature = "bincode")]
impl<'__de, T, __Context> bincode::BorrowDecode<'__de, __Context> for Entry<T>
where
    T: bincode::de::BorrowDecode<'__de, __Context>,
{
    fn borrow_decode<__D: bincode::de::BorrowDecoder<'__de, Context = __Context>>(
        decoder: &mut __D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            entity: bincode::BorrowDecode::<'_, __Context>::borrow_decode(decoder)?,
        })
    }
}
