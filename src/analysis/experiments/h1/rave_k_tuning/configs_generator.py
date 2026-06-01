import argparse
import shutil
from pathlib import Path


BOARD_SIZES = [(7, 7), (8, 8), (6, 8), (8, 6)]
DEFAULT_RAVE_K_VALUES = [1, 3, 10, 30, 100, 300, 1000, 3000, 10000]
MCTS_ITERATIONS = 10000
EXPLORATION_CONSTANT = 1.41


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


def generate_configs(output_dir: Path, board_sizes: list[tuple[int, int]], rave_k_values: list[int]):
    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    uct_agent = {
        "type": "Mcts",
        "max_iterations": MCTS_ITERATIONS,
        "exploration_constant": EXPLORATION_CONSTANT,
    }

    for rave_k in rave_k_values:
        rave_agent = {
            "type": "Mcts",
            "max_iterations": MCTS_ITERATIONS,
            "exploration_constant": EXPLORATION_CONSTANT,
            "use_rave": True,
            "rave_k": rave_k,
        }

        for width, height in board_sizes:
            board_name = f"{width}x{height}"
            k_name = f"k_{rave_k}"
            write_config(
                output_dir,
                f"h1_tuning_{board_name}_{k_name}_rave_white_vs_uct_black",
                width,
                height,
                rave_agent,
                uct_agent,
            )
            write_config(
                output_dir,
                f"h1_tuning_{board_name}_{k_name}_uct_white_vs_rave_black",
                width,
                height,
                uct_agent,
                rave_agent,
            )


def parse_board_size(value: str) -> tuple[int, int]:
    width, height = value.lower().split("x", maxsplit=1)
    return int(width), int(height)


def main():
    parser = argparse.ArgumentParser(description="Generate RAVE k tuning configs for H1.")
    parser.add_argument(
        "-o",
        "--output-dir",
        type=Path,
        default=Path(__file__).parent / "configs",
        help="Directory where TOML configs will be written.",
    )
    parser.add_argument(
        "--board-size",
        action="append",
        type=parse_board_size,
        help="Board size to include, e.g. 8x8. Can be repeated. Defaults to all H1 board sizes.",
    )
    parser.add_argument(
        "--rave-k",
        action="append",
        type=int,
        help="RAVE k value to include. Can be repeated. Defaults to a coarse logarithmic grid.",
    )
    args = parser.parse_args()

    generate_configs(
        args.output_dir,
        args.board_size or BOARD_SIZES,
        args.rave_k or DEFAULT_RAVE_K_VALUES,
    )


if __name__ == "__main__":
    main()
