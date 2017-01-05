#[macro_use]
extern crate log;
extern crate clap;
extern crate app_dirs;
extern crate env_logger;
extern crate syndicator;

use clap::{App, Arg};
use app_dirs::{AppInfo, AppDataType, get_app_dir};

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
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets the config file")
             .default_value("config.toml"))
        .get_matches();

    let database_path = matches.value_of("database").unwrap();
    let config_path = matches.value_of("config").unwrap();

    let mut syndicator = syndicator::Server::new(config_path, database_path).unwrap();

    syndicator.poll();
}
