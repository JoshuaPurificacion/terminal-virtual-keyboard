#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Base,
    Numbers,
    Symbols,
}

impl Layer {
    pub fn name(&self) -> &'static str {
        match self {
            Layer::Base => "BASE",
            Layer::Numbers => "NUMS",
            Layer::Symbols => "SYMS",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    Char(char),
    Text(&'static str),
    Backspace,
    Enter,
    Space,
    Tab,
    Escape,
    ToggleShift,
    ToggleCtrl,
    SetLayer(Layer),
    Arrow(Direction),
    PageUp,
    PageDown,
    Home,
    End,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct KeyDef {
    pub label: String,
    pub action: KeyAction,
}

impl KeyDef {
    pub fn new_char(c: char) -> Self {
        Self {
            label: c.to_string(),
            action: KeyAction::Char(c),
        }
    }

    pub fn new_action(label: &str, action: KeyAction) -> Self {
        Self {
            label: label.to_string(),
            action,
        }
    }
}

pub struct KeyboardState {
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub active_layer: Layer,
    pub shift_active: bool, // One-shot
    pub ctrl_active: bool,  // Sticky
    pub visible: bool,      // On-screen keyboard visibility toggle
    pub last_output_desc: String,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            cursor_row: 1, // Start on 'q' or home row
            cursor_col: 0,
            active_layer: Layer::Base,
            shift_active: false,
            ctrl_active: false,
            visible: true,
            last_output_desc: "Ready".to_string(),
        }
    }
}

impl KeyboardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_layout(&self) -> Vec<Vec<KeyDef>> {
        match self.active_layer {
            Layer::Base => self.base_layout(),
            Layer::Numbers => self.numbers_layout(),
            Layer::Symbols => self.symbols_layout(),
        }
    }

    fn base_layout(&self) -> Vec<Vec<KeyDef>> {
        vec![
            // Row 0: Numbers & quick punctuation
            vec![
                KeyDef::new_char(if self.shift_active { '!' } else { '1' }),
                KeyDef::new_char(if self.shift_active { '@' } else { '2' }),
                KeyDef::new_char(if self.shift_active { '#' } else { '3' }),
                KeyDef::new_char(if self.shift_active { '$' } else { '4' }),
                KeyDef::new_char(if self.shift_active { '%' } else { '5' }),
                KeyDef::new_char(if self.shift_active { '^' } else { '6' }),
                KeyDef::new_char(if self.shift_active { '&' } else { '7' }),
                KeyDef::new_char(if self.shift_active { '*' } else { '8' }),
                KeyDef::new_char(if self.shift_active { '(' } else { '9' }),
                KeyDef::new_char(if self.shift_active { ')' } else { '0' }),
                KeyDef::new_char(if self.shift_active { '_' } else { '-' }),
                KeyDef::new_char(if self.shift_active { '+' } else { '=' }),
            ],
            // Row 1: QWERTY top row
            vec![
                self.char_key('q', 'Q'),
                self.char_key('w', 'W'),
                self.char_key('e', 'E'),
                self.char_key('r', 'R'),
                self.char_key('t', 'T'),
                self.char_key('y', 'Y'),
                self.char_key('u', 'U'),
                self.char_key('i', 'I'),
                self.char_key('o', 'O'),
                self.char_key('p', 'P'),
                KeyDef::new_char(if self.shift_active { '{' } else { '[' }),
                KeyDef::new_char(if self.shift_active { '}' } else { ']' }),
            ],
            // Row 2: Home row
            vec![
                self.char_key('a', 'A'),
                self.char_key('s', 'S'),
                self.char_key('d', 'D'),
                self.char_key('f', 'F'),
                self.char_key('g', 'G'),
                self.char_key('h', 'H'),
                self.char_key('j', 'J'),
                self.char_key('k', 'K'),
                self.char_key('l', 'L'),
                KeyDef::new_char(if self.shift_active { ':' } else { ';' }),
                KeyDef::new_char(if self.shift_active { '"' } else { '\'' }),
                KeyDef::new_char(if self.shift_active { '|' } else { '\\' }),
            ],
            // Row 3: Bottom row
            vec![
                self.char_key('z', 'Z'),
                self.char_key('x', 'X'),
                self.char_key('c', 'C'),
                self.char_key('v', 'V'),
                self.char_key('b', 'B'),
                self.char_key('n', 'N'),
                self.char_key('m', 'M'),
                KeyDef::new_char(if self.shift_active { '<' } else { ',' }),
                KeyDef::new_char(if self.shift_active { '>' } else { '.' }),
                KeyDef::new_char(if self.shift_active { '?' } else { '/' }),
                KeyDef::new_char(if self.shift_active { '~' } else { '`' }),
                KeyDef::new_char(if self.shift_active { '+' } else { '_' }),
            ],
        ]
    }

