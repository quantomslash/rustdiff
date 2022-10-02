use crate::error::DiffError;
use blake2::{Blake2s256, Digest};

pub fn get_blake2(chunk: Vec<u8>) -> Result<Vec<u8>, DiffError> {
    let mut hasher = Blake2s256::new();
    hasher.update(chunk);
    let result = hasher.finalize().as_slice().to_vec();

    Ok(result)
}
