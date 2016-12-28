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

#[macro_use]
extern crate log;
extern crate hyper;
extern crate toml;
extern crate rss;
extern crate time;
extern crate rusqlite;
#[macro_use]
extern crate quick_error;
extern crate rustc_serialize;

use std::io::{Read, BufReader};
use std::fs::File;
use std::thread;
use std::path::Path;
use std::time::Duration;

use hyper::Client;
use toml::{Parser, Value};
use rustc_serialize::{Decodable};

pub mod error;
mod database;
mod story;

pub use error::{Error, ConfigError};
pub use story::Story;
pub use database::Database;

#[derive(Debug, RustcDecodable)]
pub struct Feed {
    pub name: String,
    pub url: String,
    pub category: String,
}

/// The main syndicator client.
pub struct Syndicator {
    http_client: Client,
    config: Config,
    database: Database,
}

pub struct Config {
    pub feeds: Vec<Feed>
}

impl Config {
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Config, ConfigError> {
        let mut s = String::new();
        reader.read_to_string(&mut s)?;

        let mut parser = Parser::new(&s);
        let table = match parser.parse() {
            Some(config) => Value::Table(config),
            None => return Err(ConfigError::TomlParser(parser.errors)),
        };

        let subscriptions = table.lookup("feeds").ok_or(ConfigError::NoFeeds)?;
        let mut feeds: Vec<Feed> = Vec::new();

        for entry in subscriptions.as_slice().unwrap() {
            let feed: Feed = Decodable::decode(&mut toml::Decoder::new(entry.clone())).unwrap();
            feeds.push(feed);
        }

        Ok(Config {
            feeds: feeds
        })
    }
}

impl Syndicator {
    /// Creates a new syndicator with a configuration file and database file.
    pub fn new<P: AsRef<Path>, P2: AsRef<Path>>(config_path: P, database_path: P2) -> Result<Syndicator, Error> {
        let mut file = File::open(config_path)?;

        let config = Config::read_from(&mut file)?;
        let database = Database::open(database_path)?;

        database.init().unwrap();

        Ok(Syndicator {
            http_client: Client::new(),
            database: database,
            config: config,
        })
    }

    /// Starts polling the feeds for news.
    /// 
    /// This function will block indefinitely.
    pub fn poll(&self) {
        loop {
            for feed in &self.config.feeds {
                let res = match self.http_client.get(&feed.url).send() {
                    Ok(res) => res,
                    Err(err) => { 
                        debug!("Error when requesting feed: {:?}", err);
                        continue;
                    }
                };

                let channel = match rss::Channel::read_from(BufReader::new(res)) {
                    Ok(channel) => channel,
                    Err(err) => {
                        debug!("Error processing rss feed: {:?}", err);
                        continue;
                    }
                };

                match self.process_rss_channel(&feed.url, channel) {
                    Ok(_) => {},
                    Err(err) => {
                        info!("Error when processing rss channel: {}, {:?}", err, err);
                    }
                }
            }

            thread::sleep(Duration::from_secs(60));
        }
    }

    fn process_rss_channel(&self, feed_url: &str, channel: rss::Channel) -> Result<(), Error> {
        for item in channel.items {
            let guid = item.guid.map(|guid| guid.value);

            if let Some(story) = Story::find_by_feed_url_and_guid(&self.database, &feed_url, &guid.as_ref().unwrap()) {
                //debug!("Found story: {:?}", story);

                // Skip to the next item.
                continue;
            } else {
                Story::create(&self.database, item.title, guid, item.pub_date, item.description,
                              feed_url.to_string()).unwrap();
                // Insert the story into the database.
                println!("New news story found!");
            }
        }

        Ok(())
    }
}