    fn numbers_layout(&self) -> Vec<Vec<KeyDef>> {
        vec![
            vec![
                KeyDef::new_char('1'), KeyDef::new_char('2'), KeyDef::new_char('3'),
                KeyDef::new_action(" ▲ ", KeyAction::Arrow(Direction::Up)),
                KeyDef::new_action("PgUp", KeyAction::PageUp),
                KeyDef::new_action("Home", KeyAction::Home),
                KeyDef::new_char('+'), KeyDef::new_char('-'), KeyDef::new_char('*'),
                KeyDef::new_char('/'), KeyDef::new_char('='), KeyDef::new_char('%'),
            ],
            vec![
                KeyDef::new_char('4'), KeyDef::new_char('5'), KeyDef::new_char('6'),
                KeyDef::new_action(" ▼ ", KeyAction::Arrow(Direction::Down)),
                KeyDef::new_action("PgDn", KeyAction::PageDown),
                KeyDef::new_action("End", KeyAction::End),
                KeyDef::new_char('('), KeyDef::new_char(')'), KeyDef::new_char('{'),
                KeyDef::new_char('}'), KeyDef::new_char('['), KeyDef::new_char(']'),
            ],
            vec![
                KeyDef::new_char('7'), KeyDef::new_char('8'), KeyDef::new_char('9'),
                KeyDef::new_action(" ◀ ", KeyAction::Arrow(Direction::Left)),
                KeyDef::new_action(" ▶ ", KeyAction::Arrow(Direction::Right)),
                KeyDef::new_char('_'), KeyDef::new_char('<'), KeyDef::new_char('>'),
                KeyDef::new_char('^'), KeyDef::new_char('&'), KeyDef::new_char('|'),
                KeyDef::new_char('~'),
            ],
            vec![
                KeyDef::new_char('0'), KeyDef::new_char('.'), KeyDef::new_char(','),
                KeyDef::new_char('!'), KeyDef::new_char('@'), KeyDef::new_char('#'),
                KeyDef::new_char('$'), KeyDef::new_char('\\'), KeyDef::new_char(':'),
                KeyDef::new_char(';'), KeyDef::new_char('"'), KeyDef::new_char('\''),
            ],
        ]
    }

    fn symbols_layout(&self) -> Vec<Vec<KeyDef>> {
        vec![
            vec![
                KeyDef::new_char('!'), KeyDef::new_char('@'), KeyDef::new_char('#'),
                KeyDef::new_char('$'), KeyDef::new_char('%'), KeyDef::new_char('^'),
                KeyDef::new_char('&'), KeyDef::new_char('*'), KeyDef::new_char('('),
                KeyDef::new_char(')'), KeyDef::new_char('_'), KeyDef::new_char('+'),
            ],
            vec![
                KeyDef::new_char('~'), KeyDef::new_char('`'), KeyDef::new_char('{'),
                KeyDef::new_char('}'), KeyDef::new_char('['), KeyDef::new_char(']'),
                KeyDef::new_char('|'), KeyDef::new_char('\\'), KeyDef::new_char(':'),
                KeyDef::new_char(';'), KeyDef::new_char('"'), KeyDef::new_char('\''),
            ],
            vec![
                KeyDef::new_char('<'), KeyDef::new_char('>'), KeyDef::new_char('?'),
                KeyDef::new_char('/'), KeyDef::new_char('='), KeyDef::new_char('-'),
                KeyDef::new_char('+'), KeyDef::new_char('*'), KeyDef::new_char('!'),
                KeyDef::new_char('@'), KeyDef::new_char('#'), KeyDef::new_char('$'),
            ],
            vec![
                KeyDef::new_char('0'), KeyDef::new_char('1'), KeyDef::new_char('2'),
                KeyDef::new_char('3'), KeyDef::new_char('4'), KeyDef::new_char('5'),
                KeyDef::new_char('6'), KeyDef::new_char('7'), KeyDef::new_char('8'),
                KeyDef::new_char('9'), KeyDef::new_char(','), KeyDef::new_char('.'),
            ],
        ]
    }

