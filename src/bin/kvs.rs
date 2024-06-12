use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("kvs")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Your Name <you@example.com>")
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

    match matches.subcommand() {
        Some(("set", _)) => {
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        Some(("get", _)) => {
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        Some(("rm", _)) => {
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        None => {
            eprintln!("No command provided");
            std::process::exit(1);
        }
        _ => unreachable!(), // 捕获所有其他未定义子命令
    }
}
