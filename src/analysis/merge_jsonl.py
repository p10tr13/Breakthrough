import argparse
import sys
from pathlib import Path

def merge_jsonl_files(input_paths, output_path):
    """
    Reads multiple JSONL files and writes their content into a single output file.
    """
    try:
        with open(output_path, 'w', encoding='utf-8') as outfile:
            for file_path in input_paths:
                if not file_path.exists():
                    print(f"Warning: File '{file_path}' not found. Skipping.")
                    continue
                
                print(f"Merging: {file_path}")
                with open(file_path, 'r', encoding='utf-8') as infile:
                    for line in infile:
                        # Ensure each line ends with a newline character for valid JSONL
                        if not line.endswith('\n'):
                            line += '\n'
                        outfile.write(line)
        
        print(f"\nSuccess! All files merged into: {output_path}")
        
    except Exception as e:
        print(f"Error during merging: {e}")
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(
        description="Merge multiple JSONL files into a single output file in the specified order."
    )
    
    parser.add_argument(
        "input_files", 
        nargs="+", 
        type=Path, 
        help="List of .jsonl files to merge, in the desired order."
    )
    
    parser.add_argument(
        "-o", "--output", 
        type=Path, 
        required=True, 
        help="Path to the resulting merged JSONL file."
    )

    args = parser.parse_args()

    if args.output.parent and not args.output.parent.exists():
        args.output.parent.mkdir(parents=True, exist_ok=True)

    merge_jsonl_files(args.input_files, args.output)

if __name__ == "__main__":
    main()