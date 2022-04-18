mod file_type;
use std::io::{self, prelude::*};
use std::process;
use zip_blitz::{Args, Config};

fn main() -> io::Result<()> {
    let args = <Args as clap::StructOpt>::parse();
    let mut config = Config::new(args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(password) => {
                if let Ok(file) = config
                    .archive
                    .by_name_decrypt(&config.file_name, password.as_bytes())
                {
                    match file {
                        Ok(f) => {
                            let data: Vec<u8> =
                                f.bytes().take(12).map(|d| d.unwrap_or(0)).collect();
                            if zip_blitz::is_header_valid(&data, &config.file_type) {
                                println!("Found it: {}", password);
                                break;
                            }
                        }
                        Err(_) => (),
                    }
                }
            }
            Err(_) => break,
        }
    }
    Ok(())
}
