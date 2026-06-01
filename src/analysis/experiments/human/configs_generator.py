import shutil
from pathlib import Path


SEED = 42
MAX_TIME_MS = 3000
EXPLORATION_CONSTANT = 1.41
RAVE_K_VALUE = 100
HEAVY_PLAYOUTS_EPSILON = 0.1
MINIMAX_DEPTH = 8


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
        f"board_height = {board_height}\n"
        f"seed = {SEED}\n"
        "study_mode = true\n\n"
        "[white_player]\n"
        f"{format_agent(white)}\n\n"
        "[black_player]\n"
        f"{format_agent(black)}\n"
    )


def write_config(
    output_dir: Path,
    name: str,
    board_width: int,
    board_height: int,
    white: dict,
    black: dict,
):
    path = output_dir / f"{name}.toml"
    path.write_text(format_config(board_width, board_height, white, black), encoding="utf-8")


def generate_configs(output_dir: Path):
    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    human_agent = {"type": "Human"}
    agents = {
        "uct": {
            "type": "Mcts",
            "max_time_ms": MAX_TIME_MS,
            "exploration_constant": EXPLORATION_CONSTANT,
        },
        "rave": {
            "type": "Mcts",
            "max_time_ms": MAX_TIME_MS,
            "exploration_constant": EXPLORATION_CONSTANT,
            "use_rave": True,
            "rave_k": RAVE_K_VALUE,
        },
        "heavy": {
            "type": "Mcts",
            "max_time_ms": MAX_TIME_MS,
            "exploration_constant": EXPLORATION_CONSTANT,
            "use_heavy_playouts": True,
            "heavy_playouts_epsilon": HEAVY_PLAYOUTS_EPSILON,
        },
        "minimax": {
            "type": "Minimax",
            "max_depth": MINIMAX_DEPTH,
        },
    }

    for agent_name, agent in agents.items():
        write_config(
            output_dir,
            f"study_7x7_human_white_vs_{agent_name}",
            7,
            7,
            human_agent,
            agent,
        )
        write_config(
            output_dir,
            f"study_8x8_{agent_name}_white_vs_human",
            8,
            8,
            agent,
            human_agent,
        )


if __name__ == "__main__":
    generate_configs(Path(__file__).parent / "configs")
