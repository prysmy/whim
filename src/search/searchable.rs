use crate::search::{BitapSearcher, NgramIndexer};
use std::rc::Rc;
use std::sync::Arc;

/// Trait for elements that can be indexed and searched.
/// Implemented by default on all [`Entity`] types created with the `#[entity]` macro.
///
/// [`Entity`]: crate::entities::Entity
pub trait Searchable {
    /// Indexes the item using the provided `NgramIndexer`.
    fn index(&self, indexer: &mut NgramIndexer);
    /// Retrieves the score for the item based on a search query using the provided `BitapSearcher`.
    fn get_score(&self, searcher: &BitapSearcher) -> Option<f32>;
}

impl Searchable for String {
    fn index(&self, indexer: &mut NgramIndexer) {
        indexer.index(self);
    }

    fn get_score(&self, searcher: &BitapSearcher) -> Option<f32> {
        searcher.get_score(self)
    }
}

impl<T: Searchable> Searchable for Vec<T> {
    fn index(&self, indexer: &mut NgramIndexer) {
        for item in self {
            item.index(indexer);
        }
    }

    fn get_score(&self, searcher: &BitapSearcher) -> Option<f32> {
        let list = self
            .iter()
            .filter_map(|item| item.get_score(searcher))
            .collect::<Vec<_>>();

        if list.is_empty() {
            None
        } else {
            Some(list.into_iter().fold(0f32, f32::max))
        }
    }
}

impl<T: Searchable> Searchable for Option<T> {
    fn index(&self, indexer: &mut NgramIndexer) {
        if let Some(item) = self {
            item.index(indexer);
        }
    }

    fn get_score(&self, searcher: &BitapSearcher) -> Option<f32> {
        self.as_ref().and_then(|item| item.get_score(searcher))
    }
}

impl<T: Searchable> Searchable for Rc<T> {
    fn index(&self, indexer: &mut NgramIndexer) {
        (**self).index(indexer);
    }

    fn get_score(&self, searcher: &BitapSearcher) -> Option<f32> {
        (**self).get_score(searcher)
    }
}

impl<T: Searchable> Searchable for Arc<T> {
    fn index(&self, indexer: &mut NgramIndexer) {
        (**self).index(indexer);
    }

    fn get_score(&self, searcher: &BitapSearcher) -> Option<f32> {
        (**self).get_score(searcher)
    }
}

impl<T: Searchable> Searchable for Box<T> {
    fn index(&self, indexer: &mut NgramIndexer) {
        (**self).index(indexer);
    }

    fn get_score(&self, searcher: &BitapSearcher) -> Option<f32> {
        (**self).get_score(searcher)
    }
}
