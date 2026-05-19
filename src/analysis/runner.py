import argparse
import subprocess
import sys
from pathlib import Path

def run_experiments(config_dir, runs, bin_path, base_seed=42, 
                    white_output=None, black_output=None, 
                    board_width=None, board_height=None):
    config_dir = Path(config_dir)
    
    if not config_dir.is_dir():
        print(f"Error: Directory '{config_dir}' does not exist.")
        sys.exit(1)

    toml_files = list(config_dir.glob("*.toml"))
    if not toml_files:
        print(f"No .toml files found in directory '{config_dir}'.")
        sys.exit(0)

    print(f"Found {len(toml_files)} configuration files. Each will be run {runs} times.")

    for toml_file in toml_files:
        print(f"\n{'-'*40}\nProcessing configuration: {toml_file.name}\n{'-'*40}")
        
        for i in range(runs):
            current_seed = base_seed + i
            cmd = [bin_path, str(toml_file)]

            if white_output:
                white_out = white_output.format(config=toml_file.stem, seed=current_seed)
                cmd.extend(["--white-output", white_out])
            
            if black_output:
                black_out = black_output.format(config=toml_file.stem, seed=current_seed)
                cmd.extend(["--black-output", black_out])
                
            if board_width is not None:
                cmd.extend(["--board-width", str(board_width)])
                
            if board_height is not None:
                cmd.extend(["--board-height", str(board_height)])

            cmd.extend(["--seed", str(current_seed)])

            print(f"Run {i+1}/{runs} | Seed: {current_seed}")
            
            try:
                subprocess.run(cmd, check=True)
            except subprocess.CalledProcessError as e:
                print(f"Error running iteration {i+1} for {toml_file.name}: Exited with code {e.returncode}")
            except FileNotFoundError:
                print(f"Error: Executable '{bin_path}' not found.")
                sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description="Automate running experiments for the Breakthrough game.")
    parser.add_argument("config_dir", type=Path, help="Directory containing .toml configuration files")
    parser.add_argument("--runs", type=int, default=10, help="Number of runs per configuration file")
    parser.add_argument("--bin", type=str, required=True, help="Path to the Rust executable")
    parser.add_argument("--base-seed", type=int, default=42, help="Initial seed for the first run")
    parser.add_argument("--white-output", type=str)
    parser.add_argument("--black-output", type=str)
    parser.add_argument("--board-width", type=int)
    parser.add_argument("--board-height", type=int)

    args = parser.parse_args()
    
    run_experiments(
        config_dir=args.config_dir,
        runs=args.runs,
        bin_path=args.bin,
        base_seed=args.base_seed,
        white_output=args.white_output,
        black_output=args.black_output,
        board_width=args.board_width,
        board_height=args.board_height
    )

if __name__ == "__main__":
    main()