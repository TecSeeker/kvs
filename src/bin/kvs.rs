use clap::{App, Arg, SubCommand};
use kvs::{KvStore, Result};
use std::process::exit;

fn main() -> Result<()> {
    let matches = App::new("kvs")
        .version(env!("CARGO_PKG_VERSION"))
        .author("TecSeeker <fakeluziyan@gmail.com>")
        .about("A simple key-value store")
        .subcommand(
            SubCommand::with_name("set")
                .about("Sets the value of a string key to a string")
                .arg(Arg::with_name("KEY").required(true))
                .arg(Arg::with_name("VALUE").required(true)),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Gets the string value of a given string key")
                .arg(Arg::with_name("KEY").required(true)),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Removes a given key")
                .arg(Arg::with_name("KEY").required(true)),
        )
        .get_matches();

    let mut store = KvStore::open(std::env::current_dir()?)?;

    match matches.subcommand() {
        Some(("set", matches)) => {
            let key = matches.value_of("KEY").unwrap().to_string();
            let value = matches.value_of("VALUE").unwrap().to_string();
            store.set(key, value)?;
        }
        Some(("get", matches)) => {
            let key = matches.value_of("KEY").unwrap().to_string();
            match store.get(key)? {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }
        }
        Some(("rm", matches)) => {
            let key = matches.value_of("KEY").unwrap().to_string();
            if let Err(e) = store.remove(key) {
                println!("{}", e);
                exit(1);
            }
        }
        _ => {
            eprintln!("No command provided");
            exit(1);
        }
    }

    Ok(())
}
