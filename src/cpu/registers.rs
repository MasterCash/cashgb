use crate::cpu::instructions::Interrupt;

#[derive(Clone, Copy)]
pub struct Register {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
}

impl Register {
    pub fn new() -> Self {
        Self {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
        }
    }
    pub fn get_a(&self) -> u8 {
        (self.af >> 8) as u8
    }

    pub fn set_a(&mut self, value: u8) {
        self.af = (self.af & 0x00FF) | ((value as u16) << 8);
    }

    pub fn get_f(&self) -> u8 {
        self.af as u8
    }

    pub fn set_f(&mut self, value: u8) {
        self.af = (self.af & 0xff00) | value as u16;
    }

    pub fn get_af(&self) -> u16 {
        self.af
    }

    pub fn set_af(&mut self, value: u16) {
        self.af = value;
    }

    pub fn get_b(&self) -> u8 {
        (self.bc >> 8) as u8
    }

    pub fn set_b(&mut self, value: u8) {
        self.bc = (self.bc & 0x00FF) | ((value as u16) << 8);
    }

    pub fn get_c(&self) -> u8 {
        self.bc as u8
    }

    pub fn set_c(&mut self, value: u8) {
        self.bc = (self.bc & 0xff00) | value as u16;
    }

    pub fn get_bc(&self) -> u16 {
        self.bc
    }

    pub fn set_bc(&mut self, value: u16) {
        self.bc = value;
    }

    pub fn get_d(&self) -> u8 {
        (self.de >> 8) as u8
    }

    pub fn set_d(&mut self, value: u8) {
        self.de = (self.de & 0x00FF) | ((value as u16) << 8);
    }

    pub fn get_e(&self) -> u8 {
        self.de as u8
    }

    pub fn set_e(&mut self, value: u8) {
        self.de = (self.de & 0xff00) | value as u16;
    }

    pub fn get_de(&self) -> u16 {
        self.de
    }

    pub fn set_de(&mut self, value: u16) {
        self.de = value;
    }

    pub fn get_h(&self) -> u8 {
        (self.hl >> 8) as u8
    }

    pub fn set_h(&mut self, value: u8) {
        self.hl = (self.hl & 0x00FF) | ((value as u16) << 8);
    }

    pub fn get_l(&self) -> u8 {
        self.hl as u8
    }

    pub fn set_l(&mut self, value: u8) {
        self.hl = (self.hl & 0xff00) | value as u16;
    }

    pub fn get_hl(&self) -> u16 {
        self.hl
    }

    pub fn set_hl(&mut self, value: u16) {
        self.hl = value;
    }
}

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

/// The timer: DIV (0xFF04), TIMA (0xFF05), TMA (0xFF06) and TAC (0xFF07).
///
/// Modelled as hardware does it: one 16-bit counter advances every T-cycle and DIV is its
/// upper 8 bits. TIMA increments when the counter bit selected by TAC goes from 1 to 0 while
/// the timer is enabled. Because everything hangs off that one counter, writing DIV (which
/// clears the counter) or changing TAC can itself cause a TIMA increment, as on a real DMG.
/// See <https://gbdev.io/pandocs/Timer_Obscure_Behaviour.html>.
pub struct TimerRegister {
    /// Internal counter, advanced by 4 every M-cycle. DIV is the upper byte.
    counter: u16,
    /// Timer Counter (TIMA)
    tima: u8,
    /// Timer Modulo (TMA): loaded into TIMA when it overflows
    tma: u8,
    /// Timer Control (TAC): bit 2 enables TIMA, bits 0-1 select its frequency
    tac: u8,
}

