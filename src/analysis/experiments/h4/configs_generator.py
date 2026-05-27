import copy
import tomli_w # type: ignore
import os
import shutil
from pathlib import Path

BOARD_WIDTH = 8
HEIGHTS = [5, 6, 7, 8, 9, 10, 11]
MAX_ITERATIONS_SCALE_FACTOR = 2000

def generate_board_size_configs(output_dir: str, agent_config_base: dict, name_prefix: str):
    if os.path.exists(output_dir):
        shutil.rmtree(output_dir)
    os.makedirs(output_dir, exist_ok=True)
    
    for height in HEIGHTS:
        iterations = height * MAX_ITERATIONS_SCALE_FACTOR
        
        agent_config = copy.deepcopy(agent_config_base)
        if agent_config["type"] == "Mcts":
            agent_config["max_iterations"] = iterations
            
        config = {
            "board_width": BOARD_WIDTH,
            "board_height": height,
            "white_player": agent_config,
            "black_player": agent_config
        }

        filename = os.path.join(output_dir, f"{name_prefix}_8x{height}.toml")
        with open(filename, "wb") as f:
            tomli_w.dump(config, f)

if __name__ == "__main__":
    SCRIPT_DIR = Path(__file__).parent.resolve()
    
    mcts_base = {"type": "Mcts"}
    generate_board_size_configs(str(SCRIPT_DIR / "mcts"), mcts_base, "mcts")
    
    rave_base = {
        "type": "Mcts",
        "use_rave": True,
    }
    generate_board_size_configs(str(SCRIPT_DIR / "rave"), rave_base, "rave")
    
    heavy_base = {
        "type": "Mcts",
        "use_heavy_playouts": True,
        "heavy_playouts_epsilon": 0.1
    }
    generate_board_size_configs(str(SCRIPT_DIR / "heavyplayouts"), heavy_base, "heavyplayouts")
