import shutil
from pathlib import Path


BOARD_SIZES = [(7, 7), (8, 8), (6, 8), (8, 6)]
MCTS_ITERATIONS = 10000
EXPLORATION_CONSTANT = 1.41
RAVE_K_VALUE = 100


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
        "exploration_constant": EXPLORATION_CONSTANT,
    }
    rave_agent = {
        "type": "Mcts",
        "max_iterations": MCTS_ITERATIONS,
        "exploration_constant": EXPLORATION_CONSTANT,
        "use_rave": True,
        "rave_k": RAVE_K_VALUE,
    }

    for width, height in BOARD_SIZES:
        board_name = f"{width}x{height}"
        write_config(
            output_dir,
            f"h1_{board_name}_rave_white_vs_uct_black",
            width,
            height,
            rave_agent,
            uct_agent,
        )
        write_config(
            output_dir,
            f"h1_{board_name}_uct_white_vs_rave_black",
            width,
            height,
            uct_agent,
            rave_agent,
        )


if __name__ == "__main__":
    generate_configs(Path(__file__).parent / "configs")