impl TimerRegister {
    pub fn new() -> Self {
        Self {
            counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    /// Advance one M-cycle. Returns the Timer interrupt when TIMA overflows.
    pub fn step(&mut self) -> Option<Interrupt> {
        self.update(|timer| timer.counter = timer.counter.wrapping_add(4))
    }

    /// Write a timer register. Returns the Timer interrupt if the write made TIMA overflow.
    pub fn write(&mut self, addr: u16, value: u8) -> Option<Interrupt> {
        match addr {
            0xff04 => self.update(|timer| timer.counter = 0), // any write clears the counter
            0xff05 => {
                self.tima = value;
                None
            }
            0xff06 => {
                self.tma = value;
                None
            }
            0xff07 => self.update(|timer| timer.tac = value & 0x07),
            _ => None,
        }
    }

    pub fn read(&self, addr: u16) -> Option<u8> {
        match addr {
            0xff04 => Some((self.counter >> 8) as u8),
            0xff05 => Some(self.tima),
            0xff06 => Some(self.tma),
            0xff07 => Some(self.tac | 0xF8), // Upper 5 bits are always read as 1
            _ => None,
        }
    }

    pub fn is_timer_enabled(&self) -> bool {
        (self.tac & 0x04) != 0
    }

    /// Counter bit that clocks TIMA for the frequency selected in TAC:
    /// 4096 Hz, 262144 Hz, 65536 Hz, 16384 Hz.
    fn selected_bit(&self) -> u16 {
        match self.tac & 0x03 {
            0 => 1 << 9,
            1 => 1 << 3,
            2 => 1 << 5,
            _ => 1 << 7,
        }
    }

    /// The signal TIMA counts falling edges of: the selected counter bit, gated by enable.
    fn timer_signal(&self) -> bool {
        self.is_timer_enabled() && self.counter & self.selected_bit() != 0
    }

    /// Apply a change to the counter or TAC, and increment TIMA if it made the timer signal
    /// fall from 1 to 0.
    fn update(&mut self, change: impl FnOnce(&mut Self)) -> Option<Interrupt> {
        let before = self.timer_signal();
        change(self);
        if before && !self.timer_signal() {
            self.increment_tima()
        } else {
            None
        }
    }

    fn increment_tima(&mut self) -> Option<Interrupt> {
        let (tima, overflowed) = self.tima.overflowing_add(1);
        if overflowed {
            self.tima = self.tma;
            Some(Interrupt::Timer)
        } else {
            self.tima = tima;
            None
        }
    }
}

impl Default for TimerRegister {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod timer_tests {
    use super::*;

    /// Step until TIMA changes, returning how many M-cycles that took.
    fn cycles_until_tima_changes(timer: &mut TimerRegister) -> u32 {
        let start = timer.read(0xff05);
        for cycles in 1..=2048 {
            timer.step();
            if timer.read(0xff05) != start {
                return cycles;
            }
        }
        panic!("TIMA never changed");
    }

    #[test]
    fn div_increments_every_64_m_cycles() {
        let mut timer = TimerRegister::new();
        for _ in 0..63 {
            timer.step();
        }
        assert_eq!(timer.read(0xff04), Some(0));
        timer.step();
        assert_eq!(timer.read(0xff04), Some(1));
    }

    #[test]
    fn tima_rate_follows_tac() {
        // M-cycles per TIMA increment for TAC frequency 0-3
        for (frequency, period) in [(0, 256), (1, 4), (2, 16), (3, 64)] {
            let mut timer = TimerRegister::new();
            timer.write(0xff07, 0x04 | frequency);
            assert_eq!(
                cycles_until_tima_changes(&mut timer),
                period,
                "TAC {frequency}"
            );
            assert_eq!(
                cycles_until_tima_changes(&mut timer),
                period,
                "TAC {frequency}"
            );
        }
    }

    #[test]
    fn disabled_timer_does_not_count() {
        let mut timer = TimerRegister::new();
        timer.write(0xff07, 0x01); // fastest rate, but disabled
        for _ in 0..1000 {
            timer.step();
        }
        assert_eq!(timer.read(0xff05), Some(0));
    }

    #[test]
    fn overflow_reloads_tma_and_requests_interrupt() {
        let mut timer = TimerRegister::new();
        timer.write(0xff05, 0xFF);
        timer.write(0xff06, 0xAB);
        timer.write(0xff07, 0x05); // enabled, every 4 M-cycles

        let interrupts: Vec<_> = (0..4).filter_map(|_| timer.step()).collect();

        assert_eq!(interrupts, [Interrupt::Timer]);
        assert_eq!(timer.read(0xff05), Some(0xAB));
    }

    #[test]
    fn writing_div_resets_the_tima_prescaler() {
        let mut timer = TimerRegister::new();
        timer.write(0xff07, 0x04); // enabled, every 256 M-cycles
        for _ in 0..100 {
            timer.step();
        }
        timer.write(0xff04, 0x12); // clears the shared counter

        assert_eq!(timer.read(0xff04), Some(0));
        assert_eq!(
            cycles_until_tima_changes(&mut timer),
            256,
            "a full period from the reset"
        );
    }

    #[test]
    fn writing_div_while_the_selected_bit_is_set_increments_tima() {
        let mut timer = TimerRegister::new();
        timer.write(0xff07, 0x05); // enabled, counter bit 3
        timer.step();
        timer.step(); // counter = 8: bit 3 set

        timer.write(0xff04, 0x00);

        assert_eq!(
            timer.read(0xff05),
            Some(1),
            "falling edge from clearing the counter"
        );
    }

    #[test]
    fn disabling_while_the_selected_bit_is_set_increments_tima() {
        let mut timer = TimerRegister::new();
        timer.write(0xff07, 0x05);
        timer.step();
        timer.step(); // counter = 8: bit 3 set

        timer.write(0xff07, 0x01);

        assert_eq!(
            timer.read(0xff05),
            Some(1),
            "DMG glitch: disabling is a falling edge"
        );
    }
}
