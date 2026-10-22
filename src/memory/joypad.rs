//! Joypad input register (P1/JOYP, 0xFF00).
//!
//! The game selects a button group by writing 0 to bit 4 (D-pad) or bit 5 (A/B/Select/Start),
//! then reads that group's state in bits 0-3. Every input is active-low: 0 means pressed.
//! See <https://gbdev.io/pandocs/Joypad_Input.html>.

/// A Game Boy button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

impl Button {
    /// Every button, in register bit order: the D-pad group, then the button group.
    pub const ALL: [Button; 8] = [
        Button::Right,
        Button::Left,
        Button::Up,
        Button::Down,
        Button::A,
        Button::B,
        Button::Select,
        Button::Start,
    ];

    /// Bit in `Joypad::pressed`: D-pad in bits 0-3, buttons in bits 4-7.
    fn mask(self) -> u8 {
        1 << self as u8
    }
}

/// Selecting the D-pad group (bit 4 low)
const SELECT_DPAD: u8 = 0x10;
/// Selecting the button group (bit 5 low)
const SELECT_BUTTONS: u8 = 0x20;

#[derive(Debug, Clone)]
pub struct Joypad {
    /// Group select bits as last written (only bits 4-5 are kept)
    select: u8,
    /// Currently held buttons, one bit per `Button` (1 = held)
    pressed: u8,
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            // Neither group selected
            select: SELECT_DPAD | SELECT_BUTTONS,
            pressed: 0,
        }
    }

    /// Read P1: bits 6-7 always read 1, bits 4-5 echo the selection, bits 0-3 are the
    /// selected inputs (0 = pressed). With both groups selected their inputs are ANDed.
    pub fn read(&self) -> u8 {
        0xC0 | self.select | self.input_lines()
    }

    /// Write P1. Only the group select bits are writable.
    pub fn write(&mut self, value: u8) {
        self.select = value & (SELECT_DPAD | SELECT_BUTTONS);
    }

    /// Update a button. Returns true when the Joypad interrupt should fire, which happens
    /// when a currently selected input line goes from released (1) to pressed (0).
    pub fn set_pressed(&mut self, button: Button, pressed: bool) -> bool {
        let before = self.input_lines();
        if pressed {
            self.pressed |= button.mask();
        } else {
            self.pressed &= !button.mask();
        }
        let after = self.input_lines();
        before & !after != 0
    }

    /// Whether a button is currently held.
    pub fn is_pressed(&self, button: Button) -> bool {
        self.pressed & button.mask() != 0
    }

    /// The four input lines (bits 0-3, active-low) for the selected group(s).
    fn input_lines(&self) -> u8 {
        let mut held = 0;
        if self.select & SELECT_DPAD == 0 {
            held |= self.pressed & 0x0F;
        }
        if self.select & SELECT_BUTTONS == 0 {
            held |= self.pressed >> 4;
        }
        !held & 0x0F
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_selected_reads_all_released() {
        let mut joypad = Joypad::new();
        joypad.set_pressed(Button::A, true);
        assert_eq!(joypad.read(), 0xFF);
    }

    #[test]
    fn dpad_group_reports_directions() {
        let mut joypad = Joypad::new();
        joypad.write(0x20); // select D-pad
        joypad.set_pressed(Button::Left, true);
        joypad.set_pressed(Button::A, true); // other group: not visible
        assert_eq!(joypad.read(), 0xE0 | 0b1101);
    }

    #[test]
    fn button_group_reports_buttons() {
        let mut joypad = Joypad::new();
        joypad.write(0x10); // select buttons
        joypad.set_pressed(Button::Start, true);
        joypad.set_pressed(Button::Up, true); // other group: not visible
        assert_eq!(joypad.read(), 0xD0 | 0b0111);
    }

    #[test]
    fn both_groups_selected_are_combined() {
        let mut joypad = Joypad::new();
        joypad.write(0x00);
        joypad.set_pressed(Button::Right, true); // bit 0
        joypad.set_pressed(Button::B, true); // bit 1
        assert_eq!(joypad.read() & 0x0F, 0b1100);
    }

    #[test]
    fn releasing_a_button_restores_the_line() {
        let mut joypad = Joypad::new();
        joypad.write(0x10);
        joypad.set_pressed(Button::A, true);
        joypad.set_pressed(Button::A, false);
        assert_eq!(joypad.read() & 0x0F, 0x0F);
        assert!(!joypad.is_pressed(Button::A));
    }

    #[test]
    fn input_bits_are_read_only() {
        let mut joypad = Joypad::new();
        joypad.write(0x2F); // tries to write the input lines
        assert_eq!(joypad.read(), 0xEF, "inputs still read released");
    }

    #[test]
    fn press_in_selected_group_requests_interrupt() {
        let mut joypad = Joypad::new();
        joypad.write(0x10); // buttons selected
        assert!(joypad.set_pressed(Button::A, true));
    }

    #[test]
    fn press_in_unselected_group_does_not_request_interrupt() {
        let mut joypad = Joypad::new();
        joypad.write(0x10); // buttons selected
        assert!(!joypad.set_pressed(Button::Down, true));
    }

    #[test]
    fn holding_or_releasing_does_not_request_interrupt() {
        let mut joypad = Joypad::new();
        joypad.write(0x10);
        joypad.set_pressed(Button::A, true);
        assert!(!joypad.set_pressed(Button::A, true), "already held");
        assert!(!joypad.set_pressed(Button::A, false), "release");
    }
}
