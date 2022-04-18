mod file_type;
use std::io::stdin;
use std::process;
use zip_blitz::{Args, Config};

fn main() {
    let args = <Args as clap::StructOpt>::parse();
    let config = Config::new(args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    let stdin = stdin();
    if let Err(e) = zip_blitz::run(config, stdin) {
        println!("Failure: {}", e);
        process::exit(1);
    }
}
