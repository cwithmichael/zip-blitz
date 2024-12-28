use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use zip::read::ZipFile;

#[derive(Debug)]
struct ZipFileConfigError(String);

impl fmt::Display for ZipFileConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ZipFileConfigError: {}", self.0)
    }
}

impl Error for ZipFileConfigError {}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Name of the zip file
    #[clap(short, long)]
    zip_name: String,

    /// Name of the file to test extraction with
    #[clap(short, long)]
    file_name: String,

    /// File extension
    #[clap(short = 't', long)]
    file_extension: Option<String>,
}

pub struct Config {
    pub archive: zip::ZipArchive<File>,
    pub file_name: String,
    pub file_header: Option<Vec<u8>>,
}

impl Config {
    pub fn new(args: Args) -> Result<Config, Box<dyn Error>> {
        let zip_path = std::path::Path::new(&args.zip_name);
        let zip_file = std::fs::File::open(&zip_path)?;
        let file_extension = match &args.file_extension {
            Some(file_extension) => file_extension.to_owned(),
            None => guess_file_type(&args.file_name)?,
        };
        let mut archive = zip::ZipArchive::new(zip_file)?;
        let file_exists = check_if_file_exists_in_zip(&mut archive, &args.file_name);
        if file_exists {
            Ok(Config {
                archive,
                file_name: args.file_name,
                file_header: get_header(&file_extension),
            })
        } else {
            Err(Box::new(ZipFileConfigError(
                "File does not exist in zip".into(),
            )))
        }
    }
}

pub fn run<R>(mut config: Config, mut passwords: R) -> Result<String, &'static str>
where
    R: Iterator<Item = String>,
{
    let header = &config.file_header;
    if let Some(header) = header {
        passwords
            .find(|p| {
                config
                    .archive
                    .by_name_decrypt(&config.file_name, p.as_bytes())
                    .ok()
                    .and_then(|r| r.into())
                    .map_or(false, |mut file| {
                        is_header_valid(&mut file, header.to_vec())
                    })
            })
            .ok_or("Password wasn't found")
    } else {
        return Err("Unable to detect file header");
    }
}

fn is_header_valid(file: &mut ZipFile, file_header: Vec<u8>) -> bool {
    let mut actual_header = [0u8; 128];
    let header = &mut actual_header[..file_header.len()];
    file.read_exact(header).is_ok() && header == file_header
}

pub fn get_header(extension: &str) -> Option<Vec<u8>> {
    match extension {
        "asf" | "wma" | "wmv" => Some(vec![
            0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62,
            0xCE, 0x6C,
        ]),
        "png" => Some(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
        "jpg" => Some(vec![0xFF, 0xD8]),
        "zip" | "apk" | "jar" => Some(vec![0x50, 0x4B, 0x03, 0x04]),
        "xml" => Some(vec![0x3C, 0x3F, 0x78, 0x6D, 0x6C, 0x20]),
        _ => None,
    }
}

fn guess_file_type(file_name: &str) -> Result<String, &'static str> {
    let ext = file_name
        .split('.')
        .next_back()
        .and_then(|ext| Some(ext.to_string()));
    match ext {
        Some(ext) => Ok(ext),
        None => Err("failed to guess file type"),
    }
}

fn check_if_file_exists_in_zip(archive: &mut zip::ZipArchive<File>, file_name: &str) -> bool {
    let mut file_names = archive.file_names();
    let file_exits = file_names.find(|&x| x == file_name);
    match file_exits {
        Some(_) => true,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_header_valid() {
        let mut test_zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_zip_path.push("test_data/cats.zip");
        let zip_file = std::fs::File::open(&test_zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(&zip_file).unwrap();
        let mut file = archive.by_name_decrypt("kitten.jpg", b"fun").unwrap();
        assert_eq!(true, is_header_valid(&mut file, get_header("jpg").unwrap()));
    }

    #[test]
    fn test_blitz_zip_with_jpg() {
        let mut test_zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_zip_path.push("test_data/cats.zip");
        let config = Config::new(Args {
            zip_name: test_zip_path.into_os_string().into_string().unwrap(),
            file_name: String::from("kitten.jpg"),
            file_extension: Some(String::from("jpg")),
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
            file_extension: None,
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
