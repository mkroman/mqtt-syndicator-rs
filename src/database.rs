use std::path::Path;

use ::rusqlite::{self, Connection};

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
}

impl AsRef<rusqlite::Connection> for Database {
    fn as_ref(&self) -> &rusqlite::Connection {
        &self.connection
    }
}
