# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a standalone routing tool for the game Hades that simulates game-accurate RNG to analyze optimal routes and outcomes. The tool loads game files, mods, and save files to run Lua scripts in an environment that mirrors the actual game engine.

## Architecture

The project is a hybrid Rust/Lua application:

- **Rust backend** (`src/`): Handles save file parsing, RNG simulation, and provides a Lua runtime
- **Lua scripts**: Game logic simulation and route analysis algorithms
- **Engine.lua**: Provides game engine callback stubs and utilities
- **Utils/**: Shared Lua utilities for deep copying, JSON operations, matching, and route finding

### Key Components

- `src/main.rs`: Entry point that sets up Lua runtime, loads save files, and injects game-accurate RNG
- `src/rng.rs`: Implements Supergiant Games' PCG random number generator for accurate simulation
- `src/reverse_rng/`: RNG reverse engineering module for finding original seeds from observed outputs
- `src/save.rs`: Handles decompression and parsing of Hades save files
- `src/luabins.rs`: Lua binary serialization format parser
- `Utils/FindRoute.lua`: Core route finding logic and run simulation
- Various `Find*Route.lua` scripts: Specific route analysis for different scenarios (Beowulf, Charon, Styx, etc.)

## Common Commands

### Building and Running
```bash
# Build the project
cargo build

# Build optimized release version
cargo build --release

# Run a specific route analysis script
cargo run -- <script.lua> --save-file <save_file.sav> --scripts-dir <hades_scripts_dir>

# Example: Run fresh file prediction
cargo run -- FreshFilePredict.lua --save-file ./FreshFile.sav --scripts-dir ~/legendary/Hades/Content/Scripts/

# RNG reverse engineering from observed data points
cargo run -- reverse-rng data_points.txt

# Generate test data for RNG reverse engineering validation
cargo run --bin generate_test_data > test_data.txt
```

### Development Scripts
- `fresh.sh`: Runs fresh file prediction analysis
- `beo.sh`: Runs Beowulf route analysis with release build
- Shell scripts typically use `--scripts-dir ~/legendary/Hades/Content/Scripts/` pointing to the Hades game installation

### Testing
- Test files are in `test/` directory with `.test` extension
- Contains expected output for route finding algorithms
- Run tests by comparing script output against `.test` files
- Unit tests: `cargo test` for all tests, `cargo test reverse_rng` for RNG reverse engineering tests

## Environment Setup

Set `HADES_SCRIPTS_DIR` environment variable to your Hades Scripts directory to avoid passing it every time:
```bash
export HADES_SCRIPTS_DIR=~/legendary/Hades/Content/Scripts/
```

## Save File Format

The tool works with Hades save files (`.sav`) that contain:
- Compressed Lua state using LZ4
- Game progression data
- RNG seeds and states
- Room configurations and rewards

## Route Analysis

Route finding scripts simulate game progression by:
1. Loading save file state into Lua globals
2. Creating run instances with `CreateRun()`
3. Simulating room generation and reward selection
4. Analyzing optimal paths through chambers
5. Outputting detailed predictions for rooms, rewards, and upgrade options

Scripts typically output structured data showing:
- Chamber details (C1, C2, etc.)
- Enemy configurations
- Available exits and rewards
- Upgrade options and rerolls
- RNG seeds and usage counts

## RNG Reverse Engineering

The tool can reverse engineer original RNG seeds from observed random number outputs:

### Usage
```bash
# Reverse engineer from data points file
cargo run -- reverse-rng data_points.txt

# For better performance, use release build
cargo run --release -- reverse-rng data_points.txt
```

### Input Format
Data points file should be CSV format:
```
# Format: name,offset,min,max,observed
chamber1,0,0.0,100.0,52.42
chamber2,5,0.0,1.0,0.59
chamber3,10,-50.0,50.0,42.49
```

### Performance
- **Search Space**: 2^32 possible seeds (~4.3 billion)
- **Typical Runtime**: 30-90 seconds with release build
- **Recommended Data Points**: 6-7 for unique identification
- **Early Termination**: Stops when high-confidence match found

### Test Data Generation
Generate known test data for validation:
```bash
cargo run --bin generate_test_data > known_seed_test.txt
cargo run --release -- reverse-rng known_seed_test.txt
```

### Benchmarking
To track optimization performance:
```bash
# Run benchmark using real_ursa_data_fixed.txt (expected seed: 1152303697)
./bench_reverse_rng.sh

# Expected baseline: ~60 seconds with release build
```