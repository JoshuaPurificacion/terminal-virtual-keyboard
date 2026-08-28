use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Hybrid,  // Direct keyboard passthrough + virtual navigation & touch
    Gamepad, // Physical keys emulate RG353M gamepad buttons
}

impl InputMode {
    pub fn name(&self) -> &'static str {
        match self {
            InputMode::Hybrid => "DIRECT / HYBRID",
            InputMode::Gamepad => "GAMEPAD ONLY",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Up,
    Down,
    Left,
    Right,
    Select,   // Gamepad A: Select character / execute key
    Back,     // Gamepad B: Backspace / cancel
    ShiftKey, // Gamepad X: Shift modifier toggle
    CtrlKey,  // Gamepad Y: Ctrl modifier toggle
    L1,       // Gamepad L1: Symbols layer
    R1,       // Gamepad R1: Numbers layer
    Start,    // Gamepad Start: Enter
    Select2,  // Gamepad Select: Escape
    Quit,     // System exit
    ToggleMode, // Toggle between Hybrid and Gamepad Mode
    Passthrough(Vec<u8>), // Direct raw bytes to send to shell
}

#[allow(dead_code)]
pub trait InputSource {
    fn poll_event(&mut self) -> Option<InputEvent>;
}

pub struct PcKeyboardInput {
    pub mode: InputMode,
}

impl Default for PcKeyboardInput {
    fn default() -> Self {
        Self::new()
    }
}

impl PcKeyboardInput {
    pub fn new() -> Self {
        Self {
            mode: InputMode::Hybrid,
        }
    }

    pub fn toggle_mode(&mut self) -> InputMode {
        self.mode = match self.mode {
            InputMode::Hybrid => InputMode::Gamepad,
            InputMode::Gamepad => InputMode::Hybrid,
        };
        self.mode
    }

    pub fn map_key(&self, key: KeyEvent) -> Option<InputEvent> {
        // Handle explicit exit shortcuts
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => return Some(InputEvent::Quit),
                _ => {}
            }
        }
        if key.code == KeyCode::F(10) {
            return Some(InputEvent::Quit);
        }

        // Mode toggle key: F9 or Insert
        if key.code == KeyCode::F(9) || key.code == KeyCode::Insert {
            return Some(InputEvent::ToggleMode);
        }

