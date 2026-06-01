import argparse
import json
import re
from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

from src.analysis.experiments.statistics import wilson_interval


THRESHOLD = 0.55
AGENT_RAVE = "UCT+RAVE"
AGENT_UCT = "UCT"


def read_jsonl(path: Path):
    with path.open("r", encoding="utf-8") as file:
        for line in file:
            if line.strip():
                yield json.loads(line)


def pair_result_files(data_dir: Path):
    white_files = sorted(data_dir.glob("*_white.jsonl"))
    pairs = []

    for white_file in white_files:
        prefix = white_file.name.removesuffix("_white.jsonl")
        black_file = data_dir / f"{prefix}_black.jsonl"
        if black_file.exists():
            pairs.append((prefix, white_file, black_file))
        else:
            print(f"Warning: missing black file for {white_file.name}")

    return pairs


def extract_rave_k(config_name: str, record: dict) -> int:
    if record.get("rave_k") is not None:
        return int(record["rave_k"])

    match = re.search(r"_k_(\d+)_", config_name)
    if not match:
        raise ValueError(f"Cannot determine rave_k for {config_name}")
    return int(match.group(1))


def load_data(data_dir: Path):
    game_records = []
    move_records = []

    for config_name, white_file, black_file in pair_result_files(data_dir):
        for game_index, (white, black) in enumerate(
            zip(read_jsonl(white_file), read_jsonl(black_file)),
            start=1,
        ):
            records = [white, black]
            rave = next((record for record in records if record.get("use_rave") is True), None)
            uct = next((record for record in records if record.get("use_rave") is False), None)

            if rave is None or uct is None:
                print(f"Warning: skipping non-RAVE tuning game in {config_name}, line {game_index}")
                continue

            rave_k = extract_rave_k(config_name, rave)
            board_size = f"{rave['board_width']}x{rave['board_height']}"

            game_records.append(
                {
                    "config": config_name,
                    "seed": rave.get("seed"),
                    "rave_k": rave_k,
                    "board_width": rave["board_width"],
                    "board_height": rave["board_height"],
                    "board_size": board_size,
                    "rave_color": rave["agent_color"],
                    "rave_won": int(rave["agent_won"]),
                    "uct_won": int(uct["agent_won"]),
                    "total_plies": rave["total_moves"] + uct["total_moves"],
                    "rave_mean_move_time_ms": pd.Series(rave["move_times_ms"]).mean(),
                    "uct_mean_move_time_ms": pd.Series(uct["move_times_ms"]).mean(),
                    "rave_total_iterations": rave.get("total_iterations"),
                    "uct_total_iterations": uct.get("total_iterations"),
                }
            )

            for record in records:
                agent = AGENT_RAVE if record.get("use_rave") is True else AGENT_UCT
                for move_number, time_ms in enumerate(record["move_times_ms"], start=1):
                    move_records.append(
                        {
                            "rave_k": rave_k,
                            "board_size": board_size,
                            "agent": agent,
                            "agent_color": record["agent_color"],
                            "move_number": move_number,
                            "thinking_time_ms": time_ms,
                        }
                    )

    return pd.DataFrame(game_records), pd.DataFrame(move_records)


def summarize(df_games: pd.DataFrame):
    rows = []
    groups = [("overall", "All boards", ["rave_k"])]
    groups.append(("board", "Board size", ["rave_k", "board_size"]))
    groups.append(("board_color", "Board size and RAVE color", ["rave_k", "board_size", "rave_color"]))

    for scope, _, columns in groups:
        for key, group in df_games.groupby(columns, sort=True):
            if not isinstance(key, tuple):
                key = (key,)

            total = len(group)
            wins = int(group["rave_won"].sum())
            win_rate = wins / total if total else float("nan")
            ci_low, ci_high = wilson_interval(wins, total)

            row = {
                "scope": scope,
                "rave_k": key[0],
                "wins": wins,
                "games": total,
                "win_rate": win_rate,
                "ci95_low": ci_low,
                "ci95_high": ci_high,
                "observed_above_55pct": win_rate > THRESHOLD,
                "ci95_low_above_55pct": ci_low > THRESHOLD,
                "mean_total_plies": group["total_plies"].mean(),
                "mean_rave_move_time_ms": group["rave_mean_move_time_ms"].mean(),
                "mean_uct_move_time_ms": group["uct_mean_move_time_ms"].mean(),
            }

            if scope in {"board", "board_color"}:
                row["board_size"] = key[1]
            if scope == "board_color":
                row["rave_color"] = key[2]

            rows.append(row)

    return pd.DataFrame(rows)


