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
#![feature(proc_macro)]

#[macro_use]
extern crate log;
extern crate rss;
extern crate mio;
extern crate toml;
extern crate serde;
extern crate time;
extern crate hyper;
extern crate rusqlite;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;
extern crate atom_syndication;

use std::io::{Cursor, BufReader, Read};
use std::fs::File;
use std::path::Path;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};

use hyper::Client;
use hyper::header::{AcceptCharset, Charset, qitem};
use mio::{Poll, Events, PollOpt, Ready, Token};
use mio::timer::Timer;

pub mod error;
pub mod config;
mod story;
pub mod database;

pub use error::Error;
pub use config::{Config, Feed};
pub use story::Story;
pub use database::Database;

/// The main syndicator client.
pub struct Server {
    http_client: Client,
    config: Arc<Mutex<Config>>,
    database: Database,
    poll: Poll,
}

const TIMER: Token = Token(0);

lazy_static! {
    static ref POLL_DURATION: Duration = Duration::from_secs(1);
}

impl Server {
    /// Creates a new syndicator with a configuration file and database file.
    pub fn new<P: AsRef<Path>, P2: AsRef<Path>>(config_path: P, database_path: P2)
        -> Result<Server, Error> {
        let mut file = File::open(config_path)?;

        let config = Config::read_from(&mut file)?;
        let database = Database::open(database_path)?;

        database.init().unwrap();

        Ok(Server {
            http_client: Client::new(),
            database: database,
            config: Arc::new(Mutex::new(config)),
            poll: Poll::new()?
        })
    }

    /// Refreshes any pending feeds.
    fn refresh_feeds(&mut self) {
        trace!("Refreshing feeds");

        let now = Instant::now();
        let config = self.config.clone();
        let mut config = config.lock().unwrap();
        let mut feeds: Vec<&mut Feed> = config.feeds.iter_mut()
            .filter(|feed| feed.updated_at.is_none() ||
                    now - feed.updated_at.unwrap() > Duration::from_secs(feed.interval as u64))
            .collect();

        debug!("{} feeds need updating", feeds.len());

        for mut feed in feeds.iter_mut() {
            trace!("Refreshing feed {:?}", feed);

            let mut res = match self.http_client.get(&feed.url)
                .header(AcceptCharset(vec![qitem(Charset::Ext("utf-8".to_owned()))])).send() {
                Ok(res) => res,
                Err(err) => {
                    debug!("Error when requesting feed: {:?}", err);
                    continue;
                }
            };

            feed.updated_at = Some(Instant::now());

            let buf: String = {
                let mut data: Vec<u8> = Vec::new();

                match res.read_to_end(&mut data) {
                    Ok(_) => String::from_utf8_lossy(&data).into_owned(),
                    Err(err) => {
                        error!("Error reading http body: {:?}", err);
                        continue;
                    }
                }
            };

            match buf.as_str().parse::<atom_syndication::Feed>() {
                Ok(atom_feed) => {
                    match self.process_atom_feed(&feed.url, atom_feed) {
                        Ok(_) => {},
                        Err(err) => {
                            info!("Error when processing atom feed: {}, {:?}", err, err);
                        }
                    }
                },
                Err(err) => {
                    match rss::Channel::read_from(BufReader::new(Cursor::new(buf))) {
                        Ok(channel) => {
                            match self.process_rss_channel(&feed.url, channel) {
                                Ok(_) => {},
                                Err(err) => {
                                    info!("Error when processing rss channel: {}, {:?}", err, err);
                                }
                            }
                        },
                        Err(err) => {
                            debug!("Error processing rss feed: {:?}", err);
                            continue;
                        }
                    }
                }
            }

        }
    }

    /// Starts polling the feeds for news.
    /// 
    /// This function will block indefinitely.
    pub fn poll(&mut self) {
        let mut events = Events::with_capacity(1024);
        let mut timer = Timer::default();

        trace!("Starting polling");

        // Refresh feeds on startup.
        self.refresh_feeds();

        timer.set_timeout(*POLL_DURATION, "syndicate").unwrap();

        self.poll.register(&timer, TIMER, Ready::readable(), PollOpt::edge()).unwrap();

        loop {
            self.poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                match event.token() {
                    TIMER => {
                        self.refresh_feeds();
                        timer.set_timeout(*POLL_DURATION, "syndicate").unwrap();
                    },
                    _ => {}
                }
            }
        }
    }

    fn process_atom_feed(&mut self, feed_url: &str, feed: atom_syndication::Feed) -> Result<(), Error> {
        trace!("Processing atom feed for {}", feed_url);

        Ok(())
    }

    fn process_rss_channel(&mut self, feed_url: &str, channel: rss::Channel)
        -> Result<(), Error> {
        trace!("Processing RSS channel for feed {}", feed_url);

        for item in channel.items {
            let guid = item.guid.map(|guid| guid.value);

            if let Some(story) = Story::find_by_feed_url_and_guid(&self.database, &feed_url,
                                                                  &guid.as_ref().unwrap()) {
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

