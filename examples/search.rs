//! This example demonstrates the search functionality in Whim.
//! It creates a `Note` entity with searchable fields and performs a search with a typo,
//! to demonstrate the simple fuzzy search capabilities.

#![allow(dead_code)]

use whim::prelude::*;

#[derive(Entity, Searchable)]
struct Note {
    #[id]
    id: Id<Self>,
    #[search]
    title: String,
    #[search]
    content: String,
    created_at: u64,
}

fn main() {
    let mut table = Table::<Note>::default();

    table
        .insert(Note {
            id: Id::new("note1".to_string()),
            title: "First Note".to_string(),
            content: "This is the content of the first note.".to_string(),
            created_at: 1751007259,
        })
        .ok();

    table
        .insert(Note {
            id: Id::new("note2".to_string()),
            title: "Second Note".to_string(),
            content: "This is the content of the second note.".to_string(),
            created_at: 1751007259,
        })
        .ok();

    // Search with typo
    let results = table.search("Firdt");

    // Only the first note should be found
    for result in results {
        println!(
            "Found note: {} with score: {}",
            result.entry.title, result.score
        );
    }
}