def print_summary(summary: pd.DataFrame):
    overall = summary[summary["scope"] == "overall"].sort_values(
        ["win_rate", "games"], ascending=[False, False]
    )
    best = overall.iloc[0]

    print("\nRAVE k tuning for H1")
    print("-" * 80)
    print(
        f"Best observed k: {int(best['rave_k'])} "
        f"({int(best['wins'])}/{int(best['games'])} wins = {best['win_rate']:.1%}, "
        f"95% CI [{best['ci95_low']:.1%}, {best['ci95_high']:.1%}])"
    )

    print("\nOverall by k:")
    print(
        overall[
            ["rave_k", "wins", "games", "win_rate", "ci95_low", "ci95_high", "mean_total_plies"]
        ].to_string(
            index=False,
            formatters={
                "win_rate": "{:.1%}".format,
                "ci95_low": "{:.1%}".format,
                "ci95_high": "{:.1%}".format,
                "mean_total_plies": "{:.1f}".format,
            },
        )
    )


def generate_overall_plot(summary: pd.DataFrame, output_dir: Path):
    overall = summary[summary["scope"] == "overall"].copy().sort_values("rave_k")
    overall["win_rate_pct"] = overall["win_rate"] * 100
    overall["ci_low_pct"] = overall["ci95_low"] * 100
    overall["ci_high_pct"] = overall["ci95_high"] * 100

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(9, 5))
    ax = sns.lineplot(data=overall, x="rave_k", y="win_rate_pct", marker="o", linewidth=2)
    ax.errorbar(
        overall["rave_k"],
        overall["win_rate_pct"],
        yerr=[
            overall["win_rate_pct"] - overall["ci_low_pct"],
            overall["ci_high_pct"] - overall["win_rate_pct"],
        ],
        fmt="none",
        color="black",
        capsize=4,
        linewidth=1,
    )
    ax.axhline(THRESHOLD * 100, color="black", linestyle="--", linewidth=1)
    ax.set_xscale("log")
    ax.set_xlabel("RAVE k")
    ax.set_ylabel("UCT+RAVE win rate (%)")
    ax.set_ylim(0, 100)
    plt.tight_layout()
    path = output_dir / "h1_rave_k_overall_win_rate.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_board_plot(summary: pd.DataFrame, output_dir: Path):
    board = summary[summary["scope"] == "board"].copy().sort_values(["rave_k", "board_size"])
    board["win_rate_pct"] = board["win_rate"] * 100

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(10, 6))
    ax = sns.lineplot(
        data=board,
        x="rave_k",
        y="win_rate_pct",
        hue="board_size",
        marker="o",
        linewidth=2,
    )
    ax.axhline(THRESHOLD * 100, color="black", linestyle="--", linewidth=1)
    ax.set_xscale("log")
    ax.set_xlabel("RAVE k")
    ax.set_ylabel("UCT+RAVE win rate (%)")
    ax.set_ylim(0, 100)
    ax.legend(title="Board size")
    plt.tight_layout()
    path = output_dir / "h1_rave_k_win_rate_by_board.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_time_plot(summary: pd.DataFrame, output_dir: Path):
    overall = summary[summary["scope"] == "overall"].copy().sort_values("rave_k")
    plot_data = []
    for _, row in overall.iterrows():
        plot_data.append(
            {
                "rave_k": row["rave_k"],
                "agent": AGENT_RAVE,
                "mean_move_time_ms": row["mean_rave_move_time_ms"],
            }
        )
        plot_data.append(
            {
                "rave_k": row["rave_k"],
                "agent": AGENT_UCT,
                "mean_move_time_ms": row["mean_uct_move_time_ms"],
            }
        )

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(9, 5))
    ax = sns.lineplot(
        data=pd.DataFrame(plot_data),
        x="rave_k",
        y="mean_move_time_ms",
        hue="agent",
        marker="o",
        linewidth=2,
    )
    ax.set_xscale("log")
    ax.set_xlabel("RAVE k")
    ax.set_ylabel("Mean move time (ms)")
    ax.legend(title="Agent")
    plt.tight_layout()
    path = output_dir / "h1_rave_k_thinking_time.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def main():
    parser = argparse.ArgumentParser(description="Analyze RAVE k tuning results for H1.")
    parser.add_argument(
        "--data-dir",
        type=Path,
        default=Path(__file__).parent / "data",
        help="Directory with RAVE k tuning JSONL result files.",
    )
    parser.add_argument(
        "-o",
        "--output-dir",
        type=Path,
        default=Path(__file__).parent / "plots",
        help="Directory for plots and CSV summaries.",
    )
    args = parser.parse_args()

    args.output_dir.mkdir(parents=True, exist_ok=True)

    df_games, df_moves = load_data(args.data_dir)
    if df_games.empty:
        raise SystemExit(f"No RAVE k tuning data found in {args.data_dir}")

    summary = summarize(df_games)
    print_summary(summary)

    df_games.to_csv(args.output_dir / "h1_rave_k_games.csv", index=False)
    df_moves.to_csv(args.output_dir / "h1_rave_k_moves.csv", index=False)
    summary.to_csv(args.output_dir / "h1_rave_k_summary.csv", index=False)

    generate_overall_plot(summary, args.output_dir)
    generate_board_plot(summary, args.output_dir)
    generate_time_plot(summary, args.output_dir)


if __name__ == "__main__":
    main()
