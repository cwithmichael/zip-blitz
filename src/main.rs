mod file_type;
use file_type::FileType;
use std::io::prelude::*;
use std::io::{self, Read};
use std::process;

fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <filename>", args[0]);
        ()
    }
    let zip_name = std::path::Path::new(&*args[1]);
    let file_name = &*args[2];
    let file_type_str = &*args[3];
    let file_type = determine_file_type(file_type_str).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1)
    });
    let zipfile = std::fs::File::open(&zip_name).unwrap();

    let mut archive = zip::ZipArchive::new(zipfile).unwrap();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(password) => {
                if let Ok(file) = archive.by_name_decrypt(file_name, password.as_bytes()) {
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
                } else {
                    println!("Couldn't find file in zip");
                    break;
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
