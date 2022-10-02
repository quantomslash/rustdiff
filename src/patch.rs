use crate::delta::Delta;
use crate::error::DiffError;
use crate::sign::Signature;
use log::{error, trace};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};

pub fn patch_file_with_delta(
    delta_file: String,
    out_file: String,
    signatures: HashMap<u32, Signature>,
) -> Result<(), DiffError> {
    let f = File::open(delta_file)?;
    let reader = BufReader::new(f);
    let loaded_delta: Vec<Delta> = serde_json::from_reader(reader)?;

    let mut output = Vec::<u8>::new();

    for delta in loaded_delta {
        match delta {
            Delta::B(b) => {
                trace!("Byte is {:?}", b as char);
                output.push(b);
            }
            Delta::I(i) => {
                trace!("Index is {:?}", i);
                if let Some(mut data) = get_data(i, &signatures) {
                    output.append(&mut data);
                } else {
                    error!("Couldn't find the indexed data while patching file!, Exiting");
                    panic!();
                }
            }
        }
    }

    // Write data to output file
    let mut of = File::create(out_file)?;
    of.write_all(&output)?;

    Ok(())
}

fn get_data(i: u32, signatures: &HashMap<u32, Signature>) -> Option<Vec<u8>> {
    signatures.iter().find_map(|(_, val)| {
        if val.index == i {
            Some(val.bytes.clone())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::delta::gen_delta_from_file;
    use rand::{thread_rng, Rng};

    use std::fs::{read_to_string, remove_file, write, File};

    const TEST_IN_FILE: &str = "data/tmp/patch_test_input.txt";
    const TEST_M_IN_FILE: &str = "data/tmp/patch_test_m_input.txt";
    const TEST_DELTA_FILE: &str = "data/tmp/patch_test_delta.json";
    const TEST_OUT_FILE: &str = "data/tmp/patch_test_output.json";

    #[test]
    fn test_patch_file_with_delta_adler_size3() {
        let chunk_size = 3;
        let algorithm = "adler";
        test_patch_file_with_delta(chunk_size, algorithm);
    }

    #[test]
    fn test_patch_file_with_delta_fletcher_size3() {
        let chunk_size = 3;
        let algorithm = "fletcher";
        test_patch_file_with_delta(chunk_size, algorithm);
    }

    #[test]
    fn test_patch_file_with_delta_adler_size9() {
        let chunk_size = 9;
        let algorithm = "adler";
        test_patch_file_with_delta(chunk_size, algorithm);
    }

    #[test]
    fn test_patch_file_with_delta_fletcher_size9() {
        let chunk_size = 9;
        let algorithm = "fletcher";
        test_patch_file_with_delta(chunk_size, algorithm);
    }

    #[test]
    fn test_get_data_adler_size4() {
        let chunk_size = 4;
        let algorithm = "adler";
        test_get_data(chunk_size, algorithm);
    }

    #[test]
    fn test_get_data_fletcher_size4() {
        let chunk_size = 4;
        let algorithm = "fletcher";
        test_get_data(chunk_size, algorithm);
    }

    #[test]
    fn test_get_data_adler_size12() {
        let chunk_size = 12;
        let algorithm = "adler";
        test_get_data(chunk_size, algorithm);
    }

    #[test]
    fn test_get_data_fletcher_size12() {
        let chunk_size = 12;
        let algorithm = "fletcher";
        test_get_data(chunk_size, algorithm);
    }

    fn test_patch_file_with_delta(chunk_size: usize, algorithm: &str) {
        // Create the test files
        let tmp_in_file = format!("{}_{}", TEST_IN_FILE, get_rnum());
        let tmp_m_in_file = format!("{}_{}", TEST_M_IN_FILE, get_rnum());
        let tmp_delta_file = format!("{}_{}", TEST_DELTA_FILE, get_rnum());
        let tmp_out_file = format!("{}_{}", TEST_OUT_FILE, get_rnum());

        File::create(tmp_in_file.as_str()).unwrap();
        File::create(tmp_m_in_file.as_str()).unwrap();
        File::create(tmp_delta_file.as_str()).unwrap();
        File::create(tmp_out_file.as_str()).unwrap();

        // Write some data
        let data = "He stepped gingerly onto the bridge knowing that enchantment awaited on the other side. The teens wondered what was kept in the red shed on the far edge of the school grounds.";
        write(&tmp_in_file, data).unwrap();
        let modified_data = "He stepped readily onto the bridge knowing that enchantment awaited on the other side. The teens wondered what was kept in the black shed on the far edge of the high school grounds.";
        write(&tmp_m_in_file, modified_data).unwrap();

        // Generate the signatures and delta
        let (signatures, _) = Signature::gen_sigs(&tmp_in_file, chunk_size, algorithm).unwrap();
        gen_delta_from_file(
            &tmp_m_in_file,
            chunk_size,
            algorithm,
            &tmp_delta_file,
            signatures.clone(),
        )
        .unwrap();

        // Patch the file
        patch_file_with_delta(tmp_delta_file.clone(), tmp_out_file.clone(), signatures).unwrap();

        // Verify the results
        let data = read_to_string(tmp_out_file.clone()).unwrap();
        assert_eq!(data, modified_data);

        // Cleanup
        remove_file(tmp_in_file).unwrap();
        remove_file(tmp_m_in_file).unwrap();
        remove_file(tmp_delta_file).unwrap();
        remove_file(tmp_out_file).unwrap();
    }

    fn test_get_data(chunk_size: usize, algorithm: &str) {
        // Create the test files
        let tmp_in_file = format!("{}_{}", TEST_IN_FILE, get_rnum());
        File::create(tmp_in_file.as_str()).unwrap();

        // Write some data
        let data = "They wandered into a strange Tiki bar on the edge of the small beach town";
        write(&tmp_in_file, data).unwrap();

        // Generate the signatures
        let (signatures, _) = Signature::gen_sigs(&tmp_in_file, chunk_size, algorithm).unwrap();

        // Verify data
        for (_, sign) in &signatures {
            let index = sign.index;
            let test_data = get_data(index, &signatures).unwrap();
            assert_eq!(test_data, sign.bytes);
        }

        // Cleanup
        remove_file(tmp_in_file).unwrap();
    }

    fn get_rnum() -> u32 {
        let mut rng = thread_rng();
        rng.gen()
    }
}
