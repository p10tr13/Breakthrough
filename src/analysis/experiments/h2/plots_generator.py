import argparse
import json
import math
from pathlib import Path

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

from src.analysis.experiments.statistics import wilson_interval


THRESHOLD = 0.55
AGENT_HEAVY = "MCTS+HeavyPlayouts"
AGENT_UCT = "MCTS"


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


def agent_label(record: dict) -> str:
    if record.get("use_heavy_playouts") is True:
        return AGENT_HEAVY
    return AGENT_UCT


def binomial_tail_at_threshold(wins: int, total: int, threshold: float = THRESHOLD):
    if total == 0:
        return float("nan")

    return sum(
        math.comb(total, k) * (threshold**k) * ((1 - threshold) ** (total - k))
        for k in range(wins, total + 1)
    )


def load_h2_data(data_dir: Path):
    game_records = []
    move_records = []

    for config_name, white_file, black_file in pair_result_files(data_dir):
        for game_index, (white, black) in enumerate(
            zip(read_jsonl(white_file), read_jsonl(black_file)),
            start=1,
        ):
            records = [white, black]
            heavy = next(
                (record for record in records if record.get("use_heavy_playouts") is True),
                None,
            )
            uct = next(
                (record for record in records if record.get("use_heavy_playouts") is False),
                None,
            )

            if heavy is None or uct is None:
                print(f"Warning: skipping non-H2 game in {config_name}, line {game_index}")
                continue

            board_size = f"{heavy['board_width']}x{heavy['board_height']}"
            heavy_color = heavy["agent_color"]

            game_records.append(
                {
                    "config": config_name,
                    "seed": heavy.get("seed"),
                    "board_width": heavy["board_width"],
                    "board_height": heavy["board_height"],
                    "board_size": board_size,
                    "heavy_color": heavy_color,
                    "heavy_won": int(heavy["agent_won"]),
                    "uct_won": int(uct["agent_won"]),
                    "total_plies": heavy["total_moves"] + uct["total_moves"],
                    "heavy_moves": heavy["total_moves"],
                    "uct_moves": uct["total_moves"],
                    "heavy_total_iterations": heavy.get("total_iterations"),
                    "uct_total_iterations": uct.get("total_iterations"),
                    "heavy_mean_move_time_ms": pd.Series(heavy["move_times_ms"]).mean(),
                    "uct_mean_move_time_ms": pd.Series(uct["move_times_ms"]).mean(),
                }
            )

            for record in records:
                label = agent_label(record)
                for move_number, time_ms in enumerate(record["move_times_ms"], start=1):
                    move_records.append(
                        {
                            "board_size": board_size,
                            "agent": label,
                            "agent_color": record["agent_color"],
                            "move_number": move_number,
                            "thinking_time_ms": time_ms,
                        }
                    )

    return pd.DataFrame(game_records), pd.DataFrame(move_records)


def summarize_hypothesis(df_games: pd.DataFrame):
    rows = []
    groups = [("overall", "All boards", df_games)]

    for board_size, group in df_games.groupby("board_size", sort=True):
        groups.append(("board", board_size, group))

    for (board_size, color), group in df_games.groupby(["board_size", "heavy_color"], sort=True):
        groups.append(("board_color", f"{board_size}, {color}", group))

    for scope, label, group in groups:
        total = len(group)
        wins = int(group["heavy_won"].sum())
        win_rate = wins / total if total else float("nan")
        ci_low, ci_high = wilson_interval(wins, total)
        p_value = binomial_tail_at_threshold(wins, total)

        rows.append(
            {
                "scope": scope,
                "label": label,
                "wins": wins,
                "games": total,
                "win_rate": win_rate,
                "ci95_low": ci_low,
                "ci95_high": ci_high,
                "p_value_vs_55pct": p_value,
                "observed_above_55pct": win_rate > THRESHOLD,
                "ci95_low_above_55pct": ci_low > THRESHOLD,
                "mean_total_plies": group["total_plies"].mean(),
                "std_total_plies": group["total_plies"].std(),
                "mean_heavy_move_time_ms": group["heavy_mean_move_time_ms"].mean(),
                "mean_uct_move_time_ms": group["uct_mean_move_time_ms"].mean(),
                "mean_heavy_iterations": group["heavy_total_iterations"].mean(),
                "mean_uct_iterations": group["uct_total_iterations"].mean(),
            }
        )

    return pd.DataFrame(rows)