    fn char_key(&self, lower: char, upper: char) -> KeyDef {
        let ch = if self.shift_active { upper } else { lower };
        let mut label = ch.to_string();
        if self.ctrl_active {
            label = format!("^{}", upper);
        }
        KeyDef {
            label,
            action: KeyAction::Char(ch),
        }
    }

    pub fn move_cursor(&mut self, dir: Direction) {
        let layout = self.get_layout();
        let row_count = layout.len();
        if row_count == 0 {
            return;
        }

        match dir {
            Direction::Up => {
                if self.cursor_row == 0 {
                    self.cursor_row = row_count - 1;
                } else {
                    self.cursor_row -= 1;
                }
            }
            Direction::Down => {
                self.cursor_row = (self.cursor_row + 1) % row_count;
            }
            Direction::Left => {
                let col_count = layout[self.cursor_row].len();
                if col_count > 0 {
                    if self.cursor_col == 0 {
                        self.cursor_col = col_count - 1;
                    } else {
                        self.cursor_col -= 1;
                    }
                }
            }
            Direction::Right => {
                let col_count = layout[self.cursor_row].len();
                if col_count > 0 {
                    self.cursor_col = (self.cursor_col + 1) % col_count;
                }
            }
        }

        // Clamp cursor_col to the current row width
        let current_cols = layout[self.cursor_row].len();
        if current_cols > 0 && self.cursor_col >= current_cols {
            self.cursor_col = current_cols - 1;
        }
    }

    pub fn toggle_shift(&mut self) {
        self.shift_active = !self.shift_active;
        self.last_output_desc = format!("Shift: {}", if self.shift_active { "ON" } else { "OFF" });
    }

    pub fn toggle_ctrl(&mut self) {
        self.ctrl_active = !self.ctrl_active;
        self.last_output_desc = format!("Ctrl: {}", if self.ctrl_active { "ON" } else { "OFF" });
    }

