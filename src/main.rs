use std::{collections::HashMap, fs::read_dir, path::PathBuf, process::exit};

mod args;
mod filesystem;
mod hasher;

#[derive(Default)]
struct Config {
    inputs: Vec<PathBuf>,
}

struct Candidate {
    path: PathBuf,
    hash: blake3::Hash,
}

fn group_duplicates(candidates: &[Candidate]) -> HashMap<blake3::Hash, Vec<&Candidate>> {
    let mut groups = HashMap::new();

    for candidate in candidates {
        groups
            .entry(candidate.hash)
            .or_insert_with(Vec::new)
            .push(candidate);
    }

    groups.retain(|_, group: &mut Vec<&Candidate>| group.len() > 1);

    groups
}

fn run(config: &Config) -> i32 {
    let mut candidates: HashMap<u64, Vec<Candidate>> = HashMap::new();
    let mut exit_code = 0;

    for input in &config.inputs {
        match read_dir(input) {
            Ok(dir) => filesystem::scan(dir, &mut candidates),
            Err(error) => {
                eprintln!("dupfd: {}", error);
                exit_code = 1;
            }
        }
    }

    let mut candidate_count = 0;

    for (size, candidates) in &candidates {
        let groups = group_duplicates(candidates);

        for group in groups.values() {
            println!("{} bytes:", size);

            candidate_count += group.len();

            for candidate in group {
                println!("  {}", candidate.path.display());
            }
        }
    }

    println!("{} candidate files", candidate_count);

    exit_code
}

fn main() {
    match args::parse() {
        Ok(config) => exit(run(&config)),
        Err(error) => {
            eprint!("dupfd: {}", error);
            exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_candidates_by_fingerprint() {
        let hash_a = blake3::hash(b"AAAA");
        let hash_b = blake3::hash(b"BBBB");
        let hash_c = blake3::hash(b"CCCC");

        // Two duplicate pairs sharing the same file size.
        let candidates = vec![
            Candidate {
                path: "a.txt".into(),
                hash: hash_a,
            },
            Candidate {
                path: "b.txt".into(),
                hash: hash_b,
            },
            Candidate {
                path: "c.txt".into(),
                hash: hash_a,
            },
            Candidate {
                path: "d.txt".into(),
                hash: hash_b,
            },
            Candidate {
                path: "e.txt".into(),
                hash: hash_c,
            },
        ];

        let groups = group_duplicates(&candidates);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[&hash_a].len(), 2);
        assert_eq!(groups[&hash_b].len(), 2);
        assert!(!groups.contains_key(&hash_c));
    }
}
