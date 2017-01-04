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

use std::io::{self, Read};

use toml::{Decoder, Parser, Value, ParserError};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: io::Error) {
            from()
        }

        TomlParser(errors: Vec<ParserError>) {
            description("config parse error")
        }

        NoFeeds {
            description("no `feeds` key defined in config")
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feed {
    pub name: String,
    pub url: String,
    pub category: String,
}

pub struct Config {
    pub feeds: Vec<Feed>
}

impl Config {
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Config, Error> {
        let mut s = String::new();
        reader.read_to_string(&mut s)?;

        let mut parser = Parser::new(&s);
        let table = match parser.parse() {
            Some(config) => Value::Table(config),
            None => return Err(Error::TomlParser(parser.errors)),
        };

        let subscriptions = table.lookup("feeds").ok_or(Error::NoFeeds)?;
        let mut feeds: Vec<Feed> = Vec::new();

        for entry in subscriptions.as_slice().unwrap() {
            use serde::Deserialize;
            let mut decoder = Decoder::new(entry.clone());
            let feed: Feed = Deserialize::deserialize(&mut decoder).unwrap();
            feeds.push(feed);
        }

        Ok(Config {
            feeds: feeds
        })
    }
}
