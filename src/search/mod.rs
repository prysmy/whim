use crate::tables::Entry;
pub use bitap::BitapSearcher;
pub use ngram::NgramIndexer;
pub use searchable::Searchable;
use std::collections::HashMap;

pub mod bitap;
pub mod ngram;
pub mod searchable;

/// Configuration for the search engine.
pub struct SearchConfig {
    pub ngram_size: usize,
    pub max_distance: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            ngram_size: 3,
            max_distance: 2,
        }
    }
}

/// Represents a search result containing an entry and its score.
#[derive(Debug, Clone)]
pub struct SearchResult<T> {
    pub entry: Entry<T>,
    pub score: f32,
}

impl<T> SearchResult<T> {
    pub fn new(entry: Entry<T>, score: f32) -> SearchResult<T> {
        SearchResult { entry, score }
    }
}

/// A search engine that allows for fuzzy searching of entries.
pub struct SearchEngine<T> {
    entries: Vec<Entry<T>>,
    max_bitap_mismatches: usize,
    indexer: NgramIndexer,
}

impl<T> Default for SearchEngine<T> {
    fn default() -> Self {
        SearchEngine {
            entries: Vec::new(),
            max_bitap_mismatches: 2,
            indexer: NgramIndexer::new(3),
        }
    }
}

impl<T: Searchable> SearchEngine<T> {
    /// Creates a new search engine with the provided data and configuration.
    pub fn new(data: Vec<Entry<T>>, config: SearchConfig) -> Self {
        let mut engine = SearchEngine {
            indexer: NgramIndexer::new(config.ngram_size),
            max_bitap_mismatches: config.max_distance,
            entries: data,
        };

        for (id, entry) in engine.entries.iter().enumerate() {
            engine.indexer.set_current_id(id);

            entry.index(&mut engine.indexer);
        }

        engine
    }

    /// Adds new entries to the search engine, indexing them for searching.
    pub fn add_entries(&mut self, entries: Vec<Entry<T>>) {
        let mut id = self.entries.len();

        for entry in &entries {
            self.indexer.set_current_id(id);
            entry.index(&mut self.indexer);
            id += 1;
        }

        self.entries.extend(entries);
    }

    /// Searches for entries matching the given query string.
    pub fn search(&self, query: &str) -> Vec<SearchResult<T>> {
        // Supports only queries between 1 and 32 characters.
        if query.is_empty() || query.len() > u32::BITS as usize {
            return Vec::new();
        }

        let query = query.to_lowercase();

        let mut pattern_mask = [0u32; 1024];

        for (i, ch) in query.chars().enumerate() {
            pattern_mask[ch as usize] |= 1 << i;
        }

        let ngrams = self.indexer.generate_ngrams(&query);
        let mut candidates = HashMap::new();

        for ngram in ngrams {
            if let Some(docs) = self.indexer.get(&ngram) {
                for &id in docs {
                    *candidates.entry(id).or_insert(0) += 1;
                }
            }
        }

        let searcher = BitapSearcher {
            pattern: &query,
            pattern_mask,
            max_mismatches: self.max_bitap_mismatches,
        };

        let mut results = candidates
            .into_iter()
            .filter_map(|(id, _)| {
                let entry = &self.entries[id];
                let score = entry.get_score(&searcher)?;
                Some(SearchResult::new(entry.clone(), score))
            })
            .collect::<Vec<_>>();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }
}
