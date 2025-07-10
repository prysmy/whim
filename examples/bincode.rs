//! Run this example with `cargo run --example bincode --features bincode`.
//! This example will try to load a database from a file named `db.bin`, or create a new one if it doesn't exist.
//! It will then push a new note and user into the database and save it back to the file.
//! Note: currently indices are not saved, so you need to re-add them after loading the database.

#![allow(dead_code)]

use bincode::{Decode, Encode};
use std::time::{SystemTime, UNIX_EPOCH};
use whim::prelude::*;

#[derive(Entity, Encode, Decode)]
struct Note {
    #[id]
    id: Id<Self>,
    title: String,
    content: String,
    created_at: u64,
    created_by: Id<User>,
}

#[derive(Entity, Encode, Decode)]
struct User {
    #[id]
    id: Id<Self>,
    name: String,
}

#[derive(Default, Encode, Decode)]
struct Database {
    notes: Table<Note>,
    users: Table<User>,
}

fn main() {
    // Used for unique IDs
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let mut db = load_database();

    let user = User {
        id: Id::new(time.as_millis().to_string()),
        name: "John".to_string(),
    };

    let user = db.users.insert(user).expect("Failed to insert user");

    let note = Note {
        id: Id::new(time.as_millis().to_string()),
        title: "Test Note".to_string(),
        content: "This is the content of the test note.".to_string(),
        created_at: time.as_secs(),
        created_by: user.get_id().clone(),
    };

    db.notes.insert(note).expect("Failed to insert note");

    // Print all the notes
    for entry in db.notes.iter() {
        println!(
            "Note ID: {}, Title: {}, Created At: {}, Created By: {}",
            entry.id, entry.title, entry.created_at, entry.created_by,
        );
    }

    save_database(&db);
}

fn load_database() -> Database {
    match std::fs::read("db.bin") {
        Ok(data) => match bincode::decode_from_slice(&data, bincode::config::standard()) {
            Ok((db, _)) => db,
            Err(e) => {
                eprintln!("Failed to decode database: {e}");
                Database::default()
            }
        },
        Err(_) => Database::default(),
    }
}

fn save_database(db: &Database) {
    let encoded =
        bincode::encode_to_vec(db, bincode::config::standard()).expect("Failed to encode database");

    std::fs::write("db.bin", encoded).expect("Failed to write database to file");
}
