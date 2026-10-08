use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

const MAX_SAMPLE_SIZE: u64 = 8192;
const HALF_SAMPLE_SIZE: usize = 4096;

pub fn fingerprint(path: &Path, size: u64) -> Result<blake3::Hash, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();

    if size <= MAX_SAMPLE_SIZE {
        let mut buffer = Vec::with_capacity(size as usize);
        file.read_to_end(&mut buffer)?;
        hasher.update(&buffer);
    } else {
        let mut buffer = vec![0; HALF_SAMPLE_SIZE];

        file.read_exact(&mut buffer)?;
        hasher.update(&buffer);

        file.seek(SeekFrom::End(-(HALF_SAMPLE_SIZE as i64)))?;
        file.read_exact(&mut buffer)?;
        hasher.update(&buffer);
    }

    Ok(hasher.finalize())
}
