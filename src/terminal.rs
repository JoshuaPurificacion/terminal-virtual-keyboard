use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{self, BufWriter, Stdout, Write};
use std::panic;

use crate::keyboard::{KeyboardState, Layer};

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        if let Err(e) = enable_raw_mode() {
            eprintln!("[TerminalGuard] enable_raw_mode note: {}", e);
        }
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            EnterAlternateScreen,
            Hide,
            Clear(ClearType::All),
            EnableMouseCapture,
            Print("\x1b[?7l") // Disable auto-wrap on right margin
        );

        // Panic hook to restore terminal if anything goes wrong
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let _ = disable_raw_mode();
            let mut stdout = io::stdout();
            let _ = execute!(
                stdout,
                Print("\x1b[?7h"), // Re-enable auto-wrap
                DisableMouseCapture,
                LeaveAlternateScreen,
                Show
            );
            default_hook(info);
        }));

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            Print("\x1b[?7h"), // Re-enable auto-wrap
            DisableMouseCapture,
            LeaveAlternateScreen,
            Show
        );
    }
}

pub struct Renderer {
    stdout: BufWriter<Stdout>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            stdout: BufWriter::with_capacity(16384, io::stdout()),
        }
    }

    pub fn render_frame(
        &mut self,
        screen: &vt100::Screen,
        kb: &KeyboardState,
        input_mode_name: &str,
        cols: u16,
        rows: u16,
    ) -> io::Result<()> {
        let max_render_cols = cols.saturating_sub(1).max(1);

        if !kb.visible {
            // Keyboard is hidden: render 100% full screen shell area
            self.render_shell_area(screen, max_render_cols, rows)?;
            self.stdout.flush()?;
            return Ok(());
        }

        let kb_layout = kb.get_layout();
        let kb_rows = kb_layout.len() as u16;
        let status_bar_height = 2u16; // 1 separator + 1 help legend
        let kb_total_height = kb_rows + status_bar_height;

        let shell_height = if rows > kb_total_height + 2 {
            rows - kb_total_height
        } else {
            rows.saturating_sub(kb_rows + 1)
        };

        // 1. Render Shell Output
        self.render_shell_area(screen, max_render_cols, shell_height)?;

        // 2. Render Separator & Status Line
        let status_row = shell_height;
        self.render_status_bar(kb, input_mode_name, max_render_cols, status_row)?;

        // 3. Render Virtual Keyboard Grid
        let kb_start_row = status_row + 1;
        self.render_keyboard_grid(kb, max_render_cols, kb_start_row)?;

        // 4. Render Bottom Legend
        let legend_row = kb_start_row + kb_rows;
        if legend_row < rows {
            self.render_legend(max_render_cols, legend_row)?;
        }

        self.stdout.flush()?;
        Ok(())
    }

    fn render_shell_area(
        &mut self,
        screen: &vt100::Screen,
        cols: u16,
        shell_height: u16,
    ) -> io::Result<()> {
        let (cursor_row, cursor_col) = screen.cursor_position();
        let hide_cursor = screen.hide_cursor();
        let rendered_rows: Vec<String> = screen.rows(0, cols).collect();

        for r in 0..shell_height {
            execute!(self.stdout, MoveTo(0, r), Clear(ClearType::CurrentLine))?;
            let r_usize = r as usize;
            if r_usize < rendered_rows.len() {
                let row_str = &rendered_rows[r_usize];
                let chars: Vec<char> = row_str.chars().collect();
                let mut line_buf = String::with_capacity(cols as usize);

                for c in 0..cols as usize {
                    if c < chars.len() {
                        line_buf.push(chars[c]);
                    } else {
                        line_buf.push(' ');
                    }
                }

                // Render cursor if on this row
                if !hide_cursor && r == cursor_row && (cursor_col as usize) < line_buf.len() {
                    let before: String = line_buf.chars().take(cursor_col as usize).collect();
                    let cur_char = line_buf.chars().nth(cursor_col as usize).unwrap_or(' ');
                    let after: String = line_buf.chars().skip((cursor_col + 1) as usize).collect();

                    execute!(
                        self.stdout,
                        ResetColor,
                        Print(&before),
                        SetBackgroundColor(Color::White),
                        SetForegroundColor(Color::Black),
                        Print(cur_char),
                        ResetColor,
                        Print(&after)
                    )?;
                } else {
                    execute!(self.stdout, ResetColor, Print(&line_buf))?;
                }
            }
        }
        Ok(())
    }

    fn render_status_bar(
        &mut self,
        kb: &KeyboardState,
        input_mode_name: &str,
        cols: u16,
        row: u16,
    ) -> io::Result<()> {
        execute!(self.stdout, MoveTo(0, row), Clear(ClearType::CurrentLine))?;

        let layer_color = match kb.active_layer {
            Layer::Base => Color::Green,
            Layer::Numbers => Color::Yellow,
            Layer::Symbols => Color::Magenta,
        };

        let shift_status = if kb.shift_active { "[SHIFT: ON]" } else { "[SHIFT]" };
        let ctrl_status = if kb.ctrl_active { "[CTRL: ON]" } else { "[CTRL]" };

        let left_info = format!(" ⌨ cyberdeck-kb ─ [{}] ─ Layer: ", input_mode_name);
        let right_info = format!(" ─ {} ─ {} ─ Last: {} ", shift_status, ctrl_status, kb.last_output_desc);

        let used_len = left_info.len() + kb.active_layer.name().len() + right_info.len();
        let fill_len = if (cols as usize) > used_len {
            (cols as usize) - used_len
        } else {
            0
        };
        let fill = "─".repeat(fill_len);

        execute!(
            self.stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            Print(&left_info),
            SetForegroundColor(layer_color),
            Print(kb.active_layer.name()),
            SetForegroundColor(Color::White),
            Print(&right_info),
            Print(&fill),
            ResetColor
        )?;
        Ok(())
    }

    fn render_keyboard_grid(&mut self, kb: &KeyboardState, _cols: u16, start_row: u16) -> io::Result<()> {
        let layout = kb.get_layout();

        for (r_idx, row) in layout.iter().enumerate() {
            let cur_row = start_row + r_idx as u16;
            execute!(self.stdout, MoveTo(0, cur_row), Clear(ClearType::CurrentLine), ResetColor)?;

            let item_count = row.len();

            // Render keys
            for (c_idx, key) in row.iter().enumerate() {
                let is_selected = kb.cursor_row == r_idx && kb.cursor_col == c_idx;

                let key_label = format!(" {} ", key.label);

                if is_selected {
                    execute!(
                        self.stdout,
                        SetBackgroundColor(Color::Cyan),
                        SetForegroundColor(Color::Black),
                        Print("▶"),
                        Print(&key_label),
                        Print("◀"),
                        ResetColor
                    )?;
                } else {
                    execute!(
                        self.stdout,
                        SetBackgroundColor(Color::DarkGrey),
                        SetForegroundColor(Color::White),
                        Print("["),
                        Print(&key_label),
                        Print("]"),
                        ResetColor
                    )?;
                }

                // Spacing between keys
                if c_idx + 1 < item_count {
                    execute!(self.stdout, Print(" "))?;
                }
            }
        }
        Ok(())
    }

    fn render_legend(&mut self, _cols: u16, row: u16) -> io::Result<()> {
        execute!(self.stdout, MoveTo(0, row), Clear(ClearType::CurrentLine))?;
        let legend = " [F / F6]: Show/Hide KB  [Touch]: Tap Keys  [Direct]: Type  [D-Pad]: Nav  [F9]: Mode  [Ctrl+Q]: Exit";

        execute!(
            self.stdout,
            SetBackgroundColor(Color::Black),
            SetForegroundColor(Color::DarkYellow),
            Print(legend),
            ResetColor
        )?;
        Ok(())
    }
}
