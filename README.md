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

## Streamlined RG353M Control Scheme

| Hardware Button | Linux Code | Action |
|---|---|---|
| **`A`** | `304 (BTN_SOUTH)` | **Type / Select key under cursor** |
| **`B`** | `305 (BTN_EAST)` | **Backspace / Delete** |
| **`X`** | `307 (BTN_NORTH)` | **Shift Modifier** (One-shot uppercase & symbols) |
| **`Y`** | `308 (BTN_WEST)` | **Tab Key** (Bash autocomplete / indent) |
| **`L1`** | `310 (BTN_TL)` | **Ctrl Modifier** (`L1+C` = `^C`, `L1+B` = `tmux`, `L1+D` = EOF) |
| **`L2`** | `312 (BTN_TL2)` | **Space Key** |
| **`R1`** | `311 (BTN_TR)` | **Cycle Layers** (`BASE` ➔ `NUMS` ➔ `SYMS` ➔ `BASE`) |
| **`R2`** | `313 (BTN_TR2)` | **Enter / Execute** |
| **`SELECT`** | `314 (BTN_SELECT)` | **Escape Key** |
| **`START`** | `315 (BTN_START)` | **Enter Key** |
| **`F` Button** | `316 (BTN_MODE)` | **Toggle Virtual Keyboard (Show / Hide)** |
| **D-Pad** | `544-547` | Move Virtual Keyboard Cursor |
| **Right Stick** | `ABS_RY` | **Terminal Scroll Up / Down** (History / Pagers) |

---

## Building and Running

### 1. Run in VM / PC Testing
```bash
# Run unit tests and simulation
./scripts/test-vm-sim.sh

# Run interactive cyberdeck-kb
cargo run --release
```

### 2. DarkOS / ArkOS Cyberdeck Mode (RG353M)
```
EmulationStation (Default) ──[ SELECT + START ]──► Cyberdeck Terminal (cyberdeck-kb)
                         ◄─────[ Ctrl + Q ]───────
```
Install on RG353M without touching `emulationstation.sh`:
```bash
# 1. Clone repository on RG353M
git clone https://github.com/JoshuaPurificacion/terminal-virtual-keyboard.git
cd terminal-virtual-keyboard

# 2. Run DarkOS non-destructive installer
sudo ./scripts/install-darkos.sh

# 3. Enable background launcher service
sudo systemctl enable --now deck-launcher
```

---

## Architecture Overview

- **`src/main.rs`**: Main event loop, PTY dispatch, terminal sizing, and frame orchestration.
- **`src/bin/deck-launcher.rs`**: Background daemon watching `/dev/input/event4` (`retrogame_joypad`), suspending ES (`SIGSTOP`), launching cyberdeck-kb, and resuming ES (`SIGCONT`).
- **`src/terminal.rs`**: `crossterm`-based rendering, ANSI screen rendering, and panic-safe `TerminalGuard`.
- **`src/keyboard.rs`**: Grid definitions for Base, Numbers, and Symbols layers; modifier and cursor state machines.
- **`src/input.rs`**: Native Linux `evdev` event listener & PC keyboard mapping.
- **`src/pty.rs`**: `portable-pty` session manager with background asynchronous I/O and `vt100` virtual screen parser.
- **`src/config.rs`**: Configuration schema and TOML loader for customizable keyboard layout and ratios.
- **`scripts/install-darkos.sh`**: One-command systemd installer for DarkOS / ArkOS.
- **`scripts/test-vm-sim.sh`**: End-to-end VM test harness for builds, unit tests, and lifecycle signals.
