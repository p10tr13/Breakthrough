import argparse
import csv
import json
import math
from pathlib import Path

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

from src.analysis.experiments.statistics import wilson_interval


AGENT_ORDER = ["MCTS", "MCTS+RAVE", "MCTS+HeavyPlayouts", "Minimax"]
BOARD_ORDER = ["7x7", "8x8"]
COLOR_LABELS = {"White": "Białe", "Black": "Czarne"}
ROLE_PALETTE = {"Człowiek": "#1f77b4", "AI": "#f58518"}
AGENT_PALETTE = {
    "MCTS": "#1f77b4",
    "MCTS+RAVE": "#f58518",
    "MCTS+HeavyPlayouts": "#2ca02c",
    "Minimax": "#d62728",
}
BOARD_PALETTE = {"7x7": "#1f77b4", "8x8": "#f58518"}
SURVEY_PALETTE = {"Ocena trudności": "#1f77b4", "Naturalność strategii": "#f58518"}


def read_jsonl(path: Path):
    with path.open("r", encoding="utf-8") as file:
        for line_number, line in enumerate(file, start=1):
            if line.strip():
                try:
                    yield json.loads(line)
                except json.JSONDecodeError as error:
                    print(f"Warning: skipping invalid JSON in {path.name}:{line_number}: {error}")


def read_order_and_survey(path: Path):
    if not path.exists():
        return {}, []

    order = {}
    survey_by_order = {}
    extra_survey_rows = []
    with path.open("r", encoding="utf-8", newline="") as file:
        reader = csv.reader(file)
        header = next(reader, [])
        for row_index, row in enumerate(reader, start=2):
            if not row or not any(cell.strip() for cell in row):
                continue

            first = row[0].strip()
            second = row[1].strip() if len(row) > 1 else ""
            if first.isdigit() and second.endswith(".toml"):
                order[int(first)] = second
                survey_values = {
                    header[column_index]: value
                    for column_index, value in enumerate(row)
                    if column_index < len(header)
                    and header[column_index] not in {"order", "config"}
                    and value.strip()
                }
                if survey_values:
                    survey_by_order[int(first)] = survey_values
                continue

            extra_survey_rows.append(
                {
                    "row_index": row_index,
                    **{
                        (header[column_index] if column_index < len(header) else f"column_{column_index + 1}"): value
                        for column_index, value in enumerate(row)
                    },
                }
            )

    return order, survey_by_order, extra_survey_rows


def agent_label(record: dict) -> str:
    if record.get("agent_type") == "Minimax":
        return "Minimax"
    if record.get("use_heavy_playouts") is True:
        return "MCTS+HeavyPlayouts"
    if record.get("use_rave") is True:
        return "MCTS+RAVE"
    if record.get("agent_type") == "Mcts":
        return "MCTS"
    return str(record.get("agent_type", "Unknown"))


def inferred_config(record_a: dict, record_b: dict) -> str:
    records = [record_a, record_b]
    human = next((record for record in records if record.get("agent_type") == "Human"), None)
    ai = next((record for record in records if record.get("agent_type") != "Human"), None)
    if human is None or ai is None:
        return "unknown"

    board = f"{ai['board_width']}x{ai['board_height']}"
    ai_key = {
        "MCTS": "uct",
        "MCTS+RAVE": "rave",
        "MCTS+HeavyPlayouts": "heavy",
        "Minimax": "minimax",
    }.get(agent_label(ai), "unknown")

    if human["agent_color"] == "White":
        return f"study_{board}_human_white_vs_{ai_key}.toml"
    return f"study_{board}_{ai_key}_white_vs_human.toml"


def mean_or_nan(values: list[float]) -> float:
    return float("nan") if not values else sum(values) / len(values)


def binomial_two_sided_p_value(wins: int, total: int, p0: float = 0.5) -> float:
    if total == 0:
        return float("nan")

    observed_probability = math.comb(total, wins) * (p0**wins) * ((1 - p0) ** (total - wins))
    return min(
        1.0,
        sum(
            math.comb(total, k) * (p0**k) * ((1 - p0) ** (total - k))
            for k in range(total + 1)
            if math.comb(total, k) * (p0**k) * ((1 - p0) ** (total - k))
            <= observed_probability + 1e-15
        ),
    )


