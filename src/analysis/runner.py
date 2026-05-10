import argparse
import subprocess
import sys
from pathlib import Path

def main():
    parser = argparse.ArgumentParser(description="Automate running experiments for the Breakthrough game.")
    
    parser.add_argument("config_dir", type=Path, help="Directory containing .toml configuration files")
    parser.add_argument("--runs", type=int, default=10, help="Number of runs per configuration file")
    parser.add_argument("--bin", type=str, help="Path to the Rust executable")
    parser.add_argument("--base-seed", type=int, default=42, help="Initial seed for the first run")

    parser.add_argument("--white-output", type=str, 
                        help="Path to the output JSONL file for the white player. Supports {config} and {seed} formatting.")
    parser.add_argument("--black-output", type=str, 
                        help="Path to the output JSONL file for the black player. Supports {config} and {seed} formatting.")
    parser.add_argument("--board-width", type=int, help="Width of the game board")
    parser.add_argument("--board-height", type=int, help="Height of the game board")

    args = parser.parse_args()

    if not args.config_dir.is_dir():
        print(f"Error: Directory '{args.config_dir}' does not exist.")
        sys.exit(1)

    toml_files = list(args.config_dir.glob("*.toml"))
    if not toml_files:
        print(f"No .toml files found in directory '{args.config_dir}'.")
        sys.exit(0)

    print(f"Found {len(toml_files)} configuration files. Each will be run {args.runs} times.")

    for toml_file in toml_files:
        print(f"\n{'-'*40}\nProcessing configuration: {toml_file.name}\n{'-'*40}")
        
        for i in range(args.runs):
            current_seed = args.base_seed + i
            
            cmd = [args.bin, str(toml_file)]

            if args.white_output:
                white_out = args.white_output.format(config=toml_file.stem, seed=current_seed)
                cmd.extend(["--white-output", white_out])
            
            if args.black_output:
                black_out = args.black_output.format(config=toml_file.stem, seed=current_seed)
                cmd.extend(["--black-output", black_out])
                
            if args.board_width is not None:
                cmd.extend(["--board-width", str(args.board_width)])
                
            if args.board_height is not None:
                cmd.extend(["--board-height", str(args.board_height)])

            cmd.extend(["--seed", str(current_seed)])

            print(f"Run {i+1}/{args.runs} | Seed: {current_seed}")
            print(f"Command: {' '.join(cmd)}")

            try:
                subprocess.run(cmd, check=True)
            except subprocess.CalledProcessError as e:
                print(f"Error running iteration {i+1} for {toml_file.name}: Exited with code {e.returncode}")
            except FileNotFoundError:
                print(f"Error: Executable '{args.bin}' not found. Make sure the project is built (e.g., cargo build --release).")
                sys.exit(1)

if __name__ == "__main__":
    main()