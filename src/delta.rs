use crate::algorithms::adler_32::Adler32;
use crate::algorithms::algorithm::Algorithm;
use crate::algorithms::fletcher_32::Fletcher32;
use crate::error::DiffError;
use crate::sign::Signature;
use crate::utils::get_blake2;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Delta {
    I(u32), // Index
    B(u8),  // Byte
}

pub struct HashBlock {
    index: u32,
    weak_hash: u32,
    bytes: Vec<u8>,
}

impl fmt::Debug for HashBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HashBlock index: {}, hash: {} bytes: {:?}",
            self.index,
            self.weak_hash,
            std::str::from_utf8(&self.bytes)
        )
    }
}

pub fn gen_delta_from_file(
    path: &str,
    chunk_size: usize,
    algorithm: &str,
    output_path: &str,
    signatures: HashMap<u32, Signature>,
) -> Result<Vec<Delta>, DiffError> {
    let mut f = File::open(path)?;
    let mut buffer = Vec::<u8>::new();
    f.read_to_end(&mut buffer)?; //TODO

    let mut delta = Vec::<Delta>::new();

    // TODO what happens if chunk is smaller than chunk size

    let hashes = match algorithm {
        "fletcher" => {
            let algo = Fletcher32::new();
            calculate_rolling_hashes(chunk_size, algo, &buffer)?
        }

        _ => {
            let algo = Adler32::new();
            calculate_rolling_hashes(chunk_size, algo, &buffer)?
        }
    };

    let mut index = 0;
    while index < buffer.len() {        
        if index > buffer.len() - chunk_size {
            let chunk = &buffer[index..];
            // Last iterable index
            for byte in chunk {
                delta.push(Delta::B(*byte))
            }
            break;
        }

        let curr_hash = &hashes[index].weak_hash;
        let curr_bytes = hashes[index].bytes.clone();
        if signatures.contains_key(&curr_hash) {
            // Key match!
            if let Some(sign) = &signatures.get(&curr_hash) {
                let checksum = &sign.checksum;
                let this_checksum = get_blake2(curr_bytes)?;
                if checksum == &this_checksum {
                    delta.push(Delta::I(sign.index));
                    index = index + chunk_size;
                    continue;
                }
            } else {
                error!("Something went wrong!, This is not supposed to happen.");
                panic!();
            }
        }
        // If we are here, key does not match, it's modified data
        delta.push(Delta::B(buffer[index]));
        index = index + 1;
    }

    // Write to the output file
    let f = File::create(output_path)?;
    serde_json::to_writer(&f, &delta)?;

    Ok(delta)
}

