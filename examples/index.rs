//! This example demonstrates the use of indices in Whim.
//! It creates a `Note` entity and inserts a few notes into a table.
//! We also define the following indices :
//! - `NoteCreatedAtIndex` which indexes notes by their creation time.
//! - `NoteCreatedByIndex` which indexes notes by the user who created them.
//! - `NoteTitleWordsIndex` which indexes notes by the words in their title.

#![allow(dead_code)]

use whim::prelude::*;

#[derive(Entity)]
struct Note {
    #[id]
    id: Id<Self>,
    title: String,
    content: String,
    created_at: u64,
    created_by: Option<String>,
}

#[index(u64 -> Note)]
fn NoteCreatedAtIndex(note: &Entry<Note>) -> u64 {
    note.created_at
}

/// Indices can also return `Option<T>` types, where `T` is the type of the index key.
/// If the index key is `None`, the entry will not be indexed.
#[index(String -> Note)]
fn NoteCreatedByIndex(note: &Entry<Note>) -> Option<String> {
    note.created_by.clone()
}

/// Finally, indices can also return `Vec<T>` types, where `T` is the type of the index key.
/// This allows for multiple keys to be associated with a single entry.
#[index(String -> Note)]
fn NoteTitleWordsIndex(note: &Entry<Note>) -> Vec<String> {
    note.title
        .split_whitespace()
        .map(|word| word.to_string())
        .collect()
}

fn main() {
    let mut table = Table::<Note>::default();

    table.add_index(NoteCreatedAtIndex::default());
    table.add_index(NoteCreatedByIndex::default());
    table.add_index(NoteTitleWordsIndex::default());

    table
        .insert(Note {
            id: Id::new("note1".to_string()),
            title: "First Note".to_string(),
            content: "This is the content of the first note.".to_string(),
            created_at: 1751007259,
            created_by: None,
        })
        .ok();

    table
        .insert(Note {
            id: Id::new("note2".to_string()),
            title: "Second Note".to_string(),
            content: "This is the content of the second note.".to_string(),
            created_at: 1751007260,
            created_by: Option::Some("user1".to_string()),
        })
        .ok();

    table
        .insert(Note {
            id: Id::new("note3".to_string()),
            title: "Third Note".to_string(),
            content: "This is the content of the third note.".to_string(),
            created_at: 1751007261,
            created_by: Option::Some("user1".to_string()),
        })
        .ok();

    let results = table
        .get_index::<NoteCreatedAtIndex>()
        .unwrap()
        .find(&1751007259);

    // Should print the first note
    for entry in results {
        println!(
            "Found note: {} with created_at: {}",
            entry.title, entry.created_at
        );
    }

    // Should print the second and third notes
    let results = table
        .get_index::<NoteCreatedByIndex>()
        .unwrap()
        .find(&"user1".to_string());

    for entry in results {
        println!(
            "Found note: {} created by: {:?}",
            entry.title, entry.created_by
        );
    }

    // Should print every note with the word "Note" in the title
    let results = table
        .get_index::<NoteTitleWordsIndex>()
        .unwrap()
        .find(&"Note".to_string());

    for entry in results {
        println!("Found note: {} with title containing 'Note'", entry.title);
    }
}
