mod lexc;

use std::fs;
use std::path::PathBuf;

use clap::{ArgAction, Parser};
use lexc::{LexcFile, LexcParser, validate_file};

#[derive(Parser)]
#[command(version = "0.1.0")]
#[command(about = "hayal is a Rust-based implementation of the foam finite-state tool-kit.")]
#[command(long_about = None)]
struct Cli {
    #[arg(short = 'f', long = "file")]
    input_file: PathBuf,
    #[arg(long = "validate", requires = "input_file", action = ArgAction::SetTrue)]
    validate: bool,
}

fn main() {
    let cli = Cli::parse();
    let file_content = match fs::read_to_string(&cli.input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let lexc_parser = LexcParser::new();

    println!(
        "Reading {} ...",
        &cli.input_file.as_path().to_str().unwrap()
    );

    let lexc_file: LexcFile = match lexc_parser.parse(&file_content) {
        Ok(lexc_file) => lexc_file,
        Err(e) => {
            eprintln!("Error parsing file: {:?}", e);
            std::process::exit(1);
        }
    };

    if cli.validate {
        lexc_file.validate();
    }

    println!("{}", lexc_file);
}