fn calculate_rolling_hashes(
    chunk_size: usize,
    mut algo: impl Algorithm,
    buffer: &Vec<u8>,
) -> Result<Vec<HashBlock>, DiffError> {
    let chunk = &buffer[0..chunk_size];
    let mut index = 0;
    let weak_hash = algo.get_chunk_hash(chunk)?;
    let first_hash_block = HashBlock {
        index,
        weak_hash,
        bytes: chunk.to_vec(),
    };

    let mut hash_block_list = Vec::new();
    hash_block_list.push(first_hash_block);

    for byte in &buffer[chunk_size..] {
        index = index + 1;
        let weak_hash = algo.get_rolling_hash(byte)?;
        let chunk = algo.get_current_window()?;
        let new_hash_block = HashBlock {
            index,
            weak_hash,
            bytes: chunk.to_vec(),
        };

        hash_block_list.push(new_hash_block);
    }

    Ok(hash_block_list)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::delta::Delta;
    use crate::sign::Signature;
    use rand::{thread_rng, Rng};
    use std::{
        fs::{remove_file, write, File},
        io::BufReader,
    };

    const TEST_IN_FILE: &str = "data/tmp/delta_test_input.txt";
    const TEST_M_IN_FILE: &str = "data/tmp/delta_test_m_input.txt";
    const TEST_DELTA_FILE: &str = "data/tmp/delta_test_output.txt";

    #[test]
    fn test_gen_delta_from_file_all() {
        let max_chunk_size = 16;
        let algorithm = "adler";
        let algorithm2 = "fletcher";
        // Test with varying chunk sizes
        for index in 2..=max_chunk_size {
            test_gen_delta_from_file(index, algorithm);
            test_gen_delta_from_file(index, algorithm2);
        }
    }

    #[test]
    fn test_calculate_rolling_hashes_all() {
        let max_chunk_size = 8;
        let algorithm = "adler";
        let algorithm2 = "fletcher";
        // Test with varying chunk sizes
        for index in 2..=max_chunk_size {
            test_calculate_rolling_hashes(index, algorithm);
            test_calculate_rolling_hashes(index, algorithm2);
        }
    }

    fn test_gen_delta_from_file(chunk_size: usize, algorithm: &str) {
        // Create the test files
        let tmp_in_file = format!("{}_{}", TEST_IN_FILE, get_rnum());
        let tmp_m_in_file = format!("{}_{}", TEST_M_IN_FILE, get_rnum());
        let tmp_out_file = format!("{}_{}", TEST_DELTA_FILE, get_rnum());

        File::create(tmp_in_file.as_str()).unwrap();
        File::create(tmp_m_in_file.as_str()).unwrap();
        File::create(tmp_out_file.as_str()).unwrap();
        // Write some data
        let data = "He stepped gingerly onto the bridge knowing that enchantment awaited on the other side. The teens wondered what was kept in the red shed on the far edge of the school grounds.";
        write(&tmp_in_file, data).unwrap();
        let modified_data = "He stepped readily onto the bridge knowing that enchantment awaited on the other side. The teens wondered what was kept in the black shed on the far edge of the high school grounds.";
        write(&tmp_m_in_file, modified_data).unwrap();
        // Generate the delta
        let (signatures, _) = Signature::gen_sigs(&tmp_in_file, chunk_size, algorithm).unwrap();
        gen_delta_from_file(
            &tmp_m_in_file,
            chunk_size,
            algorithm,
            &tmp_out_file,
            signatures.clone(),
        )
        .unwrap();

        // Get the hashes
        let buffer = modified_data.as_bytes().to_vec();
        let hashes = match algorithm {
            "fletcher" => {
                let algo = Fletcher32::new();
                calculate_rolling_hashes(chunk_size, algo, &buffer).unwrap()
            }

            _ => {
                let algo = Adler32::new();
                calculate_rolling_hashes(chunk_size, algo, &buffer).unwrap()
            }
        };

        // Load the delta from file
        let f = File::open(&tmp_out_file).unwrap();
        let reader = BufReader::new(f);
        let loaded_delta: Vec<Delta> = serde_json::from_reader(reader).unwrap();

        // Ensure they exist in the file
        let mut index = 0;
        while index < buffer.len() {
            let chunk = &buffer[index..];
            if index > buffer.len() - chunk_size {
                break;
            }
            let curr_hash = hashes[index].weak_hash;
            if signatures.contains_key(&curr_hash) {
                let checksum = &signatures.get(&curr_hash).unwrap().checksum;
                let this_checksum = get_blake2(chunk.to_vec()).unwrap();
                if checksum == &this_checksum {
                    let chunk_index = signatures.get(&curr_hash).unwrap().index;
                    let res = loaded_delta.iter().find(|dt| {
                        if let Delta::I(i) = dt {
                            return i == &chunk_index;
                        }
                        false
                    });
                    // Actual test to see delta is there
                    assert_ne!(res, None);
                    index = index + chunk_size;
                    continue;
                }
            }
            index = index + 1;
        }

        // Cleanup
        remove_file(tmp_in_file).unwrap();
        remove_file(tmp_m_in_file).unwrap();
        remove_file(tmp_out_file).unwrap();
    }

    fn test_calculate_rolling_hashes(chunk_size: usize, algorithm: &str) {
        let data = "hello world how are we".as_bytes().to_vec();

        match algorithm {
            "fletcher" => {
                let algo = Fletcher32::new();
                let hashes = calculate_rolling_hashes(chunk_size, algo, &data).unwrap();
                let chunk_hashes = get_chunk_hashes(data, chunk_size, algorithm).unwrap();
                for (index, hashblock) in chunk_hashes.iter().enumerate() {
                    assert_eq!(hashblock.weak_hash, hashes[index].weak_hash);
                    assert_eq!(hashblock.bytes, hashes[index].bytes);
                }
            }

            _ => {
                let algo = Adler32::new();
                let hashes = calculate_rolling_hashes(chunk_size, algo, &data).unwrap();
                let chunk_hashes = get_chunk_hashes(data, chunk_size, "adler").unwrap();
                for (index, hashblock) in chunk_hashes.iter().enumerate() {
                    assert_eq!(hashblock.weak_hash, hashes[index].weak_hash);
                    assert_eq!(hashblock.bytes, hashes[index].bytes);
                }
            }
        }
    }

    fn get_chunk_hashes(
        data: Vec<u8>,
        chunk_size: usize,
        algorithm: &str,
    ) -> Result<Vec<HashBlock>, DiffError> {
        let mut hashblocklist = Vec::new();
        for index in 0..data.len() {
            if index + chunk_size > data.len() {
                break;
            }
            let chunk = &data[index..index + chunk_size];

            let weak_hash = match algorithm {
                "fletcher" => Fletcher32::new().get_chunk_hash(chunk).unwrap(),
                _ => Adler32::new().get_chunk_hash(chunk).unwrap(),
            };

            let hash_block = HashBlock {
                index: index as u32,
                weak_hash,
                bytes: chunk.to_vec(),
            };
            hashblocklist.push(hash_block);
        }

        Ok(hashblocklist)
    }

    fn get_rnum() -> u32 {
        let mut rng = thread_rng();
        rng.gen()
    }
}
