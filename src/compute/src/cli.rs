use std::path::PathBuf;

use clap::{Arg, Command, ValueHint};

pub fn build_command() -> Command {
    Command::new("breakthrough")
        .about("Testing environment for the Breakthrough game")
        .version("1.0.0")
        .arg(
            Arg::new("config")
                .index(1)
                .required(true)
                .value_name("CONFIG")
                .value_parser(clap::value_parser!(PathBuf))
                .value_hint(ValueHint::FilePath)
                .default_value("config.toml")
                .help("Path to the TOML configuration file"),
        )
        .arg(Arg::new("white-output")
                .long("white-output")
                .required(false)
                .value_parser(clap::value_parser!(PathBuf))
                .value_hint(ValueHint::FilePath)
                .default_value("white_output.jsonl")
                .help("Path to the output JSONL file for the white player")
        )
        .arg(Arg::new("black-output")
                .long("black-output")
                .required(false)
                .value_parser(clap::value_parser!(PathBuf))
                .value_hint(ValueHint::FilePath)
                .default_value("black_output.jsonl")
                .help("Path to the output JSONL file for the black player")
        )
        .arg(
            Arg::new("board-width")
                .long("board-width")
                .required(false)
                .value_parser(clap::value_parser!(u8))
                .help("Width of the game board"),
        )
        .arg(
            Arg::new("board-height")
                .long("board-height")
                .required(false)
                .value_parser(clap::value_parser!(u8))
                .help("Height of the game board"),
        )
        .arg(
            Arg::new("seed")
                .long("seed")
                .required(false)
                .value_parser(clap::value_parser!(u64))
                .help("The seed for the random number generator to ensure experimental reproducibility"),
        )
}
