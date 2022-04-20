mod file_type;
use clap::Parser;
use file_type::{Archive, Jpg, Wmv};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use zip::read::ZipFile;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Name of the zip file
    #[clap(short, long)]
    zip_name: String,

    /// Name of the file to test extraction with
    #[clap(short, long)]
    file_name: String,

    /// File type
    #[clap(short = 't', long)]
    file_type: Option<String>,
}

pub struct Config {
    pub archive: zip::ZipArchive<File>,
    pub file_name: String,
    pub file_type: FileTypeKind,
}

#[derive(PartialEq, Eq, Debug)]
pub enum FileTypeKind {
    Wmv(Wmv),
    Jpg(Jpg),
    Archive(Archive),
}

impl Config {
    pub fn new(args: Args) -> Result<Config, Box<dyn Error>> {
        let zip_path = std::path::Path::new(&args.zip_name);
        let zip_file = std::fs::File::open(&zip_path)?;
        let file_type = match &args.file_type {
            Some(ft) => parse_file_type(&ft)?,
            None => guess_file_type(&args.file_name)?,
        };
        let mut archive = zip::ZipArchive::new(zip_file)?;
        check_if_file_exists_in_zip(&mut archive, &args.file_name)?;
        Ok(Config {
            archive,
            file_name: args.file_name,
            file_type: file_type.into(),
        })
    }
}

pub fn run<R>(mut config: Config, mut passwords: R) -> Result<String, &'static str>
where
    R: Iterator<Item = String>,
{
    passwords
        .find(|p| {
            config
                .archive
                .by_name_decrypt(&config.file_name, p.as_bytes())
                .ok()
                .and_then(|r| r.ok())
                .map_or(false, |mut file| {
                    is_header_valid(&mut file, &config.file_type)
                })
        })
        .ok_or("Password wasn't found")
}

fn is_header_valid(file: &mut ZipFile, file_type: &FileTypeKind) -> bool {
    let mut header = [0u8; 128];
    match file_type {
        FileTypeKind::Wmv(wmv) => {
            let header = &mut header[..wmv.header.len()];
            file.read_exact(header).is_ok() && header == wmv.header
        }
        FileTypeKind::Jpg(jpg) => {
            let header = &mut header[..jpg.header.len()];
            file.read_exact(header).is_ok() && header == jpg.header
        }
        FileTypeKind::Archive(archive) => {
            let header = &mut header[..archive.header.len()];
            file.read_exact(header).is_ok() && header == archive.header
        }
    }
}

fn parse_file_type(file_type: &str) -> Result<FileTypeKind, &'static str> {
    match file_type.to_ascii_lowercase().as_str() {
        "asf" | "wma" | "wmv" => Ok(FileTypeKind::Wmv(Wmv::default())),
        "jpg" => Ok(FileTypeKind::Jpg(Jpg::default())),
        "zip" | "apk" | "jar" => Ok(FileTypeKind::Archive(Archive::default())),
        _ => Err("Unknown file type"),
    }
}

fn guess_file_type(file_name: &str) -> Result<FileTypeKind, &'static str> {
    file_name
        .split('.')
        .next_back()
        .and_then(|ext| Some(parse_file_type(ext)))
        .unwrap_or(Err("didn't recognize file type"))
}

fn check_if_file_exists_in_zip(
    archive: &mut zip::ZipArchive<File>,
    file_name: &str,
) -> Result<(), &'static str> {
    match archive.by_name_decrypt(file_name, b"") {
        Ok(_) => Ok(()),
        Err(ref e) if e.to_string() == zip::result::ZipError::FileNotFound.to_string() => {
            Err("File doesn't exist in zip")
        }
        Err(_) => Err("Something went wrong locating file in zip"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_determine_file_type() {
        if let Ok(jpg) = parse_file_type("jpg") {
            assert_eq!(
                std::mem::discriminant(&jpg),
                std::mem::discriminant(&FileTypeKind::Jpg(Jpg { header: vec![0] }))
            );
        } else {
            panic!("file type not determined correctly");
        }
    }

    #[test]
    fn test_is_header_valid() {
        let mut test_zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_zip_path.push("test_data/cats.zip");
        let zip_file = std::fs::File::open(&test_zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(&zip_file).unwrap();
        let file = archive.by_name_decrypt("kitten.jpg", b"fun").unwrap();
        assert_eq!(
            true,
            is_header_valid(&mut file.unwrap(), &FileTypeKind::Jpg(Jpg::default()))
        );
    }

    #[test]
    fn test_blitz_zip_with_jpg() {
        let mut test_zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_zip_path.push("test_data/cats.zip");
        let config = Config::new(Args {
            zip_name: test_zip_path.into_os_string().into_string().unwrap(),
            file_name: String::from("kitten.jpg"),
            file_type: Some(String::from("jpg")),
        })
        .unwrap();
        let wordlist = std::fs::read_to_string("test_data/wordlist.txt")
            .expect("Something went wrong reading the file");
        let lines = wordlist.lines().map(|x| x.to_string());
        if let Ok(password) = run(config, lines) {
            assert_eq!(password, "fun");
        } else {
            panic!("password validation logic faild");
        }
    }

    #[test]
    fn test_blitz_zip_without_type_provided() {
        let mut test_zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_zip_path.push("test_data/cats.zip");
        let config = Config::new(Args {
            zip_name: test_zip_path.into_os_string().into_string().unwrap(),
            file_name: String::from("kitten.jpg"),
            file_type: None,
        })
        .unwrap();
        let wordlist = std::fs::read_to_string("test_data/wordlist.txt")
            .expect("Something went wrong reading the file");
        let lines = wordlist.lines().map(|x| x.to_string());
        if let Ok(password) = run(config, lines) {
            assert_eq!(password, "fun");
        } else {
            panic!("password validation logic faild");
        }
    }
}
