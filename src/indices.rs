use crate::prelude::Entity;
use crate::tables::Entry;
use std::collections::BTreeMap;
use std::fmt::Display;

pub trait Indexer {
    type Entity: Entity;

    fn get_indicies(&mut self, entity: &Entry<Self::Entity>) -> Vec<Index>;
}

pub struct IndexStorage<E: Entity> {
    indexer: Box<dyn Indexer<Entity = E>>,
    data: BTreeMap<Index, Vec<Entry<E>>>,
}

impl<E: Entity> IndexStorage<E> {
    pub fn new<I: Indexer<Entity = E> + 'static>(indexer: I) -> Self {
        IndexStorage {
            indexer: Box::new(indexer),
            data: BTreeMap::new(),
        }
    }

    pub fn index(&mut self, entity: &Entry<E>) {
        let keys = self.indexer.get_indicies(entity);

        for key in keys {
            self.data.entry(key).or_default().push(entity.clone());
        }
    }

    pub fn forget(&mut self, entity: &Entry<E>) {
        let keys = self.indexer.get_indicies(entity);

        for key in keys {
            let Some(entities) = self.data.get_mut(&key) else {
                continue;
            };

            let Some(pos) = entities.iter().position(|e| e.get_id() == entity.get_id()) else {
                continue;
            };

            entities.remove(pos);

            if entities.is_empty() {
                self.data.remove(&key);
            }
        }
    }

    pub fn get(&self, key: &str) -> Vec<&Entry<E>> {
        self.data
            .get(&key.into())
            .map(|entries| entries.iter().collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(String);

impl<T: Display> From<T> for Index {
    fn from(value: T) -> Self {
        Index(value.to_string())
    }
}
