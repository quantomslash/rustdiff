use crate::error::DiffError;

pub trait Algorithm {
    fn get_chunk_hash(&mut self, chunk: &[u8]) -> Result<u32, DiffError>;

    fn get_rolling_hash(&mut self, new_byte: &u8) -> Result<u32, DiffError>;

    fn get_current_hash(&self) -> Result<u32, DiffError>;

    fn get_current_window(&self) -> Result<&Vec<u8>, DiffError>;
}
