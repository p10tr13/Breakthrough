use std::{path::PathBuf, time::Instant};

use miette::{IntoDiagnostic, Result, miette};

use compute::{
    BreakthroughConfig,
    agents::{
        AgentConfig, AgentRuntimeStats, AgentStatsAccumulator, AgentType, CommonMetrics,
        HumanMetrics, MctsMetrics, MinimaxMetrics, append_record_to_jsonl,
    },
    cli::build_command,
    core::{Board, BoardConfig, Player},
};

fn main() -> Result<()> {
    miette::set_panic_hook();

    let command = build_command();
    let matches = command.get_matches();

    let white_output = matches.get_one::<PathBuf>("white-output").unwrap();
    let black_output = matches.get_one::<PathBuf>("black-output").unwrap();

    let config_path = matches.get_one::<PathBuf>("config").unwrap();
    let config_content = std::fs::read_to_string(config_path).into_diagnostic()?;
    let mut config: BreakthroughConfig = toml::from_str(&config_content).into_diagnostic()?;

    if config.white_player.is_human() || config.black_player.is_human() {
        return Err(miette!("Tournament mode does not support human players"));
    }

    if let Some(board_width) = matches.get_one::<u8>("board-width") {
        config.board_width = *board_width;
    }

    if let Some(board_height) = matches.get_one::<u8>("board-height") {
        config.board_height = *board_height;
    }
    if let Some(seed) = matches.get_one::<u64>("seed") {
        config.seed = Some(*seed);
    }

    let board_config = BoardConfig::new(config.board_width, config.board_height);
    let mut board = Board::new(config.board_width, config.board_height);

    let mut white_agent = config.white_player.create_agent(config.seed.unwrap_or(42));
    let mut black_agent = config.black_player.create_agent(config.seed.unwrap_or(42));

    let mut white_stats = AgentRuntimeStats::new(&Into::<AgentType>::into(&config.white_player));
    let mut black_stats = AgentRuntimeStats::new(&Into::<AgentType>::into(&config.black_player));

    println!(
        "Running tournament between {:?} and {:?} on a {}x{} board with seed {:?}",
        Into::<AgentType>::into(&config.white_player),
        Into::<AgentType>::into(&config.black_player),
        config.board_width,
        config.board_height,
        config.seed
    );

    let mut turn_timer = Instant::now();

    loop {
        let status = board.get_status(&board_config);
        if !status.is_ongoing() {
            println!("Simulation finished: {status:?}");
            break;
        }

        let is_white_turn = board.turn.is_white();

        let (ply, stats) = if is_white_turn {
            let ply = white_agent.select_ply(&board, &board_config);
            let stats = white_agent.take_stats();
            (ply, stats)
        } else {
            let ply = black_agent.select_ply(&board, &board_config);
            let stats = black_agent.take_stats();
            (ply, stats)
        };

        let ply =
            ply.expect("Agent did not return a move, despite the game still being in progress!");
        let time_taken = turn_timer.elapsed().as_millis();
        let ply_str = ply.encode(board_config.width);

        println!(" {:?}: {ply_str} ({time_taken} ms)", board.turn);

        if is_white_turn {
            white_stats.record_move(ply_str, time_taken, stats);
        } else {
            black_stats.record_move(ply_str, time_taken, stats);
        }

        board.apply_ply(ply);

        turn_timer = Instant::now();
    }

    let status = board.get_status(&board_config);
    let is_white_won = status.is_white_won();
    let white_pieces = board.get_piece_count(Player::White) as u8;
    let black_pieces = board.get_piece_count(Player::Black) as u8;

    let white_common = CommonMetrics {
        board_width: board_config.width,
        board_height: board_config.height,
        seed: config.seed,
        agent_type: (&config.white_player).into(),
        agent_color: Player::White,
        agent_won: is_white_won,
        pieces_remaining: white_pieces,
        move_times_ms: white_stats.move_times_ms,
        total_moves: white_stats.total_moves,
        moves: white_stats.move_history,
        opponent_type: (&config.black_player).into(),
        opponent_pieces_remaining: black_pieces,
    };

    match &white_stats.stats_accumulator {
        AgentStatsAccumulator::Minimax(acc) => {
            let AgentConfig::Minimax {
                max_depth,
                material_weight,
                advancement_weight,
                defended_weight,
                edge_penalty_weight,
            } = config.white_player.clone()
            else {
                unreachable!();
            };
            let metrics = MinimaxMetrics {
                common: white_common,
                max_depth,
                total_nodes_evaluated: acc.total_nodes,
                total_cutoffs: acc.total_cutoffs,
                material_weight,
                advancement_weight,
                defended_weight,
                edge_penalty_weight,
            };
            append_record_to_jsonl(&metrics, white_output).into_diagnostic()?;
        }
        AgentStatsAccumulator::Mcts(acc) => {
            let AgentConfig::Mcts {
                max_iterations,
                max_time_ms,
                exploration_constant,
            } = config.white_player.clone()
            else {
                unreachable!();
            };
            let metrics = MctsMetrics {
                common: white_common,
                max_iterations,
                max_time_ms,
                exploration_constant,
                total_iterations: acc.total_iterations,
                total_nodes_created: acc.total_nodes_created,
            };
            append_record_to_jsonl(&metrics, white_output).into_diagnostic()?;
        }
        AgentStatsAccumulator::None => {
            let metrics = HumanMetrics {
                common: white_common,
            };
            append_record_to_jsonl(&metrics, white_output).into_diagnostic()?;
        }
    }

    let black_common = CommonMetrics {
        board_width: board_config.width,
        board_height: board_config.height,
        seed: config.seed,
        agent_type: (&config.black_player).into(),
        agent_color: Player::Black,
        agent_won: !is_white_won,
        pieces_remaining: black_pieces,
        move_times_ms: black_stats.move_times_ms,
        total_moves: black_stats.total_moves,
        moves: black_stats.move_history,
        opponent_type: (&config.white_player).into(),
        opponent_pieces_remaining: white_pieces,
    };

    match &black_stats.stats_accumulator {
        AgentStatsAccumulator::Minimax(acc) => {
            let AgentConfig::Minimax {
                max_depth,
                material_weight,
                advancement_weight,
                defended_weight,
                edge_penalty_weight,
            } = config.black_player.clone()
            else {
                unreachable!();
            };
            let metrics = MinimaxMetrics {
                common: black_common,
                max_depth,
                total_nodes_evaluated: acc.total_nodes,
                total_cutoffs: acc.total_cutoffs,
                material_weight,
                advancement_weight,
                defended_weight,
                edge_penalty_weight,
            };
            append_record_to_jsonl(&metrics, black_output).into_diagnostic()?;
        }
        AgentStatsAccumulator::Mcts(acc) => {
            let AgentConfig::Mcts {
                max_iterations,
                max_time_ms,
                exploration_constant,
            } = config.black_player.clone()
            else {
                unreachable!();
            };
            let metrics = MctsMetrics {
                common: black_common,
                max_iterations,
                max_time_ms,
                exploration_constant,
                total_iterations: acc.total_iterations,
                total_nodes_created: acc.total_nodes_created,
            };
            append_record_to_jsonl(&metrics, black_output).into_diagnostic()?;
        }
        AgentStatsAccumulator::None => {
            let metrics = HumanMetrics {
                common: black_common,
            };
            append_record_to_jsonl(&metrics, black_output).into_diagnostic()?;
        }
    }

    println!("Metrics recorded to {white_output:?} and {black_output:?}");

    Ok(())
}
