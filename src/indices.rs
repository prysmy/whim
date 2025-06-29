use crate::prelude::Entity;
use crate::tables::Entry;
use std::collections::BTreeMap;

pub trait Indexer {
    type Entity: Entity;

    fn get_index(&mut self, entity: &Entry<Self::Entity>) -> String;
}

pub struct IndexStorage<E: Entity> {
    indexer: Box<dyn Indexer<Entity = E>>,
    data: BTreeMap<String, Vec<Entry<E>>>,
}

impl<E: Entity> IndexStorage<E> {
    pub fn new<I: Indexer<Entity = E> + 'static>(indexer: I) -> Self {
        IndexStorage {
            indexer: Box::new(indexer),
            data: BTreeMap::new(),
        }
    }

    pub fn index(&mut self, entity: &Entry<E>) {
        let key = self.indexer.get_index(entity);

        self.data.entry(key).or_default().push(entity.clone());
    }

    pub fn forget(&mut self, entity: &Entry<E>) {
        let key = self.indexer.get_index(entity);

        let Some(entities) = self.data.get_mut(&key) else {
            return;
        };

        let Some(pos) = entities.iter().position(|e| e.get_id() == entity.get_id()) else {
            return;
        };

        entities.remove(pos);

        if entities.is_empty() {
            self.data.remove(&key);
        }
    }

    pub fn get(&self, key: &str) -> Vec<&Entry<E>> {
        self.data
            .get(key)
            .map(|entries| entries.iter().collect())
            .unwrap_or_default()
    }
}
