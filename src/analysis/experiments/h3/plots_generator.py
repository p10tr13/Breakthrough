import json
import argparse
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from pathlib import Path

def load_all_data(white_filepath: Path, black_filepath: Path):
    game_records = []
    move_records = []
    
    mcts_name = "Mcts"
    minimax_name = "Minimax"
    
    with open(white_filepath, 'r') as fw, open(black_filepath, 'r') as fb:
        for _, (line_white, line_black) in enumerate(zip(fw, fb), start=1):
            if not line_white.strip() or not line_black.strip():
                continue
                
            data_white = json.loads(line_white)
            data_black = json.loads(line_black)
            
            if data_white['agent_type'] == 'Minimax':
                minimax_data, minimax_color = data_white, 'White'
                mcts_data, mcts_color = data_black, 'Black'
            else:
                minimax_data, minimax_color = data_black, 'Black'
                mcts_data, mcts_color = data_white, 'White'
                
            minimax_name = minimax_data['agent_type']
            mcts_name = mcts_data['agent_type']
            depth = minimax_data.get('max_depth')
            
            game_records.append({
                'minimax_depth': depth,
                'mcts_color': mcts_color,
                'minimax_color': minimax_color,
                'mcts_won': int(mcts_data['agent_won']),
                'minimax_won': int(minimax_data['agent_won']),
                'total_moves': mcts_data['total_moves']
            })
            
            for i, time_ms in enumerate(mcts_data['move_times_ms']):
                move_records.append({
                    'Category': mcts_name,
                    'Agent Move Number': i + 1,
                    'Thinking Time (ms)': time_ms,
                    'Sort_Key': 0
                })
                
            for i, time_ms in enumerate(minimax_data['move_times_ms']):
                move_records.append({
                    'Category': f'{minimax_name} (głębokość = {depth})',
                    'Agent Move Number': i + 1,
                    'Thinking Time (ms)': time_ms,
                    'Sort_Key': depth
                })
                
    df_games = pd.DataFrame(game_records)
    df_moves = pd.DataFrame(move_records)
    
    return df_games, df_moves, mcts_name, minimax_name

def print_statistics(df: pd.DataFrame, mcts_name: str, minimax_name: str):
    print(f"\n{'-'*75}")
    print(f" TABLE 1: WIN RATE ({mcts_name} vs {minimax_name})")
    print(f"{'-'*75}")
    
    overall_win_stats = df.groupby('minimax_depth')['mcts_won'].mean() * 100
    color_win_stats = df.groupby(['minimax_depth', 'mcts_color'])['mcts_won'].mean() * 100
    
    col_w = f"{mcts_name} as White"
    col_b = f"{mcts_name} as Black"
    
    print(f"{'Depth':<8} | {col_w:<20} | {col_b:<20} | {'Overall Average':<18}")
    print("-" * 75)
    
    for depth in sorted(df['minimax_depth'].unique()):
        win_white = color_win_stats.get((depth, 'White'), 0.0)
        win_black = color_win_stats.get((depth, 'Black'), 0.0)
        win_overall = overall_win_stats.get(depth, 0.0)
        
        print(f"d={depth:<6} | {win_white:>19.1f}% | {win_black:>19.1f}% | {win_overall:>17.1f}%")

    print(f"\n{'-'*80}")
    print(f" TABLE 2: GAME LENGTH: Mean ± SD ({mcts_name} vs {minimax_name})")
    print(f"{'-'*80}")
    
    overall_moves = df.groupby('minimax_depth')['total_moves'].agg(['mean', 'std'])
    color_moves = df.groupby(['minimax_depth', 'mcts_color'])['total_moves'].agg(['mean', 'std'])
    
    print(f"{'Depth':<8} | {col_w:<22} | {col_b:<22} | {'Overall Average':<20}")
    print("-" * 80)
    
    def format_moves(stats_df, key):
        try:
            row = stats_df.loc[key]
            std_val = row['std']
            if pd.isna(std_val):
                std_val = 0.0
            return f"{row['mean']:.1f} ± {std_val:.1f}"
        except KeyError:
            return "N/A"
    
    for depth in sorted(df['minimax_depth'].unique()):
        moves_white = format_moves(color_moves, (depth, 'White'))
        moves_black = format_moves(color_moves, (depth, 'Black'))
        moves_overall = format_moves(overall_moves, depth)
        
        print(f"d={depth:<6} | {moves_white:>22} | {moves_black:>22} | {moves_overall:>19}")

