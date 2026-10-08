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

                        if let Some(existing) = candidates.get_mut(&size) {
                            if existing.iter().any(|candidate| candidate.hash == hash) {
                                existing.push(candidate);
                            }
                        } else {
                            candidates.insert(size, vec![candidate]);
                        }
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
