use crate::prelude::Entity;
use crate::tables::Entry;
use std::any::Any;
use std::collections::BTreeMap;

pub trait Indexer: Any {
    type Entity: Entity;

    fn index(&mut self, entity: &Entry<Self::Entity>);
    fn forget(&mut self, entity: &Entry<Self::Entity>);
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct IndexStorage<K: Ord, E> {
    data: BTreeMap<K, Vec<Entry<E>>>,
}

impl<K: Ord, E> Default for IndexStorage<K, E> {
    fn default() -> Self {
        IndexStorage {
            data: BTreeMap::new(),
        }
    }
}

impl<K: Ord, E> IndexStorage<K, E> {
    pub fn push(&mut self, keys: Vec<K>, entity: &Entry<E>) {
        for key in keys {
            self.data.entry(key).or_default().push(entity.clone());
        }
    }

    pub fn forget(&mut self, keys: Vec<K>, entity: &Entry<E>)
    where
        E: Entity,
    {
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

    pub fn get(&self, key: &K) -> Vec<&Entry<E>> {
        self.data
            .get(key)
            .map(|entries| entries.iter().collect())
            .unwrap_or_default()
    }
}
