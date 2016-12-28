use std::path::Path;

use ::time::{self, Timespec};
use ::rusqlite::{self, Connection};

use super::Story;
use super::error::DatabaseError;

pub type Error = DatabaseError;

static INIT_SQL: &'static str = include_str!("../init.sql");

/// Database struct.
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Tries to open the database at `database_path`.
    pub fn open<P: AsRef<Path>>(database_path: P) -> Result<Database, Error> {
        let connection = Connection::open(database_path)?;

        Ok(Database {
            connection: connection,
        })
    }

    /// Initialises the database with tables and indices.
    pub fn init(&self)  -> Result<(), Error> { 
        self.connection.execute_batch(INIT_SQL)?;

        Ok(())
    }

    /// Finds the story with the given feed_url and guid.
    pub fn find_story(&self, feed_url: &str, guid: &str) -> Option<Story> {
        match self.connection.query_row(
            "SELECT title, guid, content, pub_date, description, feed_url
            FROM stories
            WHERE (`guid` = ? AND `feed_url` = ?)
            LIMIT 1",
            &[&guid, &feed_url], |row| {
                Story {
                    title: row.get(0),
                    guid: row.get(1),
                    content: row.get(2),
                    pub_date: row.get(3),
                    description: row.get(4),
                    feed_url: row.get(5)
                }
            }) {
            Ok(result) => return Some(result),
            Err(rusqlite::Error::QueryReturnedNoRows) => return None,
            Err(_) => return None,
        }
    }

    /// Inserts the story into the database.
    pub fn insert_story(&self, story: Story) -> Result<(), Error> {
        match self.connection.execute(
            "INSERT INTO stories (title, guid, content, pub_date, description, feed_url)
            VALUES (?, ?, ?, ?, ?, ?)",
            &[&story.guid, &time::get_time(), &story.feed_url]) {
            Ok(_) => return Ok(()),
            Err(err) => return Err(err.into()),
        }
    }
}
