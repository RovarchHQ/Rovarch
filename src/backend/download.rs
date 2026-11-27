use super::background::Progress;
use super::req_agent::download_agent;
use anyhow::Context;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

// Larger chunk size for better download throughput (1MB)
const CHUNK_SIZE: usize = 1024 * 1024;
// Smaller chunk for hashing to reduce memory pressure
const HASH_CHUNK_SIZE: usize = 64 * 1024;
const MAX_ATTEMPTS: u32 = 4;
const RANGE_NOT_SATISFIABLE: u16 = 416;
const PARTIAL_CONTENT: u16 = 206;

pub fn download_file(url: &str, destination: &Path, progress: &Progress) -> anyhow::Result<()> {
    if let Some(directory) = destination.parent() {
        fs::create_dir_all(directory)?;
    }

    let unfinished_path = destination.with_extension("partial");
    let mut last_error = None;
    for attempt in 1..=MAX_ATTEMPTS {
        match download_attempt(url, &unfinished_path, progress) {
            Ok(()) => {
                fs::rename(&unfinished_path, destination)
                    .with_context(|| format!("Couldn't install {url}"))?;
                return Ok(());
            }
            Err(error) => {
                last_error = Some(error);
                if attempt < MAX_ATTEMPTS {
                    std::thread::sleep(Duration::from_millis(1000 << (attempt - 1)));
                }
            }
        }
    }

    let _ = fs::remove_file(&unfinished_path);
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("The download of {url} failed")))
        .with_context(|| format!("Couldn't download {url}"))
}

fn download_attempt(url: &str, unfinished_path: &Path, progress: &Progress) -> anyhow::Result<()> {
    let offset = starting_offset(unfinished_path)?;
    let mut request = download_agent().get(url);
    if offset > 0 {
        request = request.header("Range", &format!("bytes={offset}-"));
    }
    let mut response = request
        .call()
        .with_context(|| format!("The download of {url} was interrupted"))?;

    if offset > 0 && response.status() == RANGE_NOT_SATISFIABLE {
        fs::remove_file(unfinished_path).ok();
        return download_attempt(url, unfinished_path, progress);
    }

    let resuming = offset > 0 && response.status() == PARTIAL_CONTENT;
    let available_size = response
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|&size| size > 0);
    let total_size = if resuming {
        available_size.map(|size| offset + size)
    } else {
        available_size
    };

    let mut file = if resuming {
        File::options().append(true).open(unfinished_path)?
    } else {
        File::create(unfinished_path)?
    };
    let mut position = if resuming { offset } else { 0 };
    let mut chunk = vec![0; CHUNK_SIZE];
    let mut body = response.body_mut().as_reader();

    loop {
        let read_size = body
            .read(&mut chunk)
            .with_context(|| format!("The download of {url} was interrupted"))?;
        if read_size == 0 {
            break;
        }
        file.write_all(&chunk[..read_size])?;
        position += read_size as u64;
        if let Some(total_size) = total_size {
            progress.fraction(position as f64 / total_size as f64);
        }
    }

    file.sync_all()?;
    Ok(())
}

fn starting_offset(unfinished_path: &Path) -> anyhow::Result<u64> {
    match fs::metadata(unfinished_path) {
        Ok(metadata) => Ok(metadata.len()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(error.into()),
    }
}

pub fn checksum_listed_for(checksums: &str, file_name: &str) -> Option<String> {
    checksums.lines().find_map(|line| {
        let (checksum, name) = line.split_once(char::is_whitespace)?;
        let name = name.trim_start().trim_start_matches('*');
        (name == file_name).then(|| checksum.to_ascii_lowercase())
    })
}

pub fn sha256_of_file(path: &Path) -> anyhow::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut chunk = vec![0; HASH_CHUNK_SIZE];

    loop {
        let read_size = file.read(&mut chunk)?;
        if read_size == 0 {
            break;
        }
        hasher.update(&chunk[..read_size]);
    }

    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub fn replace_directory(new_contents: &Path, destination: &Path) -> anyhow::Result<()> {
    if destination.exists() {
        fs::remove_dir_all(destination)
            .with_context(|| format!("Couldn't remove {}", destination.display()))?;
    }
    fs::rename(new_contents, destination)
        .with_context(|| format!("Couldn't move files into {}", destination.display()))?;
    Ok(())
}

pub fn stop_others_from_writing(directory: &Path) -> anyhow::Result<()> {
    const GROUP_AND_OTHERS_WRITE: u32 = 0o022;

    // Use an explicit stack instead of recursion to avoid stack overflow
    // and reduce function call overhead
    let mut stack = vec![directory.to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&current_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                continue;
            };
            if metadata.is_symlink() {
                continue;
            }
            let mode = metadata.permissions().mode();
            if mode & GROUP_AND_OTHERS_WRITE != 0 {
                let _ = fs::set_permissions(
                    &path,
                    fs::Permissions::from_mode(mode & !GROUP_AND_OTHERS_WRITE),
                );
            }
            if metadata.is_dir() {
                stack.push(path);
            }
        }
    }
    Ok(())
}

pub fn remove_directory_if_present(directory: &Path) -> anyhow::Result<()> {
    if directory.exists() {
        fs::remove_dir_all(directory)
            .with_context(|| format!("Couldn't remove {}", directory.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::checksum_listed_for;

    #[test]
    fn reads_both_checksum_file_styles() {
        let text = "AAA  plain\nBBB *binary\n";
        assert_eq!(checksum_listed_for(text, "plain").as_deref(), Some("aaa"));
        assert_eq!(checksum_listed_for(text, "binary").as_deref(), Some("bbb"));
        assert_eq!(checksum_listed_for(text, "missing"), None);
    }
}
