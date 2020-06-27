use super::model::Rom;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

pub enum ArchiveType {
    SEVENZIP,
    ZIP,
}

pub struct ArchiveInfo {
    pub path: String,
    pub size: u64,
    pub crc: String,
}

pub fn parse_archive(file_path: &PathBuf) -> Result<Vec<ArchiveInfo>, Box<dyn Error>> {
    println!("Scanning {:?}", file_path.file_name().unwrap());
    let output = Command::new("7z")
        .arg("l")
        .arg("-slt")
        .arg(&file_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        println!("{}", stderr);
        bail!(stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|&line| {
            line.starts_with("Path =") || line.starts_with("Size =") || line.starts_with("CRC =")
        })
        .skip(1) // the first line is the archive itself
        .map(|line| line.split("=").last().unwrap().trim()) // keep only the rhs
        .collect();

    // each chunk will have the path, size and crc respectively
    let mut sevenzip_infos: Vec<ArchiveInfo> = Vec::new();
    for info in lines.chunks(3) {
        let sevenzip_info = ArchiveInfo {
            path: String::from(info.get(0).unwrap().to_owned()),
            size: FromStr::from_str(info.get(1).unwrap()).unwrap(),
            crc: String::from(info.get(2).unwrap().to_owned()),
        };
        sevenzip_infos.push(sevenzip_info);
    }
    Ok(sevenzip_infos)
}

pub fn move_file_in_archive(
    archive_path: &PathBuf,
    sevenzip_info: &ArchiveInfo,
    rom: &Rom,
) -> Result<(), Box<dyn Error>> {
    if sevenzip_info.path != rom.name {
        println!("Renaming \"{}\" to \"{}\"", sevenzip_info.path, rom.name);
        let output = Command::new("7z")
            .arg("rn")
            .arg(archive_path)
            .arg(&sevenzip_info.path)
            .arg(&rom.name)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr)?;
            println!("{}", stderr);
            bail!(stderr);
        }
    }
    Ok(())
}

pub fn extract_files_from_archive(
    archive_path: &PathBuf,
    paths: &Vec<&str>,
    directory: &Path,
) -> Result<(), Box<dyn Error>> {
    println!("Extracting {:?}", paths);
    let output = Command::new("7z")
        .arg("x")
        .arg(archive_path)
        .args(paths)
        .current_dir(directory)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        println!("{}", stderr);
        bail!(stderr)
    }
    Ok(())
}

pub fn add_files_to_archive(
    archive_path: &PathBuf,
    paths: &Vec<&str>,
    current_directory: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    println!("Compressing {:?}", paths);
    let output = Command::new("7z")
        .arg("a")
        .arg(archive_path)
        .args(paths)
        .arg("-mx=9")
        .current_dir(current_directory)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        println!("{}", stderr);
        bail!(stderr)
    }
    Ok(())
}