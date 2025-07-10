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
```

### Development Scripts
- `fresh.sh`: Runs fresh file prediction analysis
- `beo.sh`: Runs Beowulf route analysis with release build
- Shell scripts typically use `--scripts-dir ~/legendary/Hades/Content/Scripts/` pointing to the Hades game installation

### Testing
- Test files are in `test/` directory with `.test` extension
- Contains expected output for route finding algorithms
- Run tests by comparing script output against `.test` files

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