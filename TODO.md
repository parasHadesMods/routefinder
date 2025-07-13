# GUI Implementation Plan

## Overview
Implementation of a druid-based GUI for reverse RNG analysis and route finding. The GUI provides a simple interface for collecting button presses, generating reverse-rng input files, and displaying analysis results.

## Technical Architecture

### 1. Project Structure Changes
- Add new GUI module: `src/gui/mod.rs`
- Create GUI-specific submodules:
  - `src/gui/app.rs` - Main application state and event handling
  - `src/gui/widgets.rs` - Custom widget definitions if needed
  - `src/gui/ui.rs` - UI layout and composition
- Extend main.rs to support GUI mode via new CLI subcommand

### 2. Dependencies
Add to Cargo.toml:
```toml
druid = "0.8.3"
```

### 3. CLI Integration
Extend the existing Commands enum with:
```rust
Gui,
```

### 4. GUI State Management
Central AppState struct containing:
- `offset: u32` - Current offset counter (starts at 7)
- `button_history: Vec<ButtonPress>` - History of button presses
- `text_output: String` - Content for text display area
- `save_file_path: String` - Path to save file for route analysis (editable)
- `scripts_dir_path: String` - Path to Hades Scripts directory (editable)
- `script_file: String` - Script file to run (editable, defaults to "RouteFreshFile.lua")

ButtonPress struct:
- `name: String` - Button name (Top, High, Middle, Low, Bottom)
- `offset: u32` - Offset at time of press

### 5. UI Layout
**Main Window:**
- Top: Three editable text fields in a horizontal row:
  - Save File Path (defaults to "FreshFile.sav")
  - Scripts Directory (defaults to "~/legendary/Hades/Content/Scripts/")
  - Script File (defaults to "RouteFreshFile.lua")
- Bottom: Split horizontally into two panels
  - Left: Scrollable text display (read-only)
  - Right: Vertical column of 6 buttons

**Button Specifications:**
- Names: Top, High, Middle, Low, Bottom, Calculate
- Colors: Gray for first 5, Blue for Calculate
- Button mappings to ranges:
  - Bottom: (0, 4)
  - Low: (3, 7) 
  - Middle: (6, 10)
  - High: (9, 13)
  - Top: (12, 16)

### 6. Event Handling
**Position Buttons (Top/High/Middle/Low/Bottom):**
1. Store button name + current offset as ButtonPress
2. Increment offset by 1
3. Update text display with new offset: "Current offset: {offset}"

**Calculate Button:**
1. Generate reverse-rng input file from button history
2. Execute reverse-rng
3. Parse seed result from output
4. Execute route analysis using found seed
5. Display all output in text area

### 7. File Operations
**Reverse-RNG Input Generation:**
- Create temporary file with `/range` format entries
- Format: `/range <button-name> <offset> 200 <low> <high>`
- Example: `/range Bottom 7 200 0 4`

**Process Execution:**
- Reverse-RNG: `cargo +nightly run --release --features simd_nightly -- reverse-rng <temp_file>`
- Route Analysis: `cargo run --release -- run <script_file> --save-file <save_file_path> --scripts-dir <scripts_dir_path> --lua-var AthenaSeed=<seed> --lua-var AthenaOffset=<offset>`

### 8. Error Handling
- Validate file paths from text fields before execution
- Handle process execution failures gracefully
- Display error messages in text output area
- Fallback to scalar reverse-rng if SIMD fails
- Handle empty or invalid text field values

### 9. Implementation Steps
1. **Setup Phase:**
   - Add druid dependency to Cargo.toml
   - Create GUI module structure
   - Extend CLI with GUI subcommand

2. **Core GUI Phase:**
   - Implement AppState and ButtonPress structs with default values
   - Create main window layout with text fields at top and split panels below
   - Implement editable text fields for save file, scripts directory, and script file
   - Implement button widgets with correct styling
   - Add scrollable text display widget

3. **Event Logic Phase:**
   - Implement button press handlers for position buttons
   - Add offset increment logic and text updates
   - Create calculate button handler framework

4. **Process Integration Phase:**
   - Implement reverse-rng input file generation
   - Add process execution for reverse-rng command
   - Integrate route analysis execution
   - Parse and display results in text area

5. **Polish Phase:**
   - Add proper error handling and user feedback
   - Test with various button press sequences
   - Validate output formatting and readability

### 10. File Integration Points
- Reuse existing `handle_reverse_rng_command` logic for process execution
- Leverage existing `run_script` function for route analysis
- Use existing path validation and file reading utilities

### 11. Testing Strategy
- Unit tests for state management and button mapping logic
- Integration tests for file generation and process execution
- Manual testing with various button press sequences
- Validation against expected reverse-rng input format

## Key Technical Considerations
- Process execution must be non-blocking to prevent GUI freezing
- Text output should stream results as processes complete
- File cleanup for temporary reverse-rng input files
- Path resolution and validation for user-editable file paths
- Default values from routeFreshFile.sh should be pre-populated in text fields