    pub fn switch_to_layer(&mut self, layer: Layer) {
        if self.active_layer == layer {
            self.active_layer = Layer::Base;
        } else {
            self.active_layer = layer;
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.last_output_desc = format!("Layer: {}", self.active_layer.name());
    }

    pub fn toggle_l1(&mut self) {
        self.switch_to_layer(Layer::Symbols);
    }

    pub fn toggle_r1(&mut self) {
        self.active_layer = match self.active_layer {
            Layer::Base => Layer::Numbers,
            Layer::Numbers => Layer::Symbols,
            Layer::Symbols => Layer::Base,
        };
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.last_output_desc = format!("Layer: {}", self.active_layer.name());
    }

    pub fn selected_key(&self) -> Option<KeyDef> {
        let layout = self.get_layout();
        if self.cursor_row < layout.len() {
            let row = &layout[self.cursor_row];
            if self.cursor_col < row.len() {
                return Some(row[self.cursor_col].clone());
            }
        }
        None
    }

    /// Evaluates the current selection or an action and returns the bytes to send to PTY
    pub fn process_action(&mut self, action: KeyAction) -> Option<Vec<u8>> {
        match action {
            KeyAction::Char(c) => {
                let bytes = if self.ctrl_active {
                    let upper = c.to_ascii_uppercase();
                    if upper.is_ascii_uppercase() {
                        let ctrl_code = (upper as u8) - b'@';
                        self.last_output_desc = format!("Sent: ^{} (0x{:02X})", upper, ctrl_code);
                        vec![ctrl_code]
                    } else {
                        self.last_output_desc = format!("Sent: '{}'", c);
                        c.to_string().into_bytes()
                    }
                } else {
                    self.last_output_desc = format!("Sent: '{}'", c);
                    c.to_string().into_bytes()
                };

                // One-shot shift and sticky ctrl consume on character output
                self.shift_active = false;
                self.ctrl_active = false;
                Some(bytes)
            }
            KeyAction::Text(txt) => {
                self.last_output_desc = format!("Sent text: \"{}\"", txt);
                self.shift_active = false;
                self.ctrl_active = false;
                Some(txt.as_bytes().to_vec())
            }
            KeyAction::Space => {
                self.last_output_desc = "Sent: Space".to_string();
                self.shift_active = false;
                self.ctrl_active = false;
                Some(vec![b' '])
            }
            KeyAction::Backspace => {
                self.last_output_desc = "Sent: Backspace".to_string();
                // 0x7F / 0x08 for backspace
                Some(vec![0x7F])
            }
            KeyAction::Enter => {
                self.last_output_desc = "Sent: Enter".to_string();
                self.shift_active = false;
                self.ctrl_active = false;
                Some(vec![b'\r'])
            }
            KeyAction::Tab => {
                self.last_output_desc = "Sent: Tab".to_string();
                Some(vec![b'\t'])
            }
            KeyAction::Escape => {
                self.last_output_desc = "Sent: Escape".to_string();
                Some(vec![0x1B])
            }
            KeyAction::ToggleShift => {
                self.toggle_shift();
                None
            }
            KeyAction::ToggleCtrl => {
                self.toggle_ctrl();
                None
            }
            KeyAction::SetLayer(l) => {
                self.switch_to_layer(l);
                None
            }
            KeyAction::Arrow(Direction::Up) => Some(vec![0x1B, b'[', b'A']),
            KeyAction::Arrow(Direction::Down) => Some(vec![0x1B, b'[', b'B']),
            KeyAction::Arrow(Direction::Right) => Some(vec![0x1B, b'[', b'C']),
            KeyAction::Arrow(Direction::Left) => Some(vec![0x1B, b'[', b'D']),
            KeyAction::PageUp => Some(vec![0x1B, b'[', b'5', b'~']),
            KeyAction::PageDown => Some(vec![0x1B, b'[', b'6', b'~']),
            KeyAction::Home => Some(vec![0x1B, b'[', b'H']),
            KeyAction::End => Some(vec![0x1B, b'[', b'F']),
        }
    }

    pub fn press_select(&mut self) -> Option<Vec<u8>> {
        if let Some(key) = self.selected_key() {
            self.process_action(key.action)
        } else {
            None
        }
    }

    pub fn toggle_visibility(&mut self) -> bool {
        self.visible = !self.visible;
        self.last_output_desc = format!("Keyboard: {}", if self.visible { "SHOWN" } else { "HIDDEN" });
        self.visible
    }

    /// Determines if a screen coordinate (col, row) hits any key in the keyboard grid
    pub fn hit_test(&self, col: u16, row: u16, start_row: u16) -> Option<(usize, usize)> {
        if !self.visible || row < start_row {
            return None;
        }
        let r_idx = (row - start_row) as usize;
        let layout = self.get_layout();
        if r_idx >= layout.len() {
            return None;
        }

        let keys = &layout[r_idx];
        let mut cur_x = 0u16;

        for (c_idx, key) in keys.iter().enumerate() {
            let key_width = (key.label.len() + 2) as u16;
            let end_x = cur_x + key_width;

            if col >= cur_x && col < end_x {
                return Some((r_idx, c_idx));
            }

            cur_x = end_x + 1; // 1 space gap between keys
        }

        None
    }

    /// Selects and triggers the key at (r_idx, c_idx) directly (Android/touch style)
    pub fn tap_key(&mut self, r_idx: usize, c_idx: usize) -> Option<Vec<u8>> {
        let layout = self.get_layout();
        if r_idx < layout.len() && c_idx < layout[r_idx].len() {
            self.cursor_row = r_idx;
            self.cursor_col = c_idx;
            self.press_select()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_navigation_and_wrapping() {
        let mut kb = KeyboardState::new();
        kb.cursor_row = 0;
        kb.cursor_col = 0;

        // Move Left wraps to end of row
        kb.move_cursor(Direction::Left);
        let layout = kb.get_layout();
        assert_eq!(kb.cursor_col, layout[0].len() - 1);

        // Move Right wraps back to start of row
        kb.move_cursor(Direction::Right);
        assert_eq!(kb.cursor_col, 0);

        // Move Up wraps to bottom row
        kb.move_cursor(Direction::Up);
        assert_eq!(kb.cursor_row, layout.len() - 1);

        // Move Down wraps to top row
        kb.move_cursor(Direction::Down);
        assert_eq!(kb.cursor_row, 0);
    }

    #[test]
    fn test_one_shot_shift() {
        let mut kb = KeyboardState::new();
        kb.cursor_row = 1;
        kb.cursor_col = 0; // 'q'

        // Normal press yields lowercase 'q'
        assert_eq!(kb.press_select(), Some(vec![b'q']));

        // Toggle Shift
        kb.toggle_shift();
        assert!(kb.shift_active);

        // Shifted press yields uppercase 'Q' and consumes Shift
        assert_eq!(kb.press_select(), Some(vec![b'Q']));
        assert!(!kb.shift_active);

        // Next press is back to lowercase 'q'
        assert_eq!(kb.press_select(), Some(vec![b'q']));
    }

    #[test]
    fn test_sticky_ctrl_modifiers() {
        let mut kb = KeyboardState::new();
        kb.cursor_row = 3;
        kb.cursor_col = 2; // 'c' ('z'=0, 'x'=1, 'c'=2)

        // Toggle Ctrl
        kb.toggle_ctrl();
        assert!(kb.ctrl_active);

        // Ctrl+C yields ASCII 0x03 and consumes Ctrl
        assert_eq!(kb.press_select(), Some(vec![0x03]));
        assert!(!kb.ctrl_active);

        // Test Ctrl+D (0x04)
        kb.cursor_row = 2;
        kb.cursor_col = 2; // 'd'
        kb.toggle_ctrl();
        assert_eq!(kb.press_select(), Some(vec![0x04]));

        // Test Ctrl+L (0x0C)
        kb.cursor_row = 2;
        kb.cursor_col = 8; // 'l'
        kb.toggle_ctrl();
        assert_eq!(kb.press_select(), Some(vec![0x0C]));
    }

    #[test]
    fn test_essential_keys() {
        let mut kb = KeyboardState::new();
        assert_eq!(kb.process_action(KeyAction::Space), Some(vec![b' ']));
        assert_eq!(kb.process_action(KeyAction::Backspace), Some(vec![0x7F]));
        assert_eq!(kb.process_action(KeyAction::Enter), Some(vec![b'\r']));
        assert_eq!(kb.process_action(KeyAction::Escape), Some(vec![0x1B]));
        assert_eq!(kb.process_action(KeyAction::Tab), Some(vec![b'\t']));
    }

    #[test]
    fn test_layers_switching() {
        let mut kb = KeyboardState::new();
        assert_eq!(kb.active_layer, Layer::Base);

        // Cycle to Numbers (R1)
        kb.toggle_r1();
        assert_eq!(kb.active_layer, Layer::Numbers);

        // Cycle to Symbols (R1)
        kb.toggle_r1();
        assert_eq!(kb.active_layer, Layer::Symbols);

        // Cycle back to Base (R1)
        kb.toggle_r1();
        assert_eq!(kb.active_layer, Layer::Base);

        // Switch to Symbols directly (L1)
        kb.toggle_l1();
        assert_eq!(kb.active_layer, Layer::Symbols);
    }

    #[test]
    fn test_touch_tap_and_hit_test() {
        let mut kb = KeyboardState::new();
        // Row 0 start_row = 10. Key 0 ('1') is cols 0..2, Key 1 ('2') is cols 4..6
        assert_eq!(kb.hit_test(1, 10, 10), Some((0, 0)));
        assert_eq!(kb.hit_test(5, 10, 10), Some((0, 1)));

        // Tap on key (0, 0)
        let output = kb.tap_key(0, 0);
        assert_eq!(output, Some(vec![b'1']));
        assert_eq!(kb.cursor_row, 0);
        assert_eq!(kb.cursor_col, 0);
    }
}