def load_participant(participant_id: str, results_path: Path, order_path: Path):
    records = list(read_jsonl(results_path))
    order, survey_by_order, extra_survey_rows = read_order_and_survey(order_path)

    if len(records) % 2 != 0:
        print(f"Warning: {results_path.name} contains an odd number of records ({len(records)}).")

    expected_games = len(order)
    actual_games = len(records) // 2
    if expected_games and actual_games != expected_games:
        print(
            f"Warning: {participant_id} has {actual_games} complete games, "
            f"but {order_path.name} lists {expected_games}."
        )

    games = []
    turn_counts = []

    for pair_index in range(0, len(records) - 1, 2):
        game_number = pair_index // 2 + 1
        first, second = records[pair_index], records[pair_index + 1]
        game_records = [first, second]
        human = next((record for record in game_records if record.get("agent_type") == "Human"), None)
        ai = next((record for record in game_records if record.get("agent_type") != "Human"), None)

        if human is None or ai is None:
            print(f"Warning: skipping {participant_id} game {game_number}: missing Human or AI record.")
            continue

        planned_config = order.get(game_number)
        detected_config = inferred_config(first, second)
        if planned_config and planned_config != detected_config:
            print(
                f"Warning: {participant_id} game {game_number} order/config mismatch: "
                f"planned {planned_config}, detected {detected_config}."
            )

        survey = survey_by_order.get(game_number, {})
        board_size = f"{ai['board_width']}x{ai['board_height']}"
        ai_name = agent_label(ai)
        human_won = int(human["agent_won"])
        ai_won = int(ai["agent_won"])

        games.append(
            {
                "participant_id": participant_id,
                "game_number": game_number,
                "planned_config": planned_config,
                "detected_config": detected_config,
                "board_width": ai["board_width"],
                "board_height": ai["board_height"],
                "board_size": board_size,
                "human_color": human["agent_color"],
                "ai_color": ai["agent_color"],
                "ai_agent": ai_name,
                "human_won": human_won,
                "ai_won": ai_won,
                "total_plies": human["total_moves"] + ai["total_moves"],
                "human_moves": human["total_moves"],
                "ai_moves": ai["total_moves"],
                "human_pieces_remaining": human.get("pieces_remaining"),
                "ai_pieces_remaining": ai.get("pieces_remaining"),
                "ai_total_iterations": ai.get("total_iterations"),
                "ai_total_nodes_created": ai.get("total_nodes_created"),
                "minimax_nodes_evaluated": ai.get("total_nodes_evaluated"),
                "minimax_cutoffs": ai.get("total_cutoffs"),
                "difficulty_level": survey.get("difficulty_level"),
                "natural_strategy": survey.get("natural_strategy"),
            }
        )

        for role, record in [("Człowiek", human), ("AI", ai)]:
            turn_counts.append(
                {
                    "participant_id": participant_id,
                    "game_number": game_number,
                    "board_size": board_size,
                    "ai_agent": ai_name,
                    "role": role,
                    "agent_color": record["agent_color"],
                    "moves": record["total_moves"],
                }
            )

    for row in extra_survey_rows:
        row["participant_id"] = participant_id

    return games, turn_counts, extra_survey_rows


def load_human_data(data_dir: Path):
    game_rows = []
    turn_count_rows = []
    survey_rows = []

    for results_path in sorted(data_dir.glob("*_results.jsonl")):
        participant_id = results_path.name.removesuffix("_results.jsonl")
        order_path = data_dir / f"{participant_id}_order.csv"
        games, turn_counts, surveys = load_participant(participant_id, results_path, order_path)
        game_rows.extend(games)
        turn_count_rows.extend(turn_counts)
        survey_rows.extend(surveys)

    return pd.DataFrame(game_rows), pd.DataFrame(turn_count_rows), pd.DataFrame(survey_rows)