def generate_move_time_plots(df_moves: pd.DataFrame, mcts_name: str, minimax_name: str):
    if df_moves.empty:
        print("Error: No move data found.")
        return

    sns.set_theme(style="whitegrid")
    df_moves = df_moves.sort_values(by=['Sort_Key'])
    
    plt.figure(figsize=(12, 7))
    
    depths = sorted([d for d in df_moves['Sort_Key'].unique() if d > 0])
    minimax_colors = sns.color_palette("Reds", n_colors=len(depths) + 2).as_hex()[2:] 
    
    palette = {f'{mcts_name}': '#1f77b4'}
    for depth, color in zip(depths, minimax_colors):
        palette[f'{minimax_name} (głębokość = {depth})'] = color
    
    sns.lineplot(
        data=df_moves, 
        x='Agent Move Number', 
        y='Thinking Time (ms)', 
        hue='Category',
        palette=palette, 
        marker='o',
        markersize=5,
        linewidth=2,
        errorbar=('ci', 95)
    )
    
    plt.xlabel('Numer ruchu', fontsize=12)
    plt.ylabel('Czas namysłu (ms)', fontsize=12)
    
    plt.xticks(sorted(df_moves['Agent Move Number'].unique()))
    plt.legend(title='Algorytm')
    plt.tight_layout()
    
    time_filename = f"{mcts_name.lower()}_thinking_time.png"
    plt.savefig(time_filename, dpi=300)
    plt.close()
    
    print(f" -> {time_filename}")

def generate_plots(df_games: pd.DataFrame, mcts_name: str):
    sns.set_theme(style="whitegrid")

    mcts_stats = df_games.groupby(['minimax_depth', 'mcts_color'])['mcts_won'].mean().reset_index()
    mcts_stats.rename(columns={'mcts_won': 'win_rate'}, inplace=True)
    
    mcts_stats['max_win'] = 1.0
    
    plt.figure(figsize=(8, 6))
    
    sns.barplot(
        data=mcts_stats, x='minimax_depth', y='max_win', hue='mcts_color',
        palette={'White': '#d62728', 'Black': '#1f77b4'}, dodge=True, alpha=0.3,
        legend=False, hue_order=['White', 'Black']
    )
    
    sns.barplot(
        data=mcts_stats, x='minimax_depth', y='win_rate', hue='mcts_color',
        palette={'White': '#d62728', 'Black': '#1f77b4'}, dodge=True,
        hue_order=['White', 'Black']
    )
    
    plt.xlabel('Głębokość przeszukiwania drzewa gry', fontsize=12)
    plt.ylabel('Współczynnik zwycięstw', fontsize=12)
    plt.ylim(0, 1.05)
    plt.legend(title=f'Kolor {mcts_name}')
    plt.tight_layout()
    mcts_filename = f"{mcts_name.lower()}_win_rate.png"
    plt.savefig(mcts_filename, dpi=300)
    plt.close()

    plt.figure(figsize=(8, 6))
    
    sns.boxplot(
        data=df_games, x='minimax_depth', y='total_moves', hue='mcts_color',
        palette={'White': '#d62728', 'Black': '#1f77b4'}, dodge=True,
        linewidth=1.5, fliersize=4, hue_order=['White', 'Black']
    )
    
    plt.xlabel('Głębokość przeszukiwania drzewa gry', fontsize=12)
    plt.ylabel('Całkowita liczba ruchów', fontsize=12)
    plt.legend(title=f'Kolor {mcts_name}')
    plt.tight_layout()
    
    moves_filename = f"{mcts_name.lower()}_total_moves.png"
    plt.savefig(moves_filename, dpi=300)
    plt.close()

    print("Plots saved:")
    print(f" -> {mcts_filename}")
    print(f" -> {moves_filename}")

def main():
    parser = argparse.ArgumentParser(description="Generate plots from game results.")
    parser.add_argument("--white", type=Path, required=True, help="Path to the white player JSONL file")
    parser.add_argument("--black", type=Path, required=True, help="Path to the black player JSONL file")
    
    args = parser.parse_args()
    
    if not args.white.exists() or not args.black.exists():
        print("Error: Input file(s) not found.")
        return

    df_games, df_moves, mcts_name, minimax_name = load_all_data(args.white, args.black)
    
    print(f"Processed {len(df_games)} games.")
    
    print_statistics(df_games, mcts_name, minimax_name)
    generate_plots(df_games, mcts_name)
    generate_move_time_plots(df_moves, mcts_name, minimax_name)

if __name__ == "__main__":
    main()