        match self.mode {
            InputMode::Hybrid => self.map_hybrid_key(key),
            InputMode::Gamepad => self.map_gamepad_key(key),
        }
    }

    fn map_hybrid_key(&self, key: KeyEvent) -> Option<InputEvent> {
        match key.code {
            // Virtual Keyboard Navigation
            KeyCode::Up => Some(InputEvent::Up),
            KeyCode::Down => Some(InputEvent::Down),
            KeyCode::Left => Some(InputEvent::Left),
            KeyCode::Right => Some(InputEvent::Right),

            // Layer & Modifier function keys
            KeyCode::F(1) => Some(InputEvent::ShiftKey),
            KeyCode::F(2) => Some(InputEvent::CtrlKey),
            KeyCode::F(3) | KeyCode::PageUp => Some(InputEvent::L1),
            KeyCode::F(4) | KeyCode::PageDown => Some(InputEvent::R1),
            KeyCode::F(5) => Some(InputEvent::Start),
            KeyCode::F(12) => Some(InputEvent::Select2),

            // Direct Typing: Control Sequences
            KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let upper = c.to_ascii_uppercase();
                if upper.is_ascii_uppercase() {
                    let ctrl_code = (upper as u8) - b'@';
                    Some(InputEvent::Passthrough(vec![ctrl_code]))
                } else {
                    Some(InputEvent::Passthrough(c.to_string().into_bytes()))
                }
            }

            // Direct Typing: Characters & Common Keys
            KeyCode::Char(c) => Some(InputEvent::Passthrough(c.to_string().into_bytes())),
            KeyCode::Enter => Some(InputEvent::Passthrough(vec![b'\r'])),
            KeyCode::Backspace => Some(InputEvent::Passthrough(vec![0x7F])),
            KeyCode::Tab => Some(InputEvent::Passthrough(vec![b'\t'])),
            KeyCode::Esc => Some(InputEvent::Passthrough(vec![0x1B])),
            KeyCode::Home => Some(InputEvent::Passthrough(b"\x1b[H".to_vec())),
            KeyCode::End => Some(InputEvent::Passthrough(b"\x1b[F".to_vec())),
            KeyCode::Delete => Some(InputEvent::Passthrough(b"\x1b[3~".to_vec())),

            _ => None,
        }
    }

    fn map_gamepad_key(&self, key: KeyEvent) -> Option<InputEvent> {
        match key.code {
            // D-Pad / Navigation
            KeyCode::Up => Some(InputEvent::Up),
            KeyCode::Down => Some(InputEvent::Down),
            KeyCode::Left => Some(InputEvent::Left),
            KeyCode::Right => Some(InputEvent::Right),

            // Gamepad A / Select
            KeyCode::Enter => Some(InputEvent::Select),
            KeyCode::Char(' ') => Some(InputEvent::Select),
            KeyCode::Char('j') | KeyCode::Char('J') => Some(InputEvent::Select),
            KeyCode::Char('z') | KeyCode::Char('Z') => Some(InputEvent::Select),

            // Gamepad B / Back (Backspace)
            KeyCode::Backspace => Some(InputEvent::Back),
            KeyCode::Char('k') | KeyCode::Char('K') => Some(InputEvent::Back),

            // Gamepad X / Shift
            KeyCode::F(1) => Some(InputEvent::ShiftKey),
            KeyCode::Char('u') | KeyCode::Char('U') => Some(InputEvent::ShiftKey),
            KeyCode::Char('x') | KeyCode::Char('X') => Some(InputEvent::ShiftKey),

            // Gamepad Y / Ctrl
            KeyCode::F(2) => Some(InputEvent::CtrlKey),
            KeyCode::Char('i') | KeyCode::Char('I') => Some(InputEvent::CtrlKey),
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(InputEvent::CtrlKey),

            // Bumpers / Layers
            KeyCode::F(3) | KeyCode::PageUp | KeyCode::Char('[') => Some(InputEvent::L1),
            KeyCode::F(4) | KeyCode::PageDown | KeyCode::Char(']') => Some(InputEvent::R1),

            // Start & Select buttons
            KeyCode::Tab | KeyCode::F(5) => Some(InputEvent::Start),
            KeyCode::Esc | KeyCode::F(12) => Some(InputEvent::Select2),

            _ => None,
        }
    }
}

pub struct EvdevGamepadInput {
    pub device: Option<evdev::Device>,
}

impl EvdevGamepadInput {
    pub fn new(path: Option<&str>) -> Self {
        let device = if let Some(p) = path {
            evdev::Device::open(p).ok()
        } else {
            Self::find_gamepad()
        };

        Self { device }
    }

