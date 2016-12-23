#[macro_use]
extern crate log;
extern crate clap;
extern crate app_dirs;
extern crate env_logger;
extern crate syndicator;

use clap::{App, Arg};
use app_dirs::{AppInfo, AppDataType, get_app_dir};

use syndicator::error::{Error, ConfigError};
use syndicator::Syndicator;

const APP_INFO: AppInfo = AppInfo { name: "syndicator", author: "Mikkel Kroman <mk@maero.dk>" };

fn main() {
    env_logger::init().unwrap();

    let default_database_path = get_app_dir(AppDataType::UserData, &APP_INFO, "database.db").unwrap();

    let matches = App::new("syndicator")
        .version("1.0")
        .author("Mikkel Kroman <mk@maero.dk>")
        .arg(Arg::with_name("database")
             .short("d")
             .long("database")
             .value_name("FILE")
             .help("Sets the database file")
             .default_value(default_database_path.to_str().unwrap()))
        .get_matches();

    let database_path = matches.value_of("database").unwrap();

    let syndicator = match Syndicator::new("config.toml", database_path) {
        Ok(syndicator) => syndicator,
        Err(err) => panic!("Error when starting syndicator: {}", err),
    };

    syndicator.poll();
}