def summarize_groups(df_games: pd.DataFrame):
    rows = []
    groups = [("overall", "Wszystkie partie", df_games)]

    for agent, group in df_games.groupby("ai_agent", sort=False):
        groups.append(("agent", agent, group))

    for board_size, group in df_games.groupby("board_size", sort=True):
        groups.append(("board", board_size, group))

    for (agent, board_size), group in df_games.groupby(["ai_agent", "board_size"], sort=False):
        groups.append(("agent_board", f"{agent}, {board_size}", group))

    for (agent, color), group in df_games.groupby(["ai_agent", "human_color"], sort=False):
        groups.append(("agent_human_color", f"{agent}, {color}", group))

    for scope, label, group in groups:
        total = len(group)
        human_wins = int(group["human_won"].sum())
        ai_wins = int(group["ai_won"].sum())
        human_rate = human_wins / total if total else float("nan")
        ci_low, ci_high = wilson_interval(human_wins, total)

        rows.append(
            {
                "scope": scope,
                "label": label,
                "human_wins": human_wins,
                "ai_wins": ai_wins,
                "games": total,
                "human_win_rate": human_rate,
                "ai_win_rate": ai_wins / total if total else float("nan"),
                "ci95_low": ci_low,
                "ci95_high": ci_high,
                "p_value_vs_50pct": binomial_two_sided_p_value(human_wins, total),
                "mean_total_plies": group["total_plies"].mean(),
                "std_total_plies": group["total_plies"].std(),
                "mean_human_moves": group["human_moves"].mean(),
                "mean_ai_moves": group["ai_moves"].mean(),
                "mean_human_pieces_remaining": group["human_pieces_remaining"].mean(),
                "mean_ai_pieces_remaining": group["ai_pieces_remaining"].mean(),
            }
        )

    summary = pd.DataFrame(rows)
    return summary


def ordered_agents(series: pd.Series):
    present = set(series.dropna().unique())
    return [agent for agent in AGENT_ORDER if agent in present] + sorted(present - set(AGENT_ORDER))


def print_summary(summary: pd.DataFrame, df_games: pd.DataFrame):
    overall = summary[summary["scope"] == "overall"].iloc[0]
    print("\nHuman study: Human vs AI")
    print("-" * 80)
    print(
        f"Overall: humans {int(overall['human_wins'])}/{int(overall['games'])} wins = "
        f"{overall['human_win_rate']:.1%}; AI wins = {overall['ai_win_rate']:.1%}; "
        f"95% CI [{overall['ci95_low']:.1%}, {overall['ci95_high']:.1%}]"
    )
    print(
        f"Participants: {df_games['participant_id'].nunique()}, "
        f"games: {len(df_games)}, boards: {', '.join(sorted(df_games['board_size'].unique()))}"
    )

    agent_rows = summary[summary["scope"] == "agent"].copy()
    print("\nBy AI agent:")
    print(
        agent_rows[
            [
                "label",
                "human_wins",
                "ai_wins",
                "games",
                "human_win_rate",
                "ci95_low",
                "ci95_high",
                "mean_total_plies",
                "mean_human_moves",
                "mean_ai_moves",
                "mean_human_pieces_remaining",
                "mean_ai_pieces_remaining",
            ]
        ].to_string(
            index=False,
            formatters={
                "human_win_rate": "{:.1%}".format,
                "ci95_low": "{:.1%}".format,
                "ci95_high": "{:.1%}".format,
                "mean_total_plies": "{:.1f}".format,
                "mean_human_moves": "{:.1f}".format,
                "mean_ai_moves": "{:.1f}".format,
                "mean_human_pieces_remaining": "{:.1f}".format,
                "mean_ai_pieces_remaining": "{:.1f}".format,
            },
        )
    )


