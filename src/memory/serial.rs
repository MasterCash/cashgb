//! Serial port (link cable): SB (0xFF01) and SC (0xFF02).
//!
//! A transfer starts when the game writes SC with bit 7 (start) and bit 0 (internal clock)
//! set. Eight bits shift out of SB at 8192 Hz while eight bits shift in from the other Game
//! Boy. With no link partner connected every incoming bit is 1, so SB ends up as 0xFF. When
//! the transfer finishes, SC bit 7 clears and the Serial interrupt is requested.
//! See <https://gbdev.io/pandocs/Serial_Data_Transfer_(Link_Cable).html>.
//!
//! Every byte sent is also recorded, so tests and tools can read what a program printed.
//! Blargg's test ROMs report their results this way.

/// SC bit 7: a transfer is requested or in progress
const TRANSFER_ACTIVE: u8 = 0x80;
/// SC bit 0: this Game Boy drives the clock
const INTERNAL_CLOCK: u8 = 0x01;
/// Eight bits at 8192 Hz: 8 * (1_048_576 / 8192) M-cycles
const TRANSFER_M_CYCLES: u16 = 1024;

#[derive(Debug, Clone, Default)]
pub struct Serial {
    /// SB: the byte being sent, replaced bit by bit with the byte received
    data: u8,
    /// SC: transfer control
    control: u8,
    /// M-cycles left in the current transfer, while one is running
    remaining: Option<u16>,
    /// Every byte sent, in order
    output: Vec<u8>,
}

impl Serial {
    pub fn new() -> Self {
        Self::default()
    }

    /// Read SB (0xFF01) or SC (0xFF02). SC's unused bits 1-6 read as 1.
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.data,
            _ => self.control | 0x7E,
        }
    }

    /// Write SB (0xFF01) or SC (0xFF02).
    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF01 => self.data = value,
            _ => {
                self.control = value & (TRANSFER_ACTIVE | INTERNAL_CLOCK);
                // With an external clock the transfer waits for a partner that never arrives
                if self.control == TRANSFER_ACTIVE | INTERNAL_CLOCK {
                    self.output.push(self.data);
                    self.remaining = Some(TRANSFER_M_CYCLES);
                } else {
                    self.remaining = None;
                }
            }
        }
    }

    /// Advance one M-cycle. Returns true when a transfer finishes and the Serial interrupt
    /// should be requested.
    pub fn step(&mut self) -> bool {
        let Some(remaining) = self.remaining else {
            return false;
        };
        if remaining > 1 {
            self.remaining = Some(remaining - 1);
            return false;
        }
        self.remaining = None;
        self.data = 0xFF; // nothing connected: every received bit is 1
        self.control &= !TRANSFER_ACTIVE;
        true
    }

    /// Every byte sent over the serial port so far.
    pub fn output(&self) -> &[u8] {
        &self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_transfer(serial: &mut Serial) -> u16 {
        let mut cycles = 0;
        loop {
            cycles += 1;
            if serial.step() {
                return cycles;
            }
            assert!(cycles < 10_000, "transfer never finished");
        }
    }

    #[test]
    fn internal_clock_transfer_records_the_byte() {
        let mut serial = Serial::new();
        serial.write(0xFF01, b'P');
        serial.write(0xFF02, 0x81);

        assert_eq!(serial.output(), b"P");
    }

    #[test]
    fn transfer_takes_1024_m_cycles_and_then_finishes() {
        let mut serial = Serial::new();
        serial.write(0xFF01, 0x42);
        serial.write(0xFF02, 0x81);
        assert_eq!(serial.read(0xFF02) & 0x80, 0x80, "busy while shifting");

        assert_eq!(run_transfer(&mut serial), 1024);
        assert_eq!(serial.read(0xFF02) & 0x80, 0, "start bit clears when done");
        assert_eq!(serial.read(0xFF01), 0xFF, "no partner: received all 1s");
    }

    #[test]
    fn external_clock_transfer_never_starts() {
        let mut serial = Serial::new();
        serial.write(0xFF01, 0x42);
        serial.write(0xFF02, 0x80);

        assert!(serial.output().is_empty());
        assert!(!serial.step());
    }

    #[test]
    fn byte_is_captured_even_if_sb_changes_mid_transfer() {
        let mut serial = Serial::new();
        serial.write(0xFF01, b'a');
        serial.write(0xFF02, 0x81);
        serial.write(0xFF01, b'b');
        serial.write(0xFF02, 0x81);

        assert_eq!(serial.output(), b"ab");
    }

    #[test]
    fn unused_sc_bits_read_as_one() {
        let serial = Serial::new();
        assert_eq!(serial.read(0xFF02), 0x7E);
    }
}
