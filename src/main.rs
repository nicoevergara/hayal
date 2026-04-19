mod lexc;

use std::fs;
use std::path::PathBuf;

use clap::Parser;
use lexc::{LexcFile, LexcParser};

#[derive(Parser)]
#[command(version = "0.1.0")]
#[command(about = "hayal is a Rust-based implementation of the foam finite-state tool-kit.")]
#[command(long_about = None)]
struct Cli {
    #[arg(short = 'f', long = "file")]
    input_file: PathBuf,
    #[arg(long = "validate", requires = "input_file")]
    validate: Option<bool>,
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
    let lexc_tokens: LexcFile = match lexc_parser.parse(&file_content) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Error parsing file: {:?}", e);
            std::process::exit(1);
        }
    };
}
