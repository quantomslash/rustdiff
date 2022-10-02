use clap::{Parser, Subcommand};
use log::{error, info, warn};
use rustdiff::delta::gen_delta_from_file;
use rustdiff::error::DiffError;
use rustdiff::patch::patch_file_with_delta;
use rustdiff::sign::Signature;
use simple_logger::SimpleLogger;
use std::error::Error;

const DEFAULT_SIGN_FILE: &str = "data/output/signs.json";
const DEFAULT_DELTA_FILE: &str = "data/output/delta.json";
const DEFAULT_PATCH_FILE: &str = "data/output/patch.json";
const DEFAULT_CHUNK_SIZE: u8 = 4;
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Sign {
        file: String,
        chunk_size: Option<u8>,
        algorithm: Option<String>,
        output_path: Option<String>,
    },
    Delta {
        file1: String,
        file2: String,
        chunk_size: Option<u8>,
        algorithm: Option<String>,
        output_path: Option<String>,
    },
    Patch {
        file1: String,
        file2: String,
        chunk_size: Option<u8>,
        algorithm: Option<String>,
        output_path: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    SimpleLogger::new().init().unwrap();

    // Get the args
    let cli = Cli::parse();

    match &cli.command {
        Commands::Sign {
            file,
            chunk_size,
            algorithm,
            output_path,
        } => {
            if !(std::path::Path::new(file).exists()) {
                error!("File {} doesn't exist, Exiting!", file);
                panic!();
            }
            info!("You requested to generate signature of the file {}", file);
            chunk_size_msg(chunk_size);
            algo_msg(algorithm);
            path_msg(output_path);
            gen_sign(file, chunk_size, algorithm, output_path)?;
        }
        Commands::Delta {
            file1,
            file2,
            chunk_size,
            algorithm,
            output_path,
        } => {
            if !(std::path::Path::new(file1).exists()) {
                error!("File {} doesn't exist, Exiting!", file1);
                panic!();
            }
            if !(std::path::Path::new(file2).exists()) {
                error!("File {} doesn't exist, Exiting!", file2);
                panic!();
            }
            info!(
                "You requested to generate delta with files {} and {}",
                file1, file2
            );
            chunk_size_msg(chunk_size);
            algo_msg(algorithm);
            path_msg(output_path);
            gen_delta(file1, file2, chunk_size, algorithm, output_path)?;
        }
        Commands::Patch {
            file1,
            file2,
            chunk_size,
            algorithm,
            output_path,
        } => {
            if !(std::path::Path::new(file1).exists()) {
                error!("File {} doesn't exist, Exiting!", file1);
                panic!();
            }
            if !(std::path::Path::new(file2).exists()) {
                error!("File {} doesn't exist, Exiting!", file2);
                panic!();
            }
            info!(
                "You requested to patch original file {} with delta {}",
                file1, file2
            );
            chunk_size_msg(chunk_size);
            algo_msg(algorithm);
            path_msg(output_path);
            patch(file1, file2, chunk_size, algorithm, output_path)?;
        }
    }

    info!("All done!");
    Ok(())
}

fn gen_sign(
    file: &String,
    chunk_size: &Option<u8>,
    algorithm: &Option<String>,
    output_path: &Option<String>,
) -> Result<(), DiffError> {
    // Verify the args
    let (size, algo) = verify_args(chunk_size, algorithm);
    // Check if output path is provided
    let out_path = match output_path {
        Some(path) => path,
        None => DEFAULT_SIGN_FILE,
    };
    // Generate the signatures
    let collisions = Signature::gen_sigs_save(file, size.into(), algo.as_str(), out_path)?;
    match collisions > 0 {
        true => warn!(
            "{} collisions ocurred while generating signatures",
            collisions
        ),
        false => (),
    }
    info!("Output saved to {}", out_path);
    // All good
    Ok(())
}

fn gen_delta(
    file1: &String,
    file2: &String,
    chunk_size: &Option<u8>,
    algorithm: &Option<String>,
    output_path: &Option<String>,
) -> Result<(), DiffError> {
    // Verify the args
    let (size, algo) = verify_args(chunk_size, algorithm);
    // Check if output path is provided
    let out_path = match output_path {
        Some(path) => path,
        None => DEFAULT_DELTA_FILE,
    };
    // Let's generate the signatures first
    let (signatures, collisions) = Signature::gen_sigs(file1, size.into(), algo.as_str())?;
    match collisions > 0 {
        true => warn!(
            "{} collisions ocurred while generating signatures",
            collisions
        ),
        false => (),
    }
    // Generate the delta
    gen_delta_from_file(file2, size.into(), algo.as_str(), out_path, signatures)?;
    info!("Output saved to {}", out_path);
    // All good
    Ok(())
}

fn patch(
    file1: &String,
    file2: &String,
    chunk_size: &Option<u8>,
    algorithm: &Option<String>,
    output_path: &Option<String>,
) -> Result<(), DiffError> {
    // Verify the args
    let (size, algo) = verify_args(chunk_size, algorithm);
    // Check if output path is provided
    let out_path = match output_path {
        Some(path) => path,
        None => DEFAULT_PATCH_FILE,
    };
    // Let's generate the signatures first
    let (signatures, collisions) = Signature::gen_sigs(file1, size.into(), algo.as_str())?;
    match collisions > 0 {
        true => warn!(
            "{} collisions ocurred while generating signatures",
            collisions
        ),
        false => (),
    }
    // Patch the file
    patch_file_with_delta(file2.to_string(), out_path.to_string(), signatures)?;
    info!("Output saved to {}", out_path);
    // All good
    Ok(())
}

fn verify_args(chunk_size: &Option<u8>, algorithm: &Option<String>) -> (u8, String) {
    // Check if chunk size is provided, otherwise use default
    let size = match chunk_size {
        Some(s) => s,
        None => &DEFAULT_CHUNK_SIZE,
    };

    // Check if algorithm provided, otherwise use default
    let algo = match algorithm {
        Some(al) => al,
        None => "adler",
    };

    (*size, algo.to_string())
}

fn chunk_size_msg(chunk_size: &Option<u8>) {
    if let Some(size) = chunk_size {
        info!("Using chunk size {}", size);
    }
}

fn algo_msg(algorithm: &Option<String>) {
    if let Some(algo) = algorithm {
        if !(algo.as_str() == "adler" || algo.as_str() == "fletcher") {
            panic!("Not a valid algorithm value, use either 'adler' or 'fletcher'");
        }
        info!("Using algorithm {}", algo);
    }
}

fn path_msg(output_path: &Option<String>) {
    if let Some(path) = output_path {
        info!("Output path provided {}", path);
    }
}
