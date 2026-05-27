import json
import argparse
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from pathlib import Path

def load_data(white_file: Path, black_file: Path):
    """
    Loads data from specified white and black result files.
    """
    game_records = []
    
    print(f"Loading data from:\n  White: {white_file}\n  Black: {black_file}")
    
    with open(white_file, 'r') as fw, open(black_file, 'r') as fb:
        for lw, lb in zip(fw, fb):
            if not lw.strip() or not lb.strip():
                continue
            dw = json.loads(lw)
            
            height = dw.get('board_height')
            agent_name = dw.get('agent_type', 'Mcts')
            
            category = agent_name
            if dw.get('use_rave'): 
                category = "Rave"
            elif dw.get('use_heavy_playouts'): 
                category = "HeavyPlayouts"
            
            if height is None: 
                continue
            
            white_won = int(dw['agent_won'])
            
            game_records.append({
                'height': height,
                'category': category,
                'white_won': white_won,
                'black_won': 1 - white_won,
                'total_moves': dw['total_moves']
            })
                
    return pd.DataFrame(game_records)

def print_statistics(df: pd.DataFrame):
    print(f"\n{'-'*95}")
    print(" TABLE: BOARD SIZE IMPACT (Width = 8)")
    print(f"{'-'*95}")
    
    stats = df.groupby(['category', 'height']).agg(
        white_wr=('white_won', 'mean'),
        black_wr=('black_won', 'mean'),
        moves_mean=('total_moves', 'mean'),
        moves_std=('total_moves', 'std')
    ).reset_index()
    
    print(f"{'Category':<10} | {'Height':<8} | {'White Win Rate':<16} | {'Black Win Rate':<16} | {'Game Length (Mean ± SD)':<22}")
    print("-" * 95)
    
    for _, row in stats.sort_values(['category', 'height']).iterrows():
        w_wr = f"{row['white_wr']*100:.1f}%"
        b_wr = f"{row['black_wr']*100:.1f}%"
        moves = f"{row['moves_mean']:.1f} ± {row['moves_std']:.1f}"
        print(f"{row['category']:<10} | {int(row['height']):<8} | {w_wr:<16} | {b_wr:<16} | {moves:<22}")

def generate_win_rate_plot(df: pd.DataFrame, output_dir: Path):
    if df.empty:
        print("No data to plot.")
        return

    sns.set_theme(style="whitegrid")
    categories = sorted(df['category'].unique())
    palette = {'White': '#d62728', 'Black': '#1f77b4'}
    
    for cat in categories:
        cat_df = df[df['category'] == cat]
        
        plt.figure(figsize=(10, 6))
        ax = plt.gca()
        
        plot_data = []
        for height in sorted(cat_df['height'].unique()):
            h_df = cat_df[cat_df['height'] == height]
            w_rate = h_df['white_won'].mean()
            b_rate = 1.0 - w_rate
            
            plot_data.append({'height': height, 'color': 'White', 'win_rate': w_rate, 'max_win': 1.0})
            plot_data.append({'height': height, 'color': 'Black', 'win_rate': b_rate, 'max_win': 1.0})
        
        pdf = pd.DataFrame(plot_data)
        
        sns.barplot(
            data=pdf, x='height', y='max_win', hue='color',
            palette=palette, dodge=True, alpha=0.3, ax=ax, legend=False,
            hue_order=['White', 'Black']
        )
        sns.barplot(
            data=pdf, x='height', y='win_rate', hue='color',
            palette=palette, dodge=True, ax=ax,
            hue_order=['White', 'Black']
        )
        
        white_stats = pdf[pdf['color'] == 'White'].sort_values('height')
        import numpy as np
        z = np.polyfit(white_stats['height'], white_stats['win_rate'], 2)
        p = np.poly1d(z)
        x_new = np.linspace(white_stats['height'].min(), white_stats['height'].max(), 100)
        ax.plot(x_new - white_stats['height'].min(), p(x_new), color='#d62728', linestyle='-', linewidth=2, label='Trend (White)', alpha=0.8)

        ax.set_ylabel('Współczynnik zwycięstw', fontsize=12)
        ax.set_ylim(0, 1.05)
        
        ax.axhline(0.5, ls='--', color='black', alpha=0.3, label='Fair play (0.5)')
        
        handles, labels = ax.get_legend_handles_labels()
        ax.legend(handles=handles, labels=labels, title=cat, loc='upper right')
        
        ax.set_xlabel('Wysokość planszy', fontsize=12)
        
        plt.tight_layout()
        
        filename = f"{cat.lower()}_board_size_win_rate.png"
        final_path = output_dir / filename
        plt.savefig(final_path, dpi=300)
        plt.close()
        print(f"Plot saved to: {final_path}")

def main():
    parser = argparse.ArgumentParser(description="Generate win rate plots for board size experiments.")
    parser.add_argument("--white", type=Path, required=True, help="Path to the white player JSONL file")
    parser.add_argument("--black", type=Path, required=True, help="Path to the black player JSONL file")
    parser.add_argument("-o", "--output-dir", type=Path, default=Path("."), help="Output directory for the plot")
    
    args = parser.parse_args()
    
    args.output_dir.mkdir(parents=True, exist_ok=True)
    
    if not args.white.exists() or not args.black.exists():
        print("Error: Input file(s) not found.")
        return

    df = load_data(args.white, args.black)
    
    if df.empty:
        print("Error: No valid data found in the provided files.")
        return
        
    print(f"Successfully loaded {len(df)} games.")
    print_statistics(df)
    generate_win_rate_plot(df, args.output_dir)

if __name__ == "__main__":
    main()
