import copy
import tomli_w # type: ignore
import os
import shutil
from pathlib import Path
from itertools import product

CONFIG_MCTS_VS_MINIMAX_BLACK = {
    "board_width": 6,
    "board_height": 7,
    "white_player": {"type": "Mcts", "max_iterations": 10000},
    "black_player": {"type": "Minimax"} # max_depth added in a loop
}

CONFIG_MCTS_VS_MINIMAX_WHITE = {
    "board_width": 6,
    "board_height": 7,
    "white_player": {"type": "Minimax"}, # max_depth added in a loop
    "black_player": {"type": "Mcts", "max_iterations": 10000}
}

def generate_mcts_vs_minimax_configs(output_dir: str):
    if os.path.exists(output_dir):
        shutil.rmtree(output_dir)
    os.makedirs(output_dir, exist_ok=True)
    
    depths = [1, 2, 3, 4, 5, 6, 7]
    colors = ["black", "white"] 
    
    for depth, minimax_color in product(depths, colors):
        
        if minimax_color == "black":
            config = copy.deepcopy(CONFIG_MCTS_VS_MINIMAX_BLACK)
            config["black_player"]["max_depth"] = depth
        else:
            config = copy.deepcopy(CONFIG_MCTS_VS_MINIMAX_WHITE)
            config["white_player"]["max_depth"] = depth

        filename = os.path.join(output_dir, f"minimax_{minimax_color}_depth_{depth}.toml")
        with open(filename, "wb") as f:
            tomli_w.dump(config, f)
            
if __name__ == "__main__":
    SCRIPT_DIR = Path(__file__).parent.resolve()
    CONFIG_DIR = SCRIPT_DIR / "mcts"
    
    generate_mcts_vs_minimax_configs(str(CONFIG_DIR))