def generate_agent_win_rate_plot(summary: pd.DataFrame, output_dir: Path):
    data = summary[summary["scope"] == "agent"].copy()
    data["human_win_rate_pct"] = data["human_win_rate"] * 100
    data["ci_low_pct"] = data["ci95_low"] * 100
    data["ci_high_pct"] = data["ci95_high"] * 100
    order = ordered_agents(data["label"])

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(8, 5))
    ax = sns.barplot(
        data=data,
        x="label",
        y="human_win_rate_pct",
        order=order,
        palette=AGENT_PALETTE,
        hue="label",
        legend=False,
    )

    ordered = data.set_index("label").loc[order]
    errors = [
        ordered["human_win_rate_pct"] - ordered["ci_low_pct"],
        ordered["ci_high_pct"] - ordered["human_win_rate_pct"],
    ]
    ax.errorbar(
        x=range(len(ordered)),
        y=ordered["human_win_rate_pct"],
        yerr=errors,
        fmt="none",
        color="black",
        capsize=4,
        linewidth=1,
    )
    for container in ax.containers:
        if hasattr(container, "patches"):
            for patch in container.patches:
                value = patch.get_height()
                ax.annotate(
                    f"{value:.1f}",
                    xy=(patch.get_x() + patch.get_width() / 2, value),
                    xytext=(10, 8),
                    textcoords="offset points",
                    ha="left",
                    va="bottom",
                    fontsize=10,
                )

    ax.set_xlabel("Przeciwnik")
    ax.set_ylabel("Współczynnik zwycięstw człowieka (%)")
    ax.set_ylim(0, 100)
    plt.xticks(rotation=0, ha="center")
    plt.tight_layout()
    path = output_dir / "human_win_rate_by_agent.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_board_agent_plot(df_games: pd.DataFrame, output_dir: Path):
    data = (
        df_games.groupby(["board_size", "ai_agent"], observed=True)
        .agg(human_win_rate=("human_won", "mean"), games=("human_won", "size"))
        .reset_index()
    )
    data["human_win_rate_pct"] = data["human_win_rate"] * 100
    order = [board for board in BOARD_ORDER if board in set(data["board_size"])]
    hue_order = ordered_agents(data["ai_agent"])

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(9, 5))
    ax = sns.barplot(
        data=data,
        x="board_size",
        y="human_win_rate_pct",
        hue="ai_agent",
        order=order,
        hue_order=hue_order,
        palette=AGENT_PALETTE,
    )
    ax.set_xlabel("Rozmiar planszy")
    ax.set_ylabel("Współczynnik zwycięstw człowieka (%)")
    ax.set_ylim(0, 100)
    ax.legend(title="Przeciwnik")
    plt.tight_layout()
    path = output_dir / "human_win_rate_by_board_and_agent.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_game_length_plot(df_games: pd.DataFrame, output_dir: Path):
    order = ordered_agents(df_games["ai_agent"])
    hue_order = [board for board in BOARD_ORDER if board in set(df_games["board_size"])]

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(9, 5))
    ax = sns.boxplot(
        data=df_games,
        x="ai_agent",
        y="total_plies",
        hue="board_size",
        order=order,
        hue_order=hue_order,
        palette=BOARD_PALETTE,
    )
    ax.set_xlabel("Przeciwnik")
    ax.set_ylabel("Długość partii (półruchy)")
    ax.legend(title="Rozmiar planszy")
    plt.xticks(rotation=0, ha="center")
    plt.tight_layout()
    path = output_dir / "human_game_length_by_agent.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_turn_count_plot(df_turn_counts: pd.DataFrame, output_dir: Path):
    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(9, 5))
    ax = sns.boxplot(
        data=df_turn_counts,
        x="ai_agent",
        y="moves",
        hue="role",
        order=ordered_agents(df_turn_counts["ai_agent"]),
        hue_order=["Człowiek", "AI"],
        palette=ROLE_PALETTE,
    )
    ax.set_xlabel("Przeciwnik")
    ax.set_ylabel("Liczba wykonanych ruchów")
    ax.legend(title="Gracz")
    plt.xticks(rotation=0, ha="center")
    plt.tight_layout()
    path = output_dir / "human_moves_by_agent.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_participant_heatmap(df_games: pd.DataFrame, output_dir: Path):
    data = df_games.copy()
    data["matchup"] = (
        data["board_size"]
        + " "
        + data["human_color"].map(COLOR_LABELS)
        + " vs "
        + data["ai_agent"]
    )
    pivot = data.pivot_table(
        index="participant_id",
        columns="matchup",
        values="human_won",
        aggfunc="mean",
    )
    ordered_columns = sorted(
        pivot.columns,
        key=lambda value: (
            value.split(" ")[0],
            AGENT_ORDER.index(value.split(" vs ")[1])
            if value.split(" vs ")[1] in AGENT_ORDER
            else len(AGENT_ORDER),
            value,
        ),
    )
    pivot = pivot[ordered_columns]

    sns.set_theme(style="white")
    plt.figure(figsize=(12, 4.5))
    ax = sns.heatmap(
        pivot,
        cmap=sns.color_palette(["#e45756", "#54a24b"], as_cmap=True),
        vmin=0,
        vmax=1,
        linewidths=0.5,
        linecolor="white",
        cbar_kws={"ticks": [0, 1], "label": "Zwycięstwo człowieka"},
        annot=True,
        fmt=".0f",
    )
    ax.set_xlabel("Partia")
    ax.set_ylabel("Uczestnik")
    plt.xticks(rotation=35, ha="right")
    plt.tight_layout()
    path = output_dir / "human_participant_outcomes.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_pieces_plot(df_games: pd.DataFrame, output_dir: Path):
    plot_rows = []
    for _, row in df_games.iterrows():
        plot_rows.append(
            {
                "ai_agent": row["ai_agent"],
                "role": "Człowiek",
                "pieces_remaining": row["human_pieces_remaining"],
            }
        )
        plot_rows.append(
            {
                "ai_agent": row["ai_agent"],
                "role": "AI",
                "pieces_remaining": row["ai_pieces_remaining"],
            }
        )
    data = pd.DataFrame(plot_rows)

    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(9, 5))
    ax = sns.boxplot(
        data=data,
        x="ai_agent",
        y="pieces_remaining",
        hue="role",
        order=ordered_agents(data["ai_agent"]),
        hue_order=["Człowiek", "AI"],
        palette=ROLE_PALETTE,
    )
    ax.set_xlabel("Przeciwnik")
    ax.set_ylabel("Liczba pionów po zakończeniu")
    ax.legend(title="Gracz")
    plt.xticks(rotation=0, ha="center")
    plt.tight_layout()
    path = output_dir / "human_pieces_remaining_by_agent.png"
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def generate_survey_plot(df_games: pd.DataFrame, output_dir: Path):
    survey_columns = {
        "difficulty_level": "Ocena trudności",
        "natural_strategy": "Naturalność strategii",
    }
    present_columns = [
        column
        for column in survey_columns
        if column in df_games.columns and df_games[column].notna().any()
    ]
    if not present_columns:
        return

    data = df_games[["participant_id", "ai_agent", *present_columns]].copy()
    for column in present_columns:
        data[column] = pd.to_numeric(data[column], errors="coerce")

    long_data = data.melt(
        id_vars=["participant_id", "ai_agent"],
        value_vars=present_columns,
        var_name="metric",
        value_name="rating",
    ).dropna(subset=["rating"])
    long_data["metric"] = long_data["metric"].map(survey_columns)

    sns.set_theme(style="whitegrid")
    grid = sns.catplot(
        data=long_data,
        kind="bar",
        x="ai_agent",
        y="rating",
        hue="metric",
        order=ordered_agents(long_data["ai_agent"]),
        palette=SURVEY_PALETTE,
        height=5,
        aspect=1.8,
        errorbar=None,
    )
    grid.set_axis_labels("Przeciwnik", "Średnia ocena (1-5)")
    grid.set(ylim=(0, 5))
    sns.move_legend(
        grid,
        "upper left",
        bbox_to_anchor=(0.02, 0.98),
        ncol=1,
        title="Metryka ankietowa",
        frameon=True,
    )
    for ax in grid.axes.flat:
        ax.tick_params(axis="x", rotation=0)
        for container in ax.containers:
            if hasattr(container, "patches"):
                ax.bar_label(container, fmt="%.1f", padding=3, fontsize=8)

    path = output_dir / "human_survey_ratings_by_agent.png"
    plt.tight_layout()
    plt.savefig(path, dpi=300)
    plt.close()
    print(f"Saved plot: {path}")