def print_summary(summary: pd.DataFrame):
    overall = summary[summary["scope"] == "overall"].iloc[0]
    print("\nH2: MCTS+HeavyPlayouts vs MCTS")
    print("-" * 80)
    print(
        f"Overall: {int(overall['wins'])}/{int(overall['games'])} wins = "
        f"{overall['win_rate']:.1%}; threshold > {THRESHOLD:.0%}; "
        f"95% CI [{overall['ci95_low']:.1%}, {overall['ci95_high']:.1%}]; "
        f"p-value vs 55% = {overall['p_value_vs_55pct']:.4f}"
    )

    board_rows = summary[summary["scope"] == "board"].copy()
    print("\nBy board size:")
    print(
        board_rows[
            [
                "label",
                "wins",
                "games",
                "win_rate",
                "ci95_low",
                "ci95_high",
                "mean_total_plies",
                "mean_heavy_iterations",
                "mean_uct_iterations",
            ]
        ].to_string(
            index=False,
            formatters={
                "win_rate": "{:.1%}".format,
                "ci95_low": "{:.1%}".format,
                "ci95_high": "{:.1%}".format,
                "mean_total_plies": "{:.1f}".format,
                "mean_heavy_iterations": "{:.0f}".format,
                "mean_uct_iterations": "{:.0f}".format,
            },
        )
    )


