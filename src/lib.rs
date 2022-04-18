mod file_type;
use clap::Parser;
use file_type::{Archive, FileType, Jpg, Wmv};
use std::error::Error;
use std::fs::File;
use std::io::{self};

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
    file_type: String,
}

pub struct Config<'a> {
    pub archive: zip::ZipArchive<File>,
    pub file_name: String,
    pub file_type: FileTypeKind<'a>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum FileTypeKind<'a> {
    Wmv(Wmv<'a>),
    Jpg(Jpg<'a>),
    Archive(Archive<'a>),
}

impl<'a> Config<'a> {
    pub fn new(args: Args) -> Result<Config<'a>, Box<dyn Error>> {
        let zip_path = std::path::Path::new(&args.zip_name);
        let ft = &args.file_type.as_str();
        let file_type = determine_file_type((&ft).to_string())?;
        let zip_file = std::fs::File::open(&zip_path)?;
        let mut archive = zip::ZipArchive::new(zip_file)?;
        check_if_file_exists_in_zip(&mut archive, &args.file_name)?;
        Ok(Config {
            archive,
            file_name: args.file_name,
            file_type: file_type.into(),
        })
    }
}

fn is_header_valid(data: &[u8], file_type: &FileTypeKind) -> bool {
    match file_type {
        FileTypeKind::Wmv(wmv) => wmv.is_valid_header(data),
        FileTypeKind::Jpg(jpg) => jpg.is_valid_header(data),
        FileTypeKind::Archive(archive) => archive.is_valid_header(data),
    }
}

fn determine_file_type<'a>(file_type: String) -> Result<FileTypeKind<'a>, &'static str> {
    match file_type.to_ascii_lowercase().as_str() {
        "asf" | "wma" | "wmv" => Ok(FileTypeKind::Wmv(Wmv::new())),
        "jpg" => Ok(FileTypeKind::Jpg(Jpg::new())),
        "zip" | "apk" | "jar" => Ok(FileTypeKind::Archive(Archive::new())),
        _ => Err("Unknown file type"),
    }
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

pub fn run<R>(mut config: Config, input: R) -> Result<String, &'static str>
where
    R: io::BufRead,
{
    let mut correct_password = String::from("");
    for line in input.lines() {
        match line {
            Ok(password) => {
                if let Ok(file) = config
                    .archive
                    .by_name_decrypt(&config.file_name, password.as_bytes())
                {
                    match file {
                        Ok(f) => {
                            let data: Vec<u8> = io::Read::bytes(f)
                                .take(12) // arbitrary number
                                .map(|d| d.unwrap_or(0))
                                .collect();
                            if is_header_valid(&data, &config.file_type) {
                                correct_password = password.to_string();
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
    if correct_password.is_empty() {
        return Err("Password wasn't found");
    }
    Ok(correct_password.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_determine_file_type() {
        if let Ok(jpg) = determine_file_type(String::from("jpg")) {
            assert_eq!(
                std::mem::discriminant(&jpg),
                std::mem::discriminant(&FileTypeKind::Jpg(Jpg { header: &[0] }))
            );
        } else {
            panic!("file type not determined correctly");
        }
    }

    #[test]
    fn test_is_valid_header() {
        let data = [0xFF, 0xD8];
        let file_type = determine_file_type((String::from("jpg")).to_string()).unwrap();
        let ft = match file_type {
            FileTypeKind::Jpg(jpg) => jpg,
            _ => panic!("Unable to determine file type"),
        };
        assert_eq!(true, ft.is_valid_header(&data));
    }

    #[test]
    fn test_blitz_zip_with_jpg() {
        let mut test_zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_zip_path.push("test_data/cats.zip");
        let config = Config::new(Args {
            zip_name: test_zip_path.into_os_string().into_string().unwrap(),
            file_name: String::from("kitten.jpg"),
            file_type: String::from("jpg"),
        })
        .unwrap();
        let wordlist = std::fs::read_to_string("test_data/wordlist.txt")
            .expect("Something went wrong reading the file");
        if let Ok(password) = run(config, wordlist.as_bytes()) {
            assert_eq!(password, "fun");
        } else {
            panic!("password validation logic faild");
        }
    }
}
