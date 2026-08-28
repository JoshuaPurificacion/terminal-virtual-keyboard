mod config;
mod input;
mod keyboard;
mod pty;
mod terminal;

use config::Config;
use crossterm::event::{self, Event};
use crossterm::terminal::size;
use input::{InputEvent, PcKeyboardInput};
use keyboard::{Direction, KeyAction, KeyboardState};
use pty::PtySession;
use std::time::Duration;
use terminal::{Renderer, TerminalGuard};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Setup terminal raw mode and safety drop guard
    let _guard = TerminalGuard::new()?;

    // 2. Load configuration
    let cfg = Config::load();

    // 3. Initialize terminal dimensions & renderer
    let (mut cols, mut rows) = size()?;
    let mut renderer = Renderer::new();

    // 4. Initialize keyboard state & input mapper
    let mut kb = KeyboardState::new();
    let mut input_mapper = PcKeyboardInput::new();

    // Calculate shell dimensions (reserve lines for keyboard + status + legend)
    let kb_rows = kb.get_layout().len() as u16 + 2;
    let shell_rows = if rows > kb_rows + 2 {
        rows - kb_rows
    } else {
        rows.saturating_sub(kb_rows)
    };

    // 5. Spawn PTY session with real shell (bash)
    let mut pty = PtySession::spawn(cfg.shell.clone(), shell_rows.max(1), cols.max(1))?;

    // 6. Main Event Loop
    let mut running = true;
    while running {
        // Poll for inputs with a short timeout to maintain smooth rendering
        if event::poll(Duration::from_millis(25))? {
            match event::read()? {
                Event::Resize(new_cols, new_rows) => {
                    cols = new_cols;
                    rows = new_rows;
                    let current_kb_rows = kb.get_layout().len() as u16 + 2;
                    let new_shell_rows = if rows > current_kb_rows + 2 {
                        rows - current_kb_rows
                    } else {
                        rows.saturating_sub(current_kb_rows)
                    };
                    let _ = pty.resize(new_shell_rows.max(1), cols.max(1));
                }
                Event::Mouse(mouse_event) => {
                    if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse_event.kind {
                        let current_kb_rows = kb.get_layout().len() as u16;
                        let status_bar_height = 2u16;
                        let kb_total_height = current_kb_rows + status_bar_height;
                        let shell_height = if rows > kb_total_height + 2 {
                            rows - kb_total_height
                        } else {
                            rows.saturating_sub(current_kb_rows + 1)
                        };
                        let kb_start_row = shell_height + 1;

                        if let Some((r, c)) = kb.hit_test(mouse_event.column, mouse_event.row, kb_start_row) {
                            if let Some(bytes) = kb.tap_key(r, c) {
                                let _ = pty.write(&bytes);
                            }
                        }
                    }
                }
                Event::Key(key_event) => {
                    if let Some(event) = input_mapper.map_key(key_event) {
                        match event {
                            InputEvent::Up => kb.move_cursor(Direction::Up),
                            InputEvent::Down => kb.move_cursor(Direction::Down),
                            InputEvent::Left => kb.move_cursor(Direction::Left),
                            InputEvent::Right => kb.move_cursor(Direction::Right),

                            InputEvent::Select => {
                                if let Some(bytes) = kb.press_select() {
                                    let _ = pty.write(&bytes);
                                }
                            }
                            InputEvent::Back => {
                                if let Some(bytes) = kb.process_action(KeyAction::Backspace) {
                                    let _ = pty.write(&bytes);
                                }
                            }
                            InputEvent::ShiftKey => kb.toggle_shift(),
                            InputEvent::CtrlKey => kb.toggle_ctrl(),
                            InputEvent::L1 => kb.toggle_l1(),
                            InputEvent::R1 => kb.toggle_r1(),
                            InputEvent::Start => {
                                if let Some(bytes) = kb.process_action(KeyAction::Enter) {
                                    let _ = pty.write(&bytes);
                                }
                            }
                            InputEvent::Select2 => {
                                if let Some(bytes) = kb.process_action(KeyAction::Escape) {
                                    let _ = pty.write(&bytes);
                                }
                            }
                            InputEvent::Passthrough(bytes) => {
                                let _ = pty.write(&bytes);
                            }
                            InputEvent::ToggleMode => {
                                let new_mode = input_mapper.toggle_mode();
                                kb.last_output_desc = format!("Mode: {}", new_mode.name());
                            }
                            InputEvent::Quit => {
                                running = false;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Render current frame
        let mode_name = input_mapper.mode.name();
        pty.with_screen(|screen| {
            let _ = renderer.render_frame(screen, &kb, mode_name, cols, rows);
        });
    }

    Ok(())
}
