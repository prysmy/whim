//! This example demonstrates the use of indices in Whim.
//! It creates a `Note` entity with a custom index on the `created_at` field
//! and inserts two notes into a table.
//! We can then query the index via the table.

#![allow(dead_code)]

use whim::prelude::*;

#[derive(Entity)]
struct Note {
    #[id]
    id: Id<Self>,
    title: String,
    content: String,
    created_at: u64,
}

#[index(NoteCreatedAtIndex, Note)]
fn created_at_index(note: &Entry<Note>) -> u64 {
    note.created_at
}

fn main() {
    let mut table = Table::<Note>::default();

    table.add_index(NoteCreatedAtIndex);

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
            created_at: 1751007260,
        })
        .ok();

    let results = table.find_by_index::<NoteCreatedAtIndex>("1751007259");

    for entry in results {
        println!(
            "Found note: {} with created_at: {}",
            entry.title, entry.created_at
        );
    }
}
