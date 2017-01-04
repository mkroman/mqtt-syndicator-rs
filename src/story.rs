// Copyright (c) 2016, Mikkel Kroman <mk@uplink.io>
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// * Redistributions of source code must retain the above copyright notice, this
//   list of conditions and the following disclaimer.
//
// * Redistributions in binary form must reproduce the above copyright notice,
//   this list of conditions and the following disclaimer in the documentation
//   and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use ::rusqlite;

use super::error;

/// Story struct.
#[derive(Debug)]
pub struct Story {
    pub title: Option<String>,
    pub guid: Option<String>,
    pub description: Option<String>,
    pub pub_date: Option<String>,
    pub feed_url: String,
}

impl Story {
    /// Creates a new Story and inserts it into the `database`.
    /// 
    /// Returns the `Story` object if it was successfully added to the database, an error
    /// otherwise.
    pub fn create<DB>(database: DB, title: Option<String>, guid: Option<String>, 
                      pub_date: Option<String>, description: Option<String>, feed_url: String)
        -> Result<Story, error::DatabaseError>
        where DB: AsRef<rusqlite::Connection> {
        let story = Story {
            title: title,
            guid: guid,
            pub_date: pub_date,
            description: description,
            feed_url: feed_url,
        };

        let connection = database.as_ref();

        match connection.execute(
            "INSERT INTO stories (title, guid, pub_date, description, feed_url)
             VALUES (?, ?, ?, ?, ?)",
            &[&story.title, &story.guid, &story.pub_date, &story.description,
                &story.feed_url]) {
            Ok(_) => return Ok(story),
            Err(err) => return Err(err.into()),
        }
    }

    /// Finds a single story with the given `feed_url` and `guid`.
    pub fn find_by_feed_url_and_guid<DB, S1, S2>(database: DB, feed_url: S1, guid: S2)
        -> Option<Story>
        where DB: AsRef<rusqlite::Connection>, S1: AsRef<str>, S2: AsRef<str> {

        let connection = database.as_ref();

        match connection.query_row(
            "SELECT title, guid, pub_date, description, feed_url
             FROM stories
             WHERE (feed_url = ? AND guid = ?)
             LIMIT 1",
            &[&feed_url.as_ref(), &guid.as_ref()], |row| {
                Story {
                    title: row.get(0),
                    guid: row.get(1),
                    pub_date: row.get(2),
                    description: row.get(3),
                    feed_url: row.get(4)
                }
            }) {
            Ok(result) => return Some(result),
            Err(rusqlite::Error::QueryReturnedNoRows) => return None,
            Err(_) => return None,
        }
    }
}
