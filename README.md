# Breakthrough

Experiments with Monte Carlo Tree Search (MCTS) and its modifications applied to the game [Breakthrough](https://en.wikipedia.org/wiki/Breakthrough_(board_game)).

## Game Rules

Breakthrough is a two-player abstract strategy game played on a rectangular board (most commonly 8×8). Each player begins with two full rows of pieces positioned on their side of the board.

On each turn, a player moves a single piece one square forward. Moves can be made straight ahead or diagonally into an empty square, while captures are performed by moving one square diagonally forward onto an opponent’s piece. Pieces cannot move backward, and captures are optional.

The objective is to either:
- move one of your pieces to the opponent's back row, or
- capture all of your opponent's pieces.

Because all pieces move only forward and there is constant pressure toward the goal, games are decisive and do not result in draws.

## Running the project

```sh
cargo run --bin breakthrough -- .\src\resources\configs\minimax_vs_human.toml
```

## Project Structure

The repository is organized into two main components: a high-performance game engine written in Rust and a Python-based analysis layer for experimentation and evaluation.

```txt
Breakthrough/
├── docs/
│   └── ...
├── src/
│   ├── compute/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   └── ...
│   │   ├── Cargo.toml
│   │   └── ...
│   └── analysis/
│       ├── main.py
│       └── ...
└── README.md
```