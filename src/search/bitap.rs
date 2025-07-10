/// We use the Bitap algorithm for fuzzy searching.
#[derive(Debug)]
pub struct BitapSearcher<'a> {
    /// The pattern to search for, in lowercase.
    pub(crate) pattern: &'a str,
    /// A mask for each character in the pattern, used to track mismatches.
    pub(crate) pattern_mask: [u32; 1024],
    /// The maximum number of mismatches allowed for a match to be considered valid.
    pub(crate) max_mismatches: usize,
}

impl<'a> BitapSearcher<'a> {
    /// Calculates a score for the given text based on the pattern.
    /// Returns None if every segment tested has more mismatches than allowed.
    pub fn get_score(&self, text: &str) -> Option<f32> {
        let text = text.to_lowercase();

        let text_len = text.chars().count();
        let pattern_len = self.pattern.chars().count();
        let indices = text.char_indices();

        for i in 0..=text_len.saturating_sub(pattern_len) {
            let mut mismatches = 0;
            let mut r = 0;

            let end = (i + pattern_len).min(text_len);

            let start = indices.clone().nth(i).map(|(idx, _)| idx).unwrap_or(0);
            let end = indices
                .clone()
                .nth(end)
                .map(|(idx, _)| idx)
                .unwrap_or(text.len());

            for (j, character) in text[start..end].chars().enumerate() {
                r = ((r << 1) | 1) & self.pattern_mask[character as usize];

                if r & (1 << j) == 0 {
                    mismatches += 1;
                }
            }

            if mismatches <= self.max_mismatches {
                let score = 1.0 - mismatches as f32 / pattern_len as f32;
                return Some(score);
            }
        }

        None
    }
}
