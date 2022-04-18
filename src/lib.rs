mod file_type;
use clap::Parser;
use file_type::{Archive, FileType, Jpg, Wmv};
use std::error::Error;
use std::fs::File;

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

pub fn is_header_valid(data: &[u8], file_type: &FileTypeKind) -> bool {
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
        Err(ref e) => {
            if e.to_string() == zip::result::ZipError::FileNotFound.to_string() {
                Err("File doesn't exist in zip")
            } else {
                Ok(())
            }
        }
    }
}
