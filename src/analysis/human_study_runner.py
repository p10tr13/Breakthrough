import argparse
import csv
import random
import subprocess
import sys
from pathlib import Path


def parse_args():
    parser = argparse.ArgumentParser(
        description="Run human-vs-AI study configs in randomized order."
    )
    parser.add_argument(
        "--config-dir",
        type=Path,
        default=Path("src/analysis/experiments/human/configs"),
        help="Directory containing study TOML configs.",
    )
    parser.add_argument(
        "--pattern",
        default="study_*.toml",
        help="Glob pattern selecting configs from config-dir.",
    )
    parser.add_argument(
        "--bin",
        type=Path,
        default=Path("src/compute/target/release/breakthrough.exe"),
        help="Path to the breakthrough executable.",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("src/analysis/experiments/human/data"),
        help="Directory where session results and order files will be written.",
    )
    parser.add_argument(
        "--session-id",
        required=True,
        help="Anonymous participant/session id, e.g. p01.",
    )
    parser.add_argument(
        "--shuffle-seed",
        type=int,
        help="Optional seed controlling randomized config order.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print randomized order without launching the GUI.",
    )
    return parser.parse_args()


def write_order_file(order_path: Path, configs: list[Path]):
    with order_path.open("w", encoding="utf-8", newline="") as file:
        writer = csv.writer(file)
        writer.writerow(["order", "config"])
        for index, config in enumerate(configs, start=1):
            writer.writerow([index, config.name])


def main():
    args = parse_args()

    if not args.config_dir.is_dir():
        print(f"Error: config directory does not exist: {args.config_dir}", file=sys.stderr)
        return 1

    configs = sorted(args.config_dir.glob(args.pattern))
    if not configs:
        print(
            f"Error: no configs matching {args.pattern!r} in {args.config_dir}",
            file=sys.stderr,
        )
        return 1

    if args.shuffle_seed is not None:
        rng = random.Random(args.shuffle_seed)
    else:
        rng = random.Random()
    rng.shuffle(configs)

    args.output_dir.mkdir(parents=True, exist_ok=True)
    results_path = args.output_dir / f"{args.session_id}_results.jsonl"
    order_path = args.output_dir / f"{args.session_id}_order.csv"
    write_order_file(order_path, configs)

    print(f"Session: {args.session_id}")
    print(f"Results: {results_path}")
    print(f"Order: {order_path}")
    print()

    for index, config in enumerate(configs, start=1):
        print(f"[{index}/{len(configs)}] {config.name}")

        command = [
            str(args.bin),
            str(config),
            "--white-output",
            str(results_path),
            "--black-output",
            str(results_path),
        ]

        if args.dry_run:
            print("  " + " ".join(command))
            continue

        if not args.bin.is_file():
            print(f"Error: executable does not exist: {args.bin}", file=sys.stderr)
            return 1

        completed = subprocess.run(command, check=False)
        if completed.returncode != 0:
            print(
                f"Error: config {config.name} exited with code {completed.returncode}",
                file=sys.stderr,
            )
            return completed.returncode

        print("  closed, continuing to next config")

    print("\nHuman study session finished.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
