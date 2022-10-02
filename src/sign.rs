use crate::algorithms::adler_32::Adler32;
use crate::algorithms::algorithm::Algorithm;
use crate::algorithms::fletcher_32::Fletcher32;
use crate::error::DiffError;
use crate::utils::get_blake2;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Clone)]
pub struct Signature {
    pub index: u32,
    pub checksum: Vec<u8>,
    pub bytes: Vec<u8>,
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined_checksum: String = self
            .checksum
            .iter()
            .map(|&byte| byte.to_string() + "")
            .collect();

        write!(
            f,
            "\nindex: {} \nbytes {:?} \nchecksum {}\n",
            self.index,
            std::str::from_utf8(&self.bytes),
            joined_checksum
        )
    }
}

impl Signature {
    pub fn gen_sigs_save(
        src_path: &str,
        chunk_size: usize,
        algorithm: &str,
        output_path: &str,
    ) -> Result<u32, DiffError> {
        // Generate them
        let (signatures, collisions) = Signature::gen_sigs(src_path, chunk_size, algorithm)?;

        // Write to the file
        let f = File::create(output_path)?;
        serde_json::to_writer(&f, &signatures)?;

        Ok(collisions)
    }

    pub fn gen_sigs(
        src_path: &str,
        chunk_size: usize,
        algorithm: &str,
    ) -> Result<(HashMap<u32, Signature>, u32), DiffError> {
        let mut f = File::open(src_path)?;
        let mut buffer = Vec::<u8>::new();
        f.read_to_end(&mut buffer)?; // Possible improvement with buffered reader

        let mut signatures = HashMap::new();
        let mut signature_index = 0;
        let mut collisions = 0;

        for index in (0..buffer.len()).step_by(chunk_size) {
            // Check if index is stil valid
            if index + chunk_size > buffer.len() {
                break;
            } else {
                let chunk = &buffer[index..index + chunk_size];
                let result =
                    Signature::add_next_sign(&algorithm, signature_index, chunk, &mut signatures)?;
                match result {
                    true => collisions += 1,
                    false => (),
                }
                signature_index += 1;
            }
        }

        Ok((signatures, collisions))
    }

