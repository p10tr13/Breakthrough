import shutil
from pathlib import Path


BOARD_SIZES = [(7, 7), (8, 8), (6, 8), (8, 6)]
MCTS_ITERATIONS = 10000
MAX_TIME_MS = 250
EXPLORATION_CONSTANT = 1.41
HEAVY_PLAYOUTS_EPSILON = 0.1


def format_agent(config: dict) -> str:
    lines = []
    for key, value in config.items():
        if isinstance(value, str):
            lines.append(f'{key} = "{value}"')
        elif isinstance(value, bool):
            lines.append(f"{key} = {str(value).lower()}")
        else:
            lines.append(f"{key} = {value}")
    return "\n".join(lines)


def format_config(board_width: int, board_height: int, white: dict, black: dict) -> str:
    return (
        f"board_width = {board_width}\n"
        f"board_height = {board_height}\n\n"
        "[white_player]\n"
        f"{format_agent(white)}\n\n"
        "[black_player]\n"
        f"{format_agent(black)}\n"
    )


def write_config(output_dir: Path, name: str, board_width: int, board_height: int, white: dict, black: dict):
    path = output_dir / f"{name}.toml"
    path.write_text(format_config(board_width, board_height, white, black), encoding="utf-8")


def generate_configs(output_dir: Path):
    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    uct_agent = {
        "type": "Mcts",
        "max_iterations": MCTS_ITERATIONS,
        "max_time_ms": MAX_TIME_MS,
        "exploration_constant": EXPLORATION_CONSTANT,
    }
    heavy_agent = {
        "type": "Mcts",
        "max_iterations": MCTS_ITERATIONS,
        "max_time_ms": MAX_TIME_MS,
        "exploration_constant": EXPLORATION_CONSTANT,
        "use_heavy_playouts": True,
        "heavy_playouts_epsilon": HEAVY_PLAYOUTS_EPSILON,
    }

    for width, height in BOARD_SIZES:
        board_name = f"{width}x{height}"
        write_config(
            output_dir,
            f"h2_{board_name}_heavy_white_vs_uct_black",
            width,
            height,
            heavy_agent,
            uct_agent,
        )
        write_config(
            output_dir,
            f"h2_{board_name}_uct_white_vs_heavy_black",
            width,
            height,
            uct_agent,
            heavy_agent,
        )


if __name__ == "__main__":
    generate_configs(Path(__file__).parent / "configs")
