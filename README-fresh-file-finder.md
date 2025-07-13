# Fresh File Finder

A GUI tool for analyzing Hades route predictions by reverse engineering RNG seeds from observed button presses.

## Overview

Fresh File Finder provides an intuitive graphical interface for the Hades routefinder's reverse RNG functionality. It allows users to input observed button press ranges and automatically determines the original RNG seed, then generates route predictions based on that seed.

## Building and Running

```bash
# Build the GUI
cargo build --release

# Run the GUI
cargo run --release --bin fresh-file-finder
```

## How It Works

1. **Button Press Input**: Click buttons representing different Hades game actions (like "Well", "Sisyphus", "Charon", etc.)
2. **Range Configuration**: Each button represents a specific RNG range used in the game
3. **Reverse Engineering**: The tool uses the button press sequence to reverse engineer the original RNG seed
4. **Route Analysis**: Once the seed is found, it runs route prediction scripts to show upcoming chambers and rewards

## Requirements

- Hades save file (`.sav`)
- Hades Scripts directory (from game installation)
- Route analysis Lua script (e.g., `FreshFilePredict.lua`)

## Technical Details

The GUI leverages the same RNG reverse engineering engine as the command-line tool, providing a user-friendly interface for:
- Inputting observed random events as button presses
- Automatically generating reverse-RNG input files
- Running SIMD-optimized seed search (with scalar fallback)
- Executing route analysis with the discovered seed

This tool is particularly useful for analyzing fresh file runs in Hades speedrunning and optimization scenarios.