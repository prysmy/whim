use crate::Error;
use crate::entities::Entity;
use crate::ids::Id;
use crate::indices::Indexer;
use crate::search::{SearchConfig, SearchEngine, SearchResult, Searchable};
use std::any::TypeId;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::sync::{Arc, Mutex};

/// A table that stores entities in a BTreeMap.
/// It provides basic CRUD operations and supports fuzzy text search through a search engine.
pub struct Table<T: Entity> {
    /// For now, we use a BTreeMap for simplicity.
    entities: BTreeMap<Id<T>, Entry<T>>,
    search_engine: Arc<Mutex<Option<SearchEngine<T>>>>,
    indices: HashMap<TypeId, Box<dyn Indexer<Entity = T> + Send + Sync>>,
}

impl<T: Entity + 'static> Table<T> {
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

        for index in self.indices.values_mut() {
            index.index(&entry);
        }

        self.entities.insert(id.clone(), entry);

        // Reset search engine on insert
        self.search_engine = Arc::new(Mutex::new(None));

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
        for index in self.indices.values_mut() {
            index.forget(existing_entry);
        }

        let entry = Entry {
            entity: Arc::new(entity),
        };

        // Re-index the new entry
        for index in self.indices.values_mut() {
            index.index(&entry);
        }

        self.entities.insert(id.clone(), entry);

        // Reset search engine on update
        self.search_engine = Arc::new(Mutex::new(None));

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
        for index in self.indices.values_mut() {
            index.forget(&existing_entry);
        }

        // Reset search engine on delete
        self.search_engine = Arc::new(Mutex::new(None));
        Ok(())
    }

    /// Adds an indexer to the table, allowing for indexed queries.
    pub fn add_index<I: Indexer<Entity = T> + Send + Sync + 'static>(&mut self, mut indexer: I) {
        let type_id = TypeId::of::<I>();

        if self.indices.contains_key(&type_id) {
            // If the indexer already exists, we can skip adding it again
            return;
        }

        for entry in self.entities.values() {
            indexer.index(entry);
        }

        self.indices.insert(type_id, Box::new(indexer));
    }

    /// Finds entries in the table by a specific index key.
    pub fn get_index<I: Indexer<Entity = T> + 'static>(&self) -> Option<&I> {
        let type_id = TypeId::of::<I>();

        if let Some(index) = self.indices.get(&type_id) {
            // Downcast index from &Box<dyn Indexer<Entity = T>> to &I
            if let Some(index) = index.as_any().downcast_ref::<I>() {
                return Some(index);
            }
        }

        None
    }
}

impl<T: Entity + Searchable> Table<T> {
    /// Searches for entities in the table based on a query string (fuzzy text search).
    pub fn search(&self, query: &str) -> Vec<SearchResult<T>> {
        let Ok(mut engine) = self.search_engine.lock() else {
            // If the lock is poisoned, we return an empty search result
            return Vec::new();
        };

        if engine.is_none() {
            // If the search engine is not initialized, create a new one
            *engine = Some(SearchEngine::new(
                self.entities.values().cloned().collect(),
                SearchConfig::default(),
            ));
        }

        engine.as_ref().unwrap().search(query)
    }
}

impl<T: Entity> Default for Table<T> {
    fn default() -> Self {
        Table {
            entities: BTreeMap::new(),
            search_engine: Arc::new(Mutex::new(None)),
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
            search_engine: Arc::new(Mutex::new(None)),
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
            search_engine: Arc::new(Mutex::new(None)),
            indices: HashMap::new(),
        })
    }
}

/// A read-only entry in a table, wrapping an entity.
/// This has a cheap clone, as it will only clone the Arc.
/// For mutability, you can call `into_owned` to get an owned version of the entity,
/// update it and call `update` on the table / database to persist changes.
#[derive(Debug, PartialOrd, Ord)]
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

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.entity, &other.entity)
    }
}

impl<T> Eq for Entry<T> {}

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