    pub fn find_gamepad() -> Option<evdev::Device> {
        let entries = evdev::enumerate();
        for (_, dev) in entries {
            let name = dev.name().unwrap_or("").to_lowercase();
            if name.contains("retrogame")
                || name.contains("joypad")
                || name.contains("gamepad")
                || name.contains("rk_")
                || name.contains("odroid")
            {
                return Some(dev);
            }
        }
        None
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        let dev = self.device.as_mut()?;
        if let Ok(events) = dev.fetch_events() {
            for ev in events {
                if let evdev::InputEventKind::Key(key) = ev.kind() {
                    // Only process key press (value == 1)
                    if ev.value() == 1 {
                        if let Some(mapped) = Self::map_evdev_key(key) {
                            return Some(mapped);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn map_evdev_key(key: evdev::Key) -> Option<InputEvent> {
        match key {
            evdev::Key::KEY_UP | evdev::Key::BTN_DPAD_UP => Some(InputEvent::Up),
            evdev::Key::KEY_DOWN | evdev::Key::BTN_DPAD_DOWN => Some(InputEvent::Down),
            evdev::Key::KEY_LEFT | evdev::Key::BTN_DPAD_LEFT => Some(InputEvent::Left),
            evdev::Key::KEY_RIGHT | evdev::Key::BTN_DPAD_RIGHT => Some(InputEvent::Right),

            evdev::Key::BTN_SOUTH | evdev::Key::BTN_0 | evdev::Key::KEY_ENTER => Some(InputEvent::Select),
            evdev::Key::BTN_EAST | evdev::Key::BTN_1 | evdev::Key::KEY_BACKSPACE => Some(InputEvent::Back),
            evdev::Key::BTN_NORTH | evdev::Key::BTN_2 | evdev::Key::KEY_F1 => Some(InputEvent::ShiftKey),
            evdev::Key::BTN_WEST | evdev::Key::BTN_3 | evdev::Key::KEY_F2 => Some(InputEvent::CtrlKey),

            evdev::Key::BTN_TL | evdev::Key::BTN_TL2 | evdev::Key::KEY_F3 | evdev::Key::KEY_PAGEUP => Some(InputEvent::L1),
            evdev::Key::BTN_TR | evdev::Key::BTN_TR2 | evdev::Key::KEY_F4 | evdev::Key::KEY_PAGEDOWN => Some(InputEvent::R1),

            evdev::Key::BTN_START | evdev::Key::KEY_TAB => Some(InputEvent::Start),
            evdev::Key::BTN_SELECT | evdev::Key::KEY_ESC => Some(InputEvent::Select2),

            evdev::Key::BTN_MODE | evdev::Key::KEY_HOMEPAGE | evdev::Key::KEY_F => Some(InputEvent::ToggleMode),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pc_input_mapping() {
        let mut input = PcKeyboardInput::new();
        assert_eq!(input.mode, InputMode::Hybrid);

        // Direct typing in Hybrid mode
        assert_eq!(
            input.map_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
            Some(InputEvent::Passthrough(vec![b'a']))
        );
        assert_eq!(
            input.map_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Some(InputEvent::Passthrough(vec![b'\r']))
        );
        assert_eq!(
            input.map_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(InputEvent::Passthrough(vec![0x03])) // Ctrl+C
        );

        // Navigation still works in Hybrid mode
        assert_eq!(input.map_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)), Some(InputEvent::Up));

        // Switch to Gamepad mode
        input.toggle_mode();
        assert_eq!(input.mode, InputMode::Gamepad);

        // In Gamepad mode, 'j' selects, 'k' backspaces
        assert_eq!(input.map_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)), Some(InputEvent::Select));
        assert_eq!(input.map_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)), Some(InputEvent::Back));

        // Mode toggle key
        assert_eq!(input.map_key(KeyEvent::new(KeyCode::F(9), KeyModifiers::NONE)), Some(InputEvent::ToggleMode));

        // Quit
        assert_eq!(input.map_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL)), Some(InputEvent::Quit));
    }

    #[test]
    fn test_evdev_key_mapping() {
        assert_eq!(EvdevGamepadInput::map_evdev_key(evdev::Key::BTN_SOUTH), Some(InputEvent::Select));
        assert_eq!(EvdevGamepadInput::map_evdev_key(evdev::Key::BTN_EAST), Some(InputEvent::Back));
        assert_eq!(EvdevGamepadInput::map_evdev_key(evdev::Key::BTN_NORTH), Some(InputEvent::ShiftKey));
        assert_eq!(EvdevGamepadInput::map_evdev_key(evdev::Key::BTN_WEST), Some(InputEvent::CtrlKey));
        assert_eq!(EvdevGamepadInput::map_evdev_key(evdev::Key::BTN_TL), Some(InputEvent::L1));
        assert_eq!(EvdevGamepadInput::map_evdev_key(evdev::Key::BTN_TR), Some(InputEvent::R1));
        assert_eq!(EvdevGamepadInput::map_evdev_key(evdev::Key::BTN_START), Some(InputEvent::Start));
        assert_eq!(EvdevGamepadInput::map_evdev_key(evdev::Key::BTN_SELECT), Some(InputEvent::Select2));
        assert_eq!(EvdevGamepadInput::map_evdev_key(evdev::Key::BTN_MODE), Some(InputEvent::ToggleMode));
        assert_eq!(EvdevGamepadInput::map_evdev_key(evdev::Key::BTN_DPAD_UP), Some(InputEvent::Up));
    }
}