def main():
    parser = argparse.ArgumentParser(description="Generate plots for the human-vs-AI study.")
    parser.add_argument(
        "--data-dir",
        type=Path,
        default=Path(__file__).parent / "data",
        help="Directory with participant order CSV and JSONL result files.",
    )
    parser.add_argument(
        "-o",
        "--output-dir",
        type=Path,
        default=Path(__file__).parent / "plots",
        help="Directory for generated plots and CSV summaries.",
    )
    args = parser.parse_args()

    args.output_dir.mkdir(parents=True, exist_ok=True)

    df_games, df_turn_counts, df_survey = load_human_data(args.data_dir)
    if df_games.empty:
        raise SystemExit(f"No human study data found in {args.data_dir}")

    summary = summarize_groups(df_games)
    print_summary(summary, df_games)

    df_games.to_csv(args.output_dir / "human_games.csv", index=False)
    df_turn_counts.to_csv(args.output_dir / "human_turn_counts.csv", index=False)
    summary.to_csv(args.output_dir / "human_summary.csv", index=False)
    if not df_survey.empty:
        df_survey.to_csv(args.output_dir / "human_survey_raw.csv", index=False)

    generate_agent_win_rate_plot(summary, args.output_dir)
    generate_board_agent_plot(df_games, args.output_dir)
    generate_game_length_plot(df_games, args.output_dir)
    generate_turn_count_plot(df_turn_counts, args.output_dir)
    generate_participant_heatmap(df_games, args.output_dir)
    generate_pieces_plot(df_games, args.output_dir)
    generate_survey_plot(df_games, args.output_dir)


if __name__ == "__main__":
    main()