def generate_win_rate_plot(summary: pd.DataFrame, output_dir: Path):
    board_color = summary[summary["scope"] == "board_color"].copy()
    board_color["win_rate_pct"] = board_color["win_rate"] * 100
    board_color[["board_size", "heavy_color"]] = board_color["label"].str.extract(
        r"^([^,]+), (White|Black)$"
    )
    board_color["heavy_color_label"] = board_color["heavy_color"].map(
        {"White": "Białe", "Black": "Czarne"}
    )

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(9, 6))
    ax = sns.barplot(
        data=board_color,
        x="board_size",
        y="win_rate_pct",
        hue="heavy_color_label",
        palette={"Białe": "#d62728", "Czarne": "#1f77b4"},
        hue_order=["Białe", "Czarne"],
    )

    for container in ax.containers:
        ax.bar_label(container, fmt="%.1f", padding=3, fontsize=8)

    ax.axhline(THRESHOLD * 100, color="black", linestyle="--", linewidth=1, label="Próg 55%")
    ax.set_xlabel("Rozmiar planszy")
    ax.set_ylabel("Współczynnik zwycięstw (%)")
    ax.set_ylim(0, 100)
    ax.legend(title="Kolor", loc="lower left")
    plt.tight_layout()
    path = output_dir / "h2_heavyplayouts_win_rate_by_board_and_color.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_board_win_rate_plot(summary: pd.DataFrame, output_dir: Path):
    board = summary[summary["scope"] == "board"].copy()
    board["win_rate_pct"] = board["win_rate"] * 100
    board["ci_low_pct"] = board["ci95_low"] * 100
    board["ci_high_pct"] = board["ci95_high"] * 100

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(8, 5))
    ax = sns.barplot(data=board, x="label", y="win_rate_pct", color="#4c78a8")
    errors = [
        board["win_rate_pct"] - board["ci_low_pct"],
        board["ci_high_pct"] - board["win_rate_pct"],
    ]
    ax.errorbar(
        x=range(len(board)),
        y=board["win_rate_pct"],
        yerr=errors,
        fmt="none",
        color="black",
        capsize=4,
        linewidth=1,
    )
    ax.axhline(THRESHOLD * 100, color="black", linestyle="--", linewidth=1, label="Próg 55%")
    ax.set_xlabel("Rozmiar planszy")
    ax.set_ylabel("Współczynnik zwycięstw (%)")
    ax.set_ylim(0, 100)
    ax.legend(title="Próg")
    plt.tight_layout()
    path = output_dir / "h2_heavyplayouts_win_rate_by_board.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_game_length_plot(df_games: pd.DataFrame, output_dir: Path):
    df_games = df_games.copy()
    df_games["heavy_color_label"] = df_games["heavy_color"].map(
        {"White": "Białe", "Black": "Czarne"}
    )

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(8, 5))
    ax = sns.boxplot(
        data=df_games,
        x="board_size",
        y="total_plies",
        hue="heavy_color_label",
        palette={"Białe": "#d62728", "Czarne": "#1f77b4"},
        hue_order=["Białe", "Czarne"],
    )
    ax.set_xlabel("Rozmiar planszy")
    ax.set_ylabel("Długość partii (półruchy)")
    ax.legend(title="Kolor")
    plt.tight_layout()
    path = output_dir / "h2_game_length_by_board_and_color.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_thinking_time_plot(df_moves: pd.DataFrame, output_dir: Path):
    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(8, 5))
    ax = sns.boxplot(
        data=df_moves,
        x="board_size",
        y="thinking_time_ms",
        hue="agent",
        palette={AGENT_HEAVY: "#4c78a8", AGENT_UCT: "#f58518"},
        hue_order=[AGENT_HEAVY, AGENT_UCT],
        showfliers=False,
    )
    ax.set_xlabel("Rozmiar planszy")
    ax.set_ylabel("Czas namysłu na ruch (ms)")
    ax.legend(title="Agent")
    plt.tight_layout()
    path = output_dir / "h2_thinking_time_by_board.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_iterations_plot(summary: pd.DataFrame, output_dir: Path):
    board = summary[summary["scope"] == "board"].copy()
    plot_data = []
    for _, row in board.iterrows():
        plot_data.append(
            {
                "board_size": row["label"],
                "agent": AGENT_HEAVY,
                "mean_iterations": row["mean_heavy_iterations"],
            }
        )
        plot_data.append(
            {
                "board_size": row["label"],
                "agent": AGENT_UCT,
                "mean_iterations": row["mean_uct_iterations"],
            }
        )

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(8, 5))
    ax = sns.barplot(
        data=pd.DataFrame(plot_data),
        x="board_size",
        y="mean_iterations",
        hue="agent",
        palette={AGENT_HEAVY: "#4c78a8", AGENT_UCT: "#f58518"},
        hue_order=[AGENT_HEAVY, AGENT_UCT],
    )
    ax.set_xlabel("Rozmiar planszy")
    ax.set_ylabel("Średnia liczba iteracji na partię")
    ax.legend(title="Agent")
    plt.tight_layout()
    path = output_dir / "h2_iterations_by_board.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def main():
    parser = argparse.ArgumentParser(
        description="Verify H2 and generate plots for MCTS+HeavyPlayouts vs MCTS."
    )
    parser.add_argument(
        "--data-dir",
        type=Path,
        default=Path(__file__).parent / "data",
        help="Directory with H2 JSONL result files.",
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

    df_games, df_moves = load_h2_data(args.data_dir)
    if df_games.empty:
        raise SystemExit(f"No H2 data found in {args.data_dir}")

    summary = summarize_hypothesis(df_games)
    print_summary(summary)

    df_games.to_csv(args.output_dir / "h2_games.csv", index=False)
    df_moves.to_csv(args.output_dir / "h2_moves.csv", index=False)
    summary.to_csv(args.output_dir / "h2_summary.csv", index=False)

    generate_board_win_rate_plot(summary, args.output_dir)
    generate_win_rate_plot(summary, args.output_dir)
    generate_game_length_plot(df_games, args.output_dir)
    generate_thinking_time_plot(df_moves, args.output_dir)
    generate_iterations_plot(summary, args.output_dir)


if __name__ == "__main__":
    main()