    /// Create a new Signature and add it to the signatures hashmap
    fn add_next_sign(
        algorithm: &str,
        index: u32,
        chunk: &[u8],
        signatures: &mut HashMap<u32, Signature>,
    ) -> Result<bool, DiffError> {
        let weak_hash = match algorithm {
            "fletcher" => Fletcher32::new().get_chunk_hash(chunk)?,
            _ => Adler32::new().get_chunk_hash(chunk)?,
        };

        let checksum = get_blake2(chunk.to_vec())?;
        let bytes = chunk.to_vec();

        if signatures.contains_key(&weak_hash) {
            // We found the hash in there already,
            // Let's confirm it's not a collision
            if let Some(sign) = &signatures.get(&weak_hash) {
                if &bytes != &sign.bytes {
                    warn!("Key already exists in the signatures, Skipping the block");
                    return Ok(true);
                } else {
                    // Hash already present, move on
                    return Ok(false);
                }                
            }
        }

        let signature = Signature {
            index,
            checksum,
            bytes,
        };

        signatures.insert(weak_hash, signature);

        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{thread_rng, Rng};
    use serde_json;
    use std::{
        fs::{remove_file, write, File},
        io::BufReader,
    };

    const TEST_IN_FILE: &str = "data/tmp/sign_test_input.txt";
    const TEST_SIGN_FILE: &str = "data/tmp/sign_test_output.json";

    #[test]
    fn test_gen_sigs_save_all() {
        let max_chunk_size = 16;
        let algorithm = "adler";
        let algorithm2 = "fletcher";
        // Test with varying chunk sizes
        for index in 1..=max_chunk_size {
            test_gen_sigs_save(index, algorithm);
            test_gen_sigs_save(index, algorithm2);
        }
    }

    #[test]
    fn test_gen_sigs_save_adler_size2() {
        let chunk_size = 2;
        let algorithm = "adler";
        test_gen_sigs_save(chunk_size, algorithm);
    }

    #[test]
    fn test_gen_sigs_save_fletcher_size2() {
        let chunk_size = 2;
        let algorithm = "fletcher";
        test_gen_sigs_save(chunk_size, algorithm);
    }

    #[test]
    fn test_gen_sigs_save_adler_size4() {
        let chunk_size = 4;
        let algorithm = "adler";
        test_gen_sigs_save(chunk_size, algorithm);
    }

    #[test]
    fn test_gen_sigs_save_fletcher_size4() {
        let chunk_size = 4;
        let algorithm = "fletcher";
        test_gen_sigs_save(chunk_size, algorithm);
    }

    #[test]
    fn test_gen_sigs_save_adler_size16() {
        let chunk_size = 16;
        let algorithm = "adler";
        test_gen_sigs_save(chunk_size, algorithm);
    }

    #[test]
    fn test_gen_sigs_save_fletcher_size16() {
        let chunk_size = 16;
        let algorithm = "fletcher";
        test_gen_sigs_save(chunk_size, algorithm);
    }

    #[test]
    fn test_gen_sigs_adler_size5() {
        let chunk_size = 5;
        let algorithm = "adler";
        test_gen_sigs(chunk_size, algorithm);
    }

    #[test]
    fn test_gen_sigs_fletcher_size5() {
        let chunk_size = 5;
        let algorithm = "adler";
        test_gen_sigs(chunk_size, algorithm);
    }

    #[test]
    fn test_gen_sigs_adler_size15() {
        let chunk_size = 15;
        let algorithm = "adler";
        test_gen_sigs(chunk_size, algorithm);
    }

    #[test]
    fn test_gen_sigs_fletcher_size15() {
        let chunk_size = 15;
        let algorithm = "fletcher";
        test_gen_sigs(chunk_size, algorithm);
    }

    #[test]
    fn test_add_next_sign_adler_size3() {
        let chunk_size = 3;
        let algorithm = "adler";
        test_add_next_sign(chunk_size, algorithm);
    }

    #[test]
    fn test_add_next_sign_fletcher_size3() {
        let chunk_size = 3;
        let algorithm = "fletcher";
        test_add_next_sign(chunk_size, algorithm);
    }

    #[test]
    fn test_add_next_sign_adler_size9() {
        let chunk_size = 9;
        let algorithm = "adler";
        test_add_next_sign(chunk_size, algorithm);
    }

    #[test]
    fn test_add_next_sign_fletcher_size9() {
        let chunk_size = 9;
        let algorithm = "fletcher";
        test_add_next_sign(chunk_size, algorithm);
    }

    fn test_gen_sigs_save(chunk_size: usize, algorithm: &str) {
        // Create the test files
        let tmp_in_file = format!("{}_{}", TEST_IN_FILE, get_rnum());
        let tmp_out_file = format!("{}_{}", TEST_SIGN_FILE, get_rnum());

        File::create(tmp_in_file.as_str()).unwrap();
        File::create(tmp_out_file.as_str()).unwrap();
        // Write some data
        let data = "Far far away, behind the word mountains, far from the countries Vokalia and Consonantia, there live the blind texts";
        write(&tmp_in_file, data).unwrap();

        // Write the signatures
        Signature::gen_sigs_save(&tmp_in_file, chunk_size, algorithm, tmp_out_file.as_str())
            .unwrap();

        // Grab signatures directly
        let (signatures, _) = Signature::gen_sigs(&tmp_in_file, chunk_size, algorithm).unwrap();

        // Load the other set of signatures from file
        let f = File::open(tmp_out_file.clone()).unwrap();
        let reader = BufReader::new(f);
        let loaded_signs: HashMap<u32, Signature> = serde_json::from_reader(reader).unwrap();

        // Length should be the same
        assert_eq!(signatures.len(), loaded_signs.len());
        // Check if data matches
        for (hash, sign) in signatures {
            let loaded_sign = loaded_signs.get(&hash).unwrap();
            assert_eq!(sign.index, loaded_sign.index);
            assert_eq!(sign.checksum, loaded_sign.checksum);
            assert_eq!(sign.bytes, loaded_sign.bytes);
        }

        // Cleanup
        remove_file(tmp_in_file).unwrap();
        remove_file(tmp_out_file).unwrap();
    }

    fn test_gen_sigs(chunk_size: usize, algorithm: &str) {
        // Create the test file
        let tmp_in_file = format!("{}_{}", TEST_IN_FILE, get_rnum());
        // Write some test data
        let data = "A kangaroo is really just a rabbit on steroids. When transplanting seedlings, candied teapots will make the task easier.";
        write(&tmp_in_file, data).unwrap();
        // Grab the signatures from the function
        let (signatures, _) = Signature::gen_sigs(&tmp_in_file, chunk_size, algorithm).unwrap();
        // Iterate over them and confirm data
        let buffer = data.as_bytes();
        for index in (0..buffer.len()).step_by(chunk_size) {
            let chunk = &buffer[index..index + chunk_size];
            let weak_hash = match algorithm {
                "fletcher" => Fletcher32::new().get_chunk_hash(chunk).unwrap(),
                _ => Adler32::new().get_chunk_hash(chunk).unwrap(),
            };
            let checksum = get_blake2(chunk.to_vec()).unwrap();

            // Grab the appropriate signature
            let sign = signatures.get(&weak_hash).unwrap();
            // And test
            assert_eq!(checksum, sign.checksum);
            assert_eq!(chunk, sign.bytes);
        }

        // Cleanup
        remove_file(tmp_in_file).unwrap();
    }

    fn test_add_next_sign(chunk_size: usize, algorithm: &str) {
        // Prep work
        let data =
            "The random sentence generator generated a random sentence about a random sentence"
                .as_bytes();
        let mut hmap = HashMap::new();
        // Let's start
        for index in (0..data.len()).step_by(chunk_size) {
            let chunk = &data[index..index + chunk_size];
            let weak_hash = match algorithm {
                "fletcher" => Fletcher32::new().get_chunk_hash(chunk).unwrap(),
                _ => Adler32::new().get_chunk_hash(chunk).unwrap(),
            };
            Signature::add_next_sign(algorithm, index as u32, chunk, &mut hmap).unwrap();
            // Test if it exists in the hashmap
            assert!(hmap.contains_key(&weak_hash));
            // Now ensure that the data is good
            let sign = hmap.get(&weak_hash).unwrap();
            let checksum = get_blake2(chunk.to_vec()).unwrap();
            assert_eq!(chunk, sign.bytes);
            assert_eq!(checksum, sign.checksum);
        }
    }

    fn get_rnum() -> u32 {
        let mut rng = thread_rng();
        rng.gen()
    }
}
