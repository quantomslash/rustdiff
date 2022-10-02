use crate::error::DiffError;
use super::algorithm::Algorithm;

const MOD: u32 = 65535;

pub struct Fletcher32 {
    a: u32,
    b: u32,
    current_window: Vec<u8>,
}

impl Fletcher32 {
    pub fn new() -> Self {
        Fletcher32 {
            a: 0,
            b: 0,
            current_window: Vec::new(),
        }
    }
}

impl Algorithm for Fletcher32 {
    fn get_chunk_hash(&mut self, chunk: &[u8]) -> Result<u32, DiffError> {
        for byte in chunk {
            let current_byte = *byte as u32;
            self.a = (self.a + current_byte) % MOD;
            self.b = (self.b + self.a) % MOD;
        }

        self.current_window = chunk.to_vec();

        let hash = self.get_current_hash()?;
        Ok(hash)
    }

    fn get_current_hash(&self) -> Result<u32, DiffError> {
        let hash = (self.b << 16) | self.a;
        Ok(hash)
    }

    fn get_rolling_hash(&mut self, new_byte: &u8) -> Result<u32, DiffError> {
        // Add a byte
        self.a = (self.a + *new_byte as u32) % MOD;
        self.b = (self.b + self.a) % MOD;
        self.current_window.push(new_byte.clone());

        // Remove a byte
        let last_byte = self.current_window[0] as u32;
        let size = self.current_window.len() as u32;
        self.a = (self.a - last_byte) % MOD;
        self.b = (self.b - (size * last_byte as u32)) % MOD;
        self.current_window.remove(0);

        let hash = self.get_current_hash()?;
        Ok(hash)
    }

    fn get_current_window(&self) -> Result<&Vec<u8>, DiffError> {
        Ok(&self.current_window)
    }
}

#[cfg(test)]

mod test {

    use super::*;

    #[test]
    fn test_get_chunk_hash() {
        // Test against a precomputed fletcher hash
        let chunk = "hello world".as_bytes().to_vec();
        let hash = Fletcher32::new().get_chunk_hash(&chunk).unwrap();
        let answer: u32 = 436208732;
        assert_eq!(hash, answer);
    }

    #[test]
    fn test_get_rolling_hash() {
        let chunk = "hello world".as_bytes().to_vec();
        let mut adler = Fletcher32::new();
        // Verify the hash for the first chunk
        let hash = adler.get_chunk_hash(&chunk).unwrap();
        let answer = 436208732;
        assert_eq!(hash, answer);
        // Move the window and check
        let new_byte = 'a' as u8;
        let hash = adler.get_rolling_hash(&new_byte).unwrap();
        let answer = 433914965;
        assert_eq!(hash, answer);
        // Move the window and check
        let new_byte = 'm' as u8;
        let hash = adler.get_rolling_hash(&new_byte).unwrap();
        let answer = 434308189;
        assert_eq!(hash, answer);
    }
}
