# cyberdeck-kb — Terminal-Native Virtual Keyboard

A PTY-native virtual keyboard for the RG353M cyberdeck, prototyped on Linux and progressively hardened to real hardware. Proves that gamepad controls can drive a full terminal session through a custom TUI.

---

## Milestone Progress (M0–M7)

| Milestone | Description | Status |
|-----------|-------------|--------|
| **M0** | Linux Toolchain (rustc + cargo + gcc verified) | ✅ Completed |
| **M1** | Raw terminal UI + input loop (safe raw mode restoration) | ✅ Completed |
| **M2** | Input abstraction (`InputEvent` enum, PC keyboard to gamepad mapping) | ✅ Completed |
| **M3** | Virtual alphabet keyboard (QWERTY grid, cursor navigation, character selection) | ✅ Completed |
| **M4** | Essential keys (Space, Backspace, Enter, Esc) | ✅ Completed |
| **M5** | Modifier system (One-shot Shift, sticky Ctrl, Ctrl combos `^C`, `^D`, `^L`, etc.) | ✅ Completed |
| **M6** | Layers (Numbers `R1` and Symbols `L1` layers) | ✅ Completed |
| **M7** | PTY + real bash (Shell output rendered above keyboard; end-to-end interactive) | ✅ Completed |

---

## Control Mapping Reference

### Input Modes
- **DIRECT / HYBRID (Default)**: Type normally on any physical keyboard directly into the CLI shell, use `Arrow keys` to move cursor on the virtual keyboard, or tap/click directly on keys like an Android touch keyboard.
- **GAMEPAD ONLY**: Full RG353M gamepad key emulation (`J`/`Z` for A, `K` for B, `U`/`X` for X, etc.).
- **Toggle Mode**: Press `F9` or `Insert`.

### PC Testing Controls & RG353M Mapping

| Gamepad / Input | PC Keyboard Equivalent | Action |
|---|---|---|
| **Touch / Mouse Tap** | Mouse Click / Touchscreen | Directly type/trigger tapped key (Android-style) |
| **Physical Typing** | Direct keyboard keys | Types directly into the CLI shell (Hybrid mode) |
| **D-Pad Up/Down/Left/Right** | `Up` / `Down` / `Left` / `Right` | Move keyboard cursor |
| **A (Select)** | `Enter` / `Space` / `J` / `Z` | Select key under cursor |
| **B (Back)** | `Backspace` / `K` | Backspace / Delete character |
| **X (Shift)** | `F1` / `U` / `X` | Toggle Shift (One-shot) |
| **Y (Ctrl)** | `F2` / `I` / `Y` | Toggle Ctrl (Sticky) |
| **L1** | `F3` / `PageUp` / `[` | Toggle Symbols Layer |
| **R1** | `F4` / `PageDown` / `]` | Toggle Numbers Layer |
| **Start** | `Tab` / `F5` | Send Enter / Execute command |
| **Select** | `Esc` / `F12` | Send Escape |
| **Toggle Input Mode** | `F9` / `Insert` | Switch between Direct/Hybrid & Gamepad mode |
| **Quit** | `Ctrl + Q` / `F10` | Exit cyberdeck-kb cleanly |

---

## Building and Running

Ensure Rust and C build tools are installed:
```bash
# Build the project
cargo build --release

# Run cyberdeck-kb
cargo run
```

---

## Architecture Overview

- **`src/main.rs`**: Main event loop, PTY dispatch, terminal sizing, and frame orchestration.
- **`src/terminal.rs`**: `crossterm`-based rendering, ANSI screen rendering, and panic-safe `TerminalGuard`.
- **`src/keyboard.rs`**: Grid definitions for Base, Numbers, and Symbols layers; modifier and cursor state machines.
- **`src/input.rs`**: Input abstraction mapping physical events to gamepad-style `InputEvent`s.
- **`src/pty.rs`**: `portable-pty` session manager with background asynchronous I/O and `vt100` virtual screen parser.
- **`src/config.rs`**: Configuration schema and TOML loader for customizable keyboard layout and ratios.
