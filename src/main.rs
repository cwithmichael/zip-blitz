mod file_type;
use clap::Parser;
use file_type::FileType;
use std::io::prelude::*;
use std::io::{self, Read};
use std::process;
use zip::result::ZipError;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the zip file
    #[clap(short, long)]
    zip_name: String,

    /// Name of the file to test extraction with
    #[clap(short, long)]
    file_name: String,

    /// File type
    #[clap(short = 't', long)]
    file_type: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let zip_path = std::path::Path::new(&args.zip_name);
    let file_type = determine_file_type(&args.file_type).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1)
    });
    let zipfile = std::fs::File::open(&zip_path).unwrap_or_else(|err| {
        println!("Problem opening zip file: {}", err);
        process::exit(1)
    });
    let mut archive = zip::ZipArchive::new(zipfile).unwrap_or_else(|err| {
        println!("Problem reading zip file: {}", err);
        process::exit(1)
    });
    match archive.by_name_decrypt(&args.file_name, "".as_bytes()) {
        Ok(_) => (),
        Err(ref e) => {
            if e.to_string() == ZipError::FileNotFound.to_string() {
                println!("File: {} doesn't exist in zip", &args.file_name);
                process::exit(1)
            }
        }
    }

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(password) => {
                if let Ok(file) = archive.by_name_decrypt(&args.file_name, password.as_bytes()) {
                    match file {
                        Ok(f) => {
                            let data: Vec<u8> =
                                f.bytes().take(12).map(|d| d.unwrap_or(0)).collect();
                            //println!("{:?}", data);
                            if is_header_valid(&data, &file_type) {
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

fn is_header_valid(data: &[u8], file_type: &file_type::FileTypeKind) -> bool {
    match file_type {
        file_type::FileTypeKind::Wmv(wmv) => wmv.is_valid_header(data),
        file_type::FileTypeKind::Jpg(jpg) => jpg.is_valid_header(data),
        file_type::FileTypeKind::Archive(archive) => archive.is_valid_header(data),
    }
}

fn determine_file_type(file_type: &str) -> Result<file_type::FileTypeKind, &'static str> {
    match file_type.to_ascii_lowercase().as_str() {
        "asf" | "wma" | "wmv" => Ok(file_type::FileTypeKind::Wmv(file_type::Wmv::new())),
        "jpg" => Ok(file_type::FileTypeKind::Jpg(file_type::Jpg::new())),
        "zip" | "apk" | "jar" => Ok(file_type::FileTypeKind::Archive(file_type::Archive::new())),
        _ => Err("Unknown file type"),
    }
}
