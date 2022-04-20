mod file_type;
use clap::StructOpt;
use std::{io::BufRead, process};
use zip_blitz::{Args, Config};

fn main() {
    let args = Args::parse();
    let config = Config::new(args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    match zip_blitz::run(config, std::io::stdin().lock().lines().flatten()) {
        Ok(password) => println!("Found it: {}", password),
        Err(e) => {
            println!("Failure: {}", e);
            process::exit(1);
        }
    }
}
