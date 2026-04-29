use std::path::PathBuf;

use miette::{IntoDiagnostic, Result};

use compute::{BreakthroughConfig, cli::build_command};

fn main() -> Result<()> {
    miette::set_panic_hook();

    let command = build_command();
    let matches = command.get_matches();

    let config_path = matches.get_one::<PathBuf>("config").unwrap();
    let config_content = std::fs::read_to_string(config_path).into_diagnostic()?;
    let mut config: BreakthroughConfig = toml::from_str(&config_content).into_diagnostic()?;

    if let Some(board_width) = matches.get_one::<u8>("board-width") {
        config.board_width = *board_width;
    }

    if let Some(board_height) = matches.get_one::<u8>("board-height") {
        config.board_height = *board_height;
    }
    if let Some(seed) = matches.get_one::<u64>("seed") {
        config.seed = Some(*seed);
    }

    dbg!(config);

    Ok(())
}
