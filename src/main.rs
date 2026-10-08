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

fn run(config: &Config) -> i32 {
    let mut candidates: HashMap<u64, Vec<Candidate>> = HashMap::new();
    let mut exit_code = 0;

    for input in &config.inputs {
        match read_dir(input) {
            Ok(dir) => {
                filesystem::scan(dir, &mut candidates);

                for (size, candidates) in &candidates {
                    if candidates.len() > 1 {
                        println!("{} bytes:", size);

                        for candidate in candidates {
                            println!("  {}", candidate.path.display());
                        }
                    }
                }
                let candidate_count: usize = candidates
                    .values()
                    .filter(|paths| paths.len() > 1)
                    .map(Vec::len)
                    .sum();

                println!("{} candidate files", candidate_count);
            }
            Err(error) => {
                eprintln!("dupfd: {}", error);
                exit_code = 1;
            }
        }
    }

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
