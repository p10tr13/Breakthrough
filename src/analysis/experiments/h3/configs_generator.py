import copy
import tomli_w # type: ignore
import os
import shutil
from pathlib import Path
from itertools import product

BOARD_WIDTH = 6
BOARD_HEIGHT = 7
MCTS_ITERATIONS = 10000
DEPTHS = [1, 2, 3, 4, 5, 6, 7]
COLORS = ["black", "white"]

def generate_configs(output_dir: str, mcts_config_base: dict, name_prefix: str):
    if os.path.exists(output_dir):
        shutil.rmtree(output_dir)
    os.makedirs(output_dir, exist_ok=True)
    
    for depth, minimax_color in product(DEPTHS, COLORS):
        minimax_config = {"type": "Minimax", "max_depth": depth}
        mcts_config = copy.deepcopy(mcts_config_base)
        
        if minimax_color == "black":
            config = {
                "board_width": BOARD_WIDTH,
                "board_height": BOARD_HEIGHT,
                "white_player": mcts_config,
                "black_player": minimax_config
            }
        else:
            config = {
                "board_width": BOARD_WIDTH,
                "board_height": BOARD_HEIGHT,
                "white_player": minimax_config,
                "black_player": mcts_config
            }

        filename = os.path.join(output_dir, f"{name_prefix}_{minimax_color}_depth_{depth}.toml")
        with open(filename, "wb") as f:
            tomli_w.dump(config, f)

if __name__ == "__main__":
    SCRIPT_DIR = Path(__file__).parent.resolve()
    
    mcts_base = {"type": "Mcts", "max_iterations": MCTS_ITERATIONS}
    generate_configs(str(SCRIPT_DIR / "mcts"), mcts_base, "minimax")
    
    rave_base = {
        "type": "Mcts", 
        "max_iterations": MCTS_ITERATIONS,
        "use_rave": True,
    }
    generate_configs(str(SCRIPT_DIR / "rave"), rave_base, "minimax")
    
    heavy_base = {
        "type": "Mcts", 
        "max_iterations": MCTS_ITERATIONS,
        "use_heavy_playouts": True,
    }
    generate_configs(str(SCRIPT_DIR / "heavyplayouts"), heavy_base, "minimax")