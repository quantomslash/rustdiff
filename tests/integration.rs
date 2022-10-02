use rand::{thread_rng, Rng};
use rustdiff::delta::gen_delta_from_file;
use rustdiff::patch::patch_file_with_delta;
use rustdiff::sign::Signature;
use std::fs::{read_to_string, remove_file, write, File};

const TEST_IN_FILE: &str = "data/tmp/patch_test_input.txt";
const TEST_M_IN_FILE: &str = "data/tmp/patch_test_m_input.txt";
const TEST_DELTA_FILE: &str = "data/tmp/patch_test_delta.json";
const TEST_OUT_FILE: &str = "data/tmp/patch_test_output.json";

#[test]
fn test_modify_add_data_all() {
    let max_chunk_size = 32;
    let algorithm = "adler";
    let algorithm2 = "fletcher";
    // Test with varying chunk sizes
    for i in 2..=max_chunk_size {
        test_modify_add_data(i, algorithm);
        test_modify_add_data(i, algorithm2);
    }
}

#[test]
fn test_modify_remove_data_all() {
    let max_chunk_size = 16;
    let algorithm = "adler";
    let algorithm2 = "fletcher";
    // Test with varying chunk sizes
    for i in 2..=max_chunk_size {
        test_modify_remove_data(i, algorithm);
        test_modify_remove_data(i, algorithm2);
    }
}

fn test_modify_add_data(chunk_size: usize, algorithm: &str) {
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
    let data = "Red is greener than purple, for sure.";
    write(&tmp_in_file, data).unwrap();
    let modified_data = "Red is greener than purple, for sure. Yepp.";
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

fn test_modify_remove_data(chunk_size: usize, algorithm: &str) {
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
    let data = "Red is greener than purple, for sure.";
    write(&tmp_in_file, data).unwrap();
    let modified_data = "Red is greener, for sure.";
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

fn get_rnum() -> u32 {
    let mut rng = thread_rng();
    rng.gen()
}
