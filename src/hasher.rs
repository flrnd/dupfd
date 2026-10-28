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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn fingerprint_small_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        let content = b"Hello, dupfd";
        file.write_all(content).unwrap();

        let result = fingerprint(file.path(), content.len() as u64).unwrap();
        let expected = blake3::hash(content);

        assert_eq!(result, expected);
    }

    #[test]
    fn fingerprint_hashes_entire_file_at_same_boundary() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        // Exactly 8 KiB must still use full-file hashing, not sampling.
        let content = vec![0xAB; MAX_SAMPLE_SIZE as usize];
        file.write_all(&content).unwrap();

        let result = fingerprint(file.path(), content.len() as u64).unwrap();
        let expected = blake3::hash(&content);

        assert_eq!(result, expected);
    }

    #[test]
    fn fingerprint_samples_file_above_boundary() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        // 8193 bytes: first 4 KiB + one excluded byte + last 4 KiB.
        // This is the smallest file that triggers sampling instead of full hashing.
        let mut content = vec![0xAA_u8; HALF_SAMPLE_SIZE];
        content.push(0xFF);
        content.extend_from_slice(&vec![0xBB_u8; HALF_SAMPLE_SIZE]);
        file.write_all(&content).unwrap();

        let result = fingerprint(file.path(), content.len() as u64).unwrap();

        // The middle byte must not contribute to the fingerprint.
        let mut hasher = blake3::Hasher::new();
        hasher.update(&vec![0xAA_u8; HALF_SAMPLE_SIZE]);
        hasher.update(&vec![0xBB_u8; HALF_SAMPLE_SIZE]);
        let expected = hasher.finalize();

        assert_eq!(result, expected);
    }

    #[test]
    fn finterprint_ignores_middle_of_large_files() {
        let mut file_a = tempfile::NamedTempFile::new().unwrap();
        let mut file_b = tempfile::NamedTempFile::new().unwrap();

        // Both files have identical sampled regions but different middle bytes.
        // This verifies why fingerprint matches require full-file verification.
        let mut content_a = vec![0xAA_u8; HALF_SAMPLE_SIZE];
        content_a.extend_from_slice(&[0x11; 1024]);
        content_a.extend_from_slice(&[0xBB; HALF_SAMPLE_SIZE]);

        let mut content_b = vec![0xAA_u8; HALF_SAMPLE_SIZE];
        content_b.extend_from_slice(&[0x22; 1024]);
        content_b.extend_from_slice(&[0xBB; HALF_SAMPLE_SIZE]);

        file_a.write_all(&content_a).unwrap();
        file_b.write_all(&content_b).unwrap();

        let hash_a = fingerprint(file_a.path(), content_a.len() as u64).unwrap();
        let hash_b = fingerprint(file_b.path(), content_b.len() as u64).unwrap();

        assert_ne!(content_a, content_b);
        assert_eq!(hash_a, hash_b);
    }
}
