use std::{
    collections::HashMap,
    fs::{ReadDir, read_dir},
};

use crate::{Candidate, hasher::fingerprint};

pub fn scan(entries: ReadDir, candidates: &mut HashMap<u64, Vec<Candidate>>) {
    for entry in entries {
        match entry {
            Ok(entry) => match entry.file_type() {
                Ok(file_type) if file_type.is_dir() => match read_dir(entry.path()) {
                    Ok(dir_entries) => {
                        scan(dir_entries, candidates);
                    }
                    Err(error) => {
                        eprintln!("dupfd: {}", error);
                    }
                },
                Ok(file_type) if file_type.is_file() => match entry.metadata() {
                    Ok(metadata) => {
                        let size = metadata.len();
                        let path = entry.path();
                        let hash = match fingerprint(&path, size) {
                            Ok(hash) => hash,
                            Err(error) => {
                                eprintln!("dupfd: {}", error);
                                continue;
                            }
                        };

                        let candidate = Candidate { path, hash };

                        candidates.entry(size).or_default().push(candidate);
                    }
                    Err(error) => {
                        eprintln!("dupfd: {}", error);
                    }
                },
                Ok(_) => {}
                Err(error) => {
                    eprintln!("dupfd: {}", error);
                }
            },
            Err(error) => {
                eprintln!("dupfd: {}", error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let entries = read_dir(dir.path()).unwrap();
        let mut candidates = HashMap::new();

        scan(entries, &mut candidates);

        assert!(candidates.is_empty());
    }

    #[test]
    fn scan_identical_files() {
        let dir = tempfile::tempdir().unwrap();

        std::fs::write(dir.path().join("a.txt"), b"AAAA").unwrap();
        std::fs::write(dir.path().join("b.txt"), b"AAAA").unwrap();

        let entries = read_dir(dir.path()).unwrap();
        let mut candidates = HashMap::new();

        scan(entries, &mut candidates);

        // Candidates are grouped by file size (4 bytes for "AAAA").
        let group = candidates.get(&4).unwrap();

        assert_eq!(group.len(), 2);
        assert_eq!(group[0].hash, group[1].hash);
    }

    #[test]
    fn scan_nested_directories() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");

        std::fs::create_dir(&nested).unwrap();

        // Use different file sizes (4 and 5 bytes, respectively) to isolate
        // recursive traversal from fingerprint grouping.
        std::fs::write(dir.path().join("a.txt"), b"AAAA").unwrap();
        std::fs::write(nested.join("b.txt"), b"BBBBB").unwrap();

        let entries = read_dir(dir.path()).unwrap();
        let mut candidates = HashMap::new();

        scan(entries, &mut candidates);

        // Files from both directories should be discovered.
        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn scan_preserves_multiple_duplicate_groups_of_same_size() {
        let dir = tempfile::tempdir().unwrap();

        // Two distinct duplicate pairs, all 4 bytes long.
        // The scanner must retain both pairs regardless of directory iteration order.
        std::fs::write(dir.path().join("a.txt"), b"AAAA").unwrap();
        std::fs::write(dir.path().join("b.txt"), b"BBBB").unwrap();
        std::fs::write(dir.path().join("c.txt"), b"AAAA").unwrap();
        std::fs::write(dir.path().join("d.txt"), b"BBBB").unwrap();

        let entries = read_dir(dir.path()).unwrap();
        let mut candidates = HashMap::new();

        scan(entries, &mut candidates);

        // Both duplicate pairs must survive candidate grouping.
        let group = candidates.get(&4).unwrap();

        assert_eq!(group.len(), 4);
    }
}
