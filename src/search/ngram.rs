use std::collections::HashMap;

/// Used to create an index to easily narrow down search results
pub struct NgramIndexer {
    /// The size of the n-grams to generate
    ngram_size: usize,
    /// The index mapping n-grams to entry IDs
    index: HashMap<String, Vec<usize>>,
    /// The current ID to assign to the next indexed entry
    current_id: usize,
}

impl NgramIndexer {
    pub fn new(ngram_size: usize) -> Self {
        NgramIndexer {
            ngram_size,
            index: HashMap::new(),
            current_id: 0,
        }
    }

    /// Indexes the input string by generating n-grams and storing them in the index.
    pub fn index(&mut self, input: &str) {
        let input = input.to_lowercase();
        let ngrams = self.generate_ngrams(&input);

        for ngram in ngrams {
            self.index.entry(ngram).or_default().push(self.current_id);
        }
    }

    /// Sets the current ID for the next indexed entry.
    pub(crate) fn set_current_id(&mut self, id: usize) {
        self.current_id = id;
    }

    /// Retrieves the IDs associated with a given n-gram.
    pub(crate) fn get(&self, ngram: &str) -> Option<&Vec<usize>> {
        self.index.get(ngram)
    }

    /// Generates n-grams from the input string.
    pub(crate) fn generate_ngrams(&self, input: &str) -> Vec<String> {
        let len = input.chars().count();

        if self.ngram_size == 0 || len < self.ngram_size {
            return Vec::new();
        }

        let mut ngrams = Vec::new();
        let indices = input.char_indices();

        for i in 0..=len - self.ngram_size {
            let start = indices.clone().nth(i).map(|(idx, _)| idx).unwrap_or(0);
            let end = indices
                .clone()
                .nth(i + self.ngram_size - 1)
                .map(|(idx, _)| idx)
                .unwrap_or(len);

            ngrams.push(input[start..=end].to_string());
        }

        ngrams
    }
}
