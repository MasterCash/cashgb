use super::instructions::*;
use super::Cpu;

impl Cpu {
    // Instruction execution methods
    pub(super) fn compare(&mut self, source: CompareSource) {
        let value = match source {
            CompareSource::A => self.register.get_a(),
            CompareSource::B => self.register.get_b(),
            CompareSource::C => self.register.get_c(),
            CompareSource::D => self.register.get_d(),
            CompareSource::E => self.register.get_e(),
            CompareSource::H => self.register.get_h(),
            CompareSource::L => self.register.get_l(),
            CompareSource::HLAddr => self.read(&self.register.get_hl()),
            CompareSource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n
            }
        };

        let (result, c, h) = Self::subtract(&self.register.get_a(), &value);
        println!(
            "Comparing: {} - {} = {}",
            self.register.get_a(),
            value,
            result
        );
        self.update_flag(Flag::Z, result == 0);
        self.update_flag(Flag::N, true);
        self.update_flag(Flag::H, h);
        self.update_flag(Flag::C, c);
    }

    pub(super) fn xor(&mut self, source: XOrSource) {
        let source = match source {
            XOrSource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n
            }
            XOrSource::A => self.register.get_a(),
            XOrSource::B => self.register.get_b(),
            XOrSource::C => self.register.get_c(),
            XOrSource::D => self.register.get_d(),
            XOrSource::E => self.register.get_e(),
            XOrSource::H => self.register.get_h(),
            XOrSource::L => self.register.get_l(),
            XOrSource::HLAddr => self.read(&self.register.get_hl()),
        };
        let xor = self.register.get_a() ^ source;

        let mut results = 0u8;
        if xor == 0 {
            results |= Flag::Z as u8;
        }
        self.register.set_a(xor);
        self.register.set_f(results);
    }

    pub(super) fn load(&mut self, target: LoadTarget, source: LoadSource) {
        let source: u16 = match source {
            LoadSource::PCAddr => {
                let lsb = self.read(&self.program_counter);
                self.program_counter += 1;
                let msb = self.read(&self.program_counter);
                self.program_counter += 1;
                let n = ((msb as u16) << 8) | lsb as u16;
                self.read(&n) as u16
            }
            LoadSource::HL => self.register.get_hl(),
            LoadSource::SPE => {
                let e = self.read(&self.program_counter) as i8;
                self.program_counter += 1;
                (self.stack_pointer as i16 + e as i16) as u16
            }
            LoadSource::BCAddr => self.read(&self.register.get_bc()) as u16,
            LoadSource::DEAddr => self.read(&self.register.get_de()) as u16,
            LoadSource::HLAddrInc => {
                let hl = self.register.get_hl();
                let n = self.read(&hl);
                self.register.set_hl(hl + 1);
                n as u16
            }
            LoadSource::HLAddrDec => {
                let hl = self.register.get_hl();
                let n = self.read(&hl);
                self.register.set_hl(hl - 1);
                n as u16
            }
            LoadSource::SP => self.stack_pointer,
            LoadSource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n as u16
            }
            LoadSource::PC16 => {
                let lsb = self.read(&self.program_counter);
                self.program_counter += 1;
                let msb = self.read(&self.program_counter);
                self.program_counter += 1;
                ((msb as u16) << 8) | lsb as u16
            }
            LoadSource::A => self.register.get_a() as u16,
            LoadSource::B => self.register.get_b() as u16,
            LoadSource::C => self.register.get_c() as u16,
            LoadSource::D => self.register.get_d() as u16,
            LoadSource::E => self.register.get_e() as u16,
            LoadSource::H => self.register.get_h() as u16,
            LoadSource::L => self.register.get_l() as u16,
            LoadSource::HLAddr => self.read(&self.register.get_hl()) as u16,
        };

        match target {
            LoadTarget::A => self.register.set_a(source as u8),
            LoadTarget::B => self.register.set_b(source as u8),
            LoadTarget::C => self.register.set_c(source as u8),
            LoadTarget::D => self.register.set_d(source as u8),
            LoadTarget::E => self.register.set_e(source as u8),
            LoadTarget::H => self.register.set_h(source as u8),
            LoadTarget::L => self.register.set_l(source as u8),
            LoadTarget::BC => self.register.set_bc(source),
            LoadTarget::DE => self.register.set_de(source),
            LoadTarget::HL => self.register.set_hl(source),
            LoadTarget::SP => self.stack_pointer = source,
            LoadTarget::BCAddr => self.write(&self.register.get_bc(), source as u8),
            LoadTarget::DEAddr => self.write(&self.register.get_de(), source as u8),
            LoadTarget::HLAddrInc => {
                let hl = self.register.get_hl();
                self.write(&hl, source as u8);
                self.register.set_hl(hl + 1);
            }
            LoadTarget::HLAddrDec => {
                let hl = self.register.get_hl();
                self.write(&hl, source as u8);
                self.register.set_hl(hl - 1);
            }
            LoadTarget::HLAddr => self.write(&self.register.get_hl(), source as u8),
            LoadTarget::PCAddr => {
                let lsb = self.read(&self.program_counter);
                self.program_counter += 1;
                let msb = self.read(&self.program_counter);
                self.program_counter += 1;
                let n = ((msb as u16) << 8) | lsb as u16;
                self.write(&n, source as u8);
            }
            LoadTarget::PC16Addr => {
                let lsb = self.read(&self.program_counter);
                self.program_counter += 1;
                let msb = self.read(&self.program_counter);
                self.program_counter += 1;
                let n = ((msb as u16) << 8) | lsb as u16;
                self.write(&n, source as u8);
                let n = n + 1;
                self.write(&n, (source >> 8) as u8)
            }
        };
    }

    pub(super) fn bit(&mut self, bit: u8, source: BitwiseSource) {
        let source = match source {
            BitwiseSource::B => self.register.get_b(),
            BitwiseSource::C => self.register.get_c(),
            BitwiseSource::D => self.register.get_d(),
            BitwiseSource::E => self.register.get_e(),
            BitwiseSource::H => self.register.get_h(),
            BitwiseSource::L => self.register.get_l(),
            BitwiseSource::A => self.register.get_a(),
            BitwiseSource::HLAddr => self.read(&self.register.get_hl()),
        };

        self.update_flag(Flag::Z, ((source >> bit) & 1) == 0);
        self.set_flag(Flag::H);
        self.clear_flag(Flag::N);
    }

    pub(super) fn jump_relative(&mut self, condition: JumpCondition) {
        if condition == JumpCondition::None {
            let e = self.read(&self.program_counter) as i8;
            self.program_counter += 1;
            self.program_counter = (self.program_counter as i16 + e as i16) as u16;
            println!("jump to: {:#x}", self.program_counter);
            return;
        }

        let condition = match condition {
            JumpCondition::None => unreachable!(),
            JumpCondition::Z => self.get_flag(Flag::Z),
            JumpCondition::C => self.get_flag(Flag::C),
            JumpCondition::NZ => !self.get_flag(Flag::Z),
            JumpCondition::NC => !self.get_flag(Flag::C),
        };

        if condition {
            self.instruction = Instruction::JumpRelative(JumpCondition::None);
            self.step_count = 1;
        } else {
            self.program_counter += 1;
        }
    }

    pub(super) fn load_accumulator(
        &mut self,
        target: LoadAccumulatorTarget,
        source: LoadAccumulatorSource,
    ) {
        let source = match source {
            LoadAccumulatorSource::A => self.register.get_a(),
            LoadAccumulatorSource::PCAddr => {
                let n = self.read(&self.program_counter) as u16;
                self.program_counter += 1;
                self.read(&(0xFF00 | n))
            }
            LoadAccumulatorSource::CAddr => self.read(&(0xFF00 | (self.register.get_c() as u16))),
        };

        match target {
            LoadAccumulatorTarget::PCAddr => {
                let n = 0xFF00 | self.read(&self.program_counter) as u16;
                self.program_counter += 1;
                self.write(&n, source);
            }
            LoadAccumulatorTarget::A => self.register.set_a(source),
            LoadAccumulatorTarget::CAddr => {
                let addr = (0xFF_u16 << 8) | (self.register.get_c() as u16);
                self.write(&addr, source);
            }
        };
    }

    pub(super) fn increment(&mut self, target: IncrementTarget) {
        match target {
            IncrementTarget::BC => {
                let (result, _) = self.register.get_bc().overflowing_add(1);
                self.register.set_bc(result);
            }
            IncrementTarget::DE => {
                let (result, _) = self.register.get_de().overflowing_add(1);
                self.register.set_de(result);
            }
            IncrementTarget::HL => {
                let (result, _) = self.register.get_hl().overflowing_add(1);
                self.register.set_hl(result);
            }
            IncrementTarget::SP => {
                let (result, _) = self.stack_pointer.overflowing_add(1);
                self.stack_pointer = result;
            }
            IncrementTarget::B => {
                let (results, _, h) = Self::addition(&self.register.get_b(), &1);
                self.register.set_b(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::D => {
                let (results, _, h) = Self::addition(&self.register.get_d(), &1);
                self.register.set_d(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::H => {
                let (results, _, h) = Self::addition(&self.register.get_h(), &1);
                self.register.set_h(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::C => {
                let (results, _, h) = Self::addition(&self.register.get_c(), &1);
                self.register.set_c(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::E => {
                let (results, _, h) = Self::addition(&self.register.get_e(), &1);
                self.register.set_e(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::L => {
                let (results, _, h) = Self::addition(&self.register.get_l(), &1);
                self.register.set_l(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::A => {
                let (results, _, h) = Self::addition(&self.register.get_a(), &1);
                self.register.set_a(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::HLAddr => {
                let addr = self.register.get_hl();
                let (results, _, h) = Self::addition(&self.read(&addr), &1);
                self.write(&addr, results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
        }
    }

    // Add stub implementations for all other instruction methods
    // You can implement these by copying from the original cpu.rs
    pub(super) fn call(&mut self, condition: JumpCondition) {
        if condition == JumpCondition::None {
            let addr = self.read(&self.program_counter) as u16;
            self.program_counter += 1;
            let addr = addr | ((self.read(&self.program_counter) as u16) << 8);
            self.program_counter += 1;

            self.restart(addr);
            return;
        }

        let condition = match condition {
            JumpCondition::None => unreachable!(),
            JumpCondition::Z => self.get_flag(Flag::Z),
            JumpCondition::C => self.get_flag(Flag::C),
            JumpCondition::NZ => !self.get_flag(Flag::Z),
            JumpCondition::NC => !self.get_flag(Flag::C),
        };

        if condition {
            self.instruction = Instruction::Call(JumpCondition::None);
            self.step_count = 3;
        } else {
            self.program_counter += 3;
        }
    }
    pub(super) fn sub(&mut self, source: SubtractSource) {
        let source = match source {
            SubtractSource::A => self.register.get_a(),
            SubtractSource::B => self.register.get_b(),
            SubtractSource::C => self.register.get_c(),
            SubtractSource::D => self.register.get_d(),
            SubtractSource::E => self.register.get_e(),
            SubtractSource::H => self.register.get_h(),
            SubtractSource::L => self.register.get_l(),
            SubtractSource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n
            }
            SubtractSource::HLAddr => self.read(&self.register.get_hl()),
        };
        let (result, c, h) = Self::subtract(&self.register.get_a(), &source);

        self.register.set_a(result);
        self.update_flag(Flag::C, c);
        self.update_flag(Flag::H, h);
        self.set_flag(Flag::N);
        self.update_flag(Flag::Z, result == 0);
    }
    pub(super) fn push(&mut self, target: PushTarget) {
        let (msb, lsb) = match target {
            PushTarget::BC => (self.register.get_b(), self.register.get_c()),
            PushTarget::DE => (self.register.get_d(), self.register.get_e()),
            PushTarget::HL => (self.register.get_h(), self.register.get_l()),
            PushTarget::AF => (self.register.get_a(), self.register.get_f()),
        };

        self.stack_pointer -= 1;
        let sp = self.stack_pointer;
        self.write(&sp, msb);
        self.stack_pointer -= 1;
        let sp = self.stack_pointer;
        self.write(&sp, lsb);
    }
    pub(super) fn rotate_left(&mut self, source: BitwiseSource) {
        let c = match source {
            BitwiseSource::B => {
                let c = self.register.get_b() >> 7;
                self.register.set_b(self.register.get_b() << 1);
                c
            }
            BitwiseSource::C => {
                let c = self.register.get_c() >> 7;
                self.register.set_c(self.register.get_c() << 1);
                c
            }
            BitwiseSource::D => {
                let c = self.register.get_d() >> 7;
                self.register.set_d(self.register.get_d() << 1);
                c
            }
            BitwiseSource::E => {
                let c = self.register.get_e() >> 7;
                self.register.set_e(self.register.get_e() << 1);
                c
            }
            BitwiseSource::H => {
                let c = self.register.get_h() >> 7;
                self.register.set_h(self.register.get_h() << 1);
                c
            }
            BitwiseSource::L => {
                let c = self.register.get_l() >> 7;
                self.register.set_l(self.register.get_l() << 1);
                c
            }
            BitwiseSource::A => {
                let c = self.register.get_a() >> 7;
                self.register.set_a(self.register.get_a() << 1);
                c
            }
            BitwiseSource::HLAddr => {
                let n = self.read(&self.register.get_hl());
                let c = n >> 7;
                self.write(&self.register.get_hl(), n << 1);
                c
            }
        };

        self.update_flag(Flag::C, c == 1);
    }
    pub(super) fn pop(&mut self, target: PopTarget) {
        let n = self.read(&self.stack_pointer) as u16;
        self.stack_pointer += 1;
        let n = n | ((self.read(&self.stack_pointer) as u16) << 8);
        self.stack_pointer += 1;

        match target {
            PopTarget::BC => self.register.set_bc(n),
            PopTarget::DE => self.register.set_de(n),
            PopTarget::HL => self.register.set_hl(n),
            PopTarget::AF => self.register.set_af(n),
        };
    }
    pub(super) fn decrement(&mut self, target: DecrementTarget) {
        match target {
            DecrementTarget::BC => {
                let (result, _) = self.register.get_bc().overflowing_sub(1);
                self.register.set_bc(result);
            }
            DecrementTarget::DE => {
                let (result, _) = self.register.get_de().overflowing_sub(1);
                self.register.set_de(result);
            }
            DecrementTarget::HL => {
                let (result, _) = self.register.get_hl().overflowing_sub(1);
                self.register.set_hl(result);
            }
            DecrementTarget::SP => {
                let (result, _) = self.stack_pointer.overflowing_sub(1);
                self.stack_pointer = result;
            }
            DecrementTarget::B => {
                let (results, _, h) = Self::subtract(&self.register.get_b(), &1);
                self.register.set_b(results);
                self.update_flag(Flag::Z, results == 0);
                self.set_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            DecrementTarget::D => {
                let (results, _, h) = Self::subtract(&self.register.get_d(), &1);
                self.register.set_d(results);
                self.update_flag(Flag::Z, results == 0);
                self.set_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            DecrementTarget::H => {
                let (results, _, h) = Self::subtract(&self.register.get_h(), &1);
                self.register.set_h(results);
                self.update_flag(Flag::Z, results == 0);
                self.set_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            DecrementTarget::C => {
                let (results, _, h) = Self::subtract(&self.register.get_c(), &1);
                self.register.set_c(results);
                self.update_flag(Flag::Z, results == 0);
                self.set_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            DecrementTarget::E => {
                let (results, _, h) = Self::subtract(&self.register.get_e(), &1);
                self.register.set_e(results);
                self.update_flag(Flag::Z, results == 0);
                self.set_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            DecrementTarget::L => {
                let (results, _, h) = Self::subtract(&self.register.get_l(), &1);
                self.register.set_l(results);
                self.update_flag(Flag::Z, results == 0);
                self.set_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            DecrementTarget::A => {
                let (results, _, h) = Self::subtract(&self.register.get_a(), &1);
                self.register.set_a(results);
                self.update_flag(Flag::Z, results == 0);
                self.set_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            DecrementTarget::HLAddr => {
                let addr = self.register.get_hl();
                let (results, _, h) = Self::subtract(&self.read(&addr), &1);
                self.write(&addr, results);
                self.update_flag(Flag::Z, results == 0);
                self.set_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
        }
    }
    pub(super) fn add(&mut self, target: AddTarget, source: AddSource) {
        let source = match source {
            AddSource::BC => self.register.get_bc(),
            AddSource::DE => self.register.get_de(),
            AddSource::HL => self.register.get_hl(),
            AddSource::SP => self.stack_pointer,
            AddSource::A => self.register.get_a() as u16,
            AddSource::B => self.register.get_b() as u16,
            AddSource::C => self.register.get_c() as u16,
            AddSource::D => self.register.get_d() as u16,
            AddSource::E => self.register.get_e() as u16,
            AddSource::H => self.register.get_h() as u16,
            AddSource::L => self.register.get_l() as u16,
            AddSource::PC | AddSource::PCe => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n as u16
            }
            AddSource::HLAddr => self.read(&self.register.get_hl()) as u16,
        };

        match target {
            AddTarget::SP => {
                let n = source as i8;
                let (result, _) = self.stack_pointer.overflowing_add_signed(n as i16);
                self.update_flag(
                    Flag::H,
                    (result as i16 ^ self.stack_pointer as i16 ^ n as i16) & 0x10 == 0x10,
                );
                self.update_flag(
                    Flag::C,
                    (result as i16 ^ self.stack_pointer as i16 ^ n as i16) & 0x100 == 0x100,
                );
                self.clear_flag(Flag::Z);
                self.clear_flag(Flag::N);
                self.stack_pointer = result;
            }
            AddTarget::A => {
                let (result, c, h) = Self::addition(&self.register.get_a(), &(source as u8));
                self.register.set_a(result);
                self.update_flag(Flag::C, c);
                self.update_flag(Flag::H, h);
                self.update_flag(Flag::Z, result == 0);
                self.clear_flag(Flag::N);
            }
            AddTarget::HL => {
                let (result, c) = self.register.get_hl().overflowing_add(source);
                let h = (result ^ self.register.get_hl() ^ source) & 0x1000 == 0x1000;
                self.update_flag(Flag::C, c);
                self.update_flag(Flag::H, h);
                self.register.set_hl(result);
                self.clear_flag(Flag::Z);
                self.clear_flag(Flag::N);
            }
        };
    }
    pub(super) fn add_carry(&mut self, source: AddCarrySource) {
        let source = match source {
            AddCarrySource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n
            }
            AddCarrySource::A => self.register.get_a(),
            AddCarrySource::B => self.register.get_b(),
            AddCarrySource::C => self.register.get_c(),
            AddCarrySource::D => self.register.get_d(),
            AddCarrySource::E => self.register.get_e(),
            AddCarrySource::H => self.register.get_h(),
            AddCarrySource::L => self.register.get_l(),
            AddCarrySource::HLAddr => self.read(&self.register.get_hl()),
        };
        let source = source + self.get_flag(Flag::C) as u8;
        let (result, c, h) = Self::addition(&self.register.get_a(), &source);
        self.register.set_a(result);
        self.update_flag(Flag::C, c);
        self.update_flag(Flag::H, h);
        self.update_flag(Flag::Z, result == 0);
        self.clear_flag(Flag::N);
    }
    pub(super) fn and(&mut self, source: AndSource) {
        let source = match source {
            AndSource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n
            }
            AndSource::A => self.register.get_a(),
            AndSource::B => self.register.get_b(),
            AndSource::C => self.register.get_c(),
            AndSource::D => self.register.get_d(),
            AndSource::E => self.register.get_e(),
            AndSource::H => self.register.get_h(),
            AndSource::L => self.register.get_l(),
            AndSource::HLAddr => self.read(&self.register.get_hl()),
        };

        let results = self.register.get_a() & source;
        self.register.set_a(results);

        self.set_flag(Flag::H);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::C);
        self.update_flag(Flag::Z, results == 0);
    }
    pub(super) fn complement_accumulator(&mut self) {
        self.register.set_a(!self.register.get_a());
        self.set_flag(Flag::N);
        self.set_flag(Flag::H);
    }
    pub(super) fn complement_carry_flag(&mut self) {
        self.update_flag(Flag::C, !self.get_flag(Flag::C));
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);
    }
    pub(super) fn decimal_adjust_accumulator(&mut self) {
        let mut c = false;
        let mut n = 0u8;
        let a = self.register.get_a();
        let lower = a & 0xf;
        let upper = a >> 4;
        let carry = self.get_flag(Flag::C);
        let half = self.get_flag(Flag::H);

        if self.get_flag(Flag::N) {
            match upper {
                0x0..=0x9 if (0x0..=0x9).contains(&lower) && !half && !carry => {
                    n = 0;
                    c = false;
                }
                0x0..=0x8 if (0x6..=0xf).contains(&lower) && half && !carry => {
                    n = 0xFA;
                    c = false;
                }
                0x7..=0xf if (0x0..=0x9).contains(&lower) && !half && carry => {
                    n = 0xA0;
                    c = true;
                }
                0x6..=0xf if (0x6..=0xf).contains(&lower) && half && carry => {
                    n = 0x9a;
                    c = true;
                }
                _ => (),
            }
        } else {
            match lower {
                0x0..=0x9 if (0x0..=0x9).contains(&upper) && !half && !carry => {
                    n = 0;
                    c = false;
                }
                0xa..=0xf if (0x0..=0x8).contains(&upper) && !half && !carry => {
                    n = 0x06;
                    c = false;
                }
                0x0..=0x3 if (0x0..=0x9).contains(&upper) && half && !carry => {
                    n = 0x06;
                    c = false;
                }
                0x0..=0x9 if (0xa..=0xf).contains(&upper) && !half && !carry => {
                    n = 0x60;
                    c = true;
                }
                0xa..=0xf if (0x9..=0xf).contains(&upper) && !half && !carry => {
                    n = 0x66;
                    c = true;
                }
                0x0..=0x3 if (0xa..=0xf).contains(&upper) && half && !carry => {
                    n = 0x66;
                    c = true;
                }
                0x0..=0x9 if (0x0..=0x2).contains(&upper) && !half && carry => {
                    n = 0x60;
                    c = true;
                }
                0xa..=0xf if (0x0..=0x2).contains(&upper) && !half && carry => {
                    n = 0x66;
                    c = true;
                }
                0x0..=0x3 if (0x0..=0x3).contains(&upper) && half && carry => {
                    n = 0x66;
                    c = true;
                }
                _ => (),
            }
        }
        let (results, _) = self.register.get_a().overflowing_add(n);
        self.clear_flag(Flag::H);
        self.update_flag(Flag::C, c);
        self.update_flag(Flag::Z, results == 0);
        self.register.set_a(results);
    }
    pub(super) fn restart(&mut self, addr: u16) {
        self.stack_pointer -= 1;
        let sp = self.stack_pointer;
        let pc_high = (self.program_counter >> 8) as u8;
        self.write(&sp, pc_high);
        self.stack_pointer -= 1;
        let sp = self.stack_pointer;
        let pc_low = self.program_counter as u8;
        self.write(&sp, pc_low);

        self.program_counter = addr;
    }
    pub(super) fn ret(&mut self, condition: JumpCondition) {
        if condition == JumpCondition::None {
            let n = self.read(&self.stack_pointer) as u16;
            self.stack_pointer += 1;
            let n = n | ((self.read(&self.stack_pointer) as u16) << 8);
            self.stack_pointer += 1;
            self.program_counter = n;
            return;
        };

        let condition = match condition {
            JumpCondition::None => unreachable!(),
            JumpCondition::Z => self.get_flag(Flag::Z),
            JumpCondition::C => self.get_flag(Flag::C),
            JumpCondition::NZ => !self.get_flag(Flag::Z),
            JumpCondition::NC => !self.get_flag(Flag::C),
        };

        if condition {
            self.instruction = Instruction::Return(JumpCondition::None);
            self.step_count = 3;
        }
    }
    pub(super) fn jump_hl(&mut self) {
        self.program_counter = self.register.get_hl();
    }
    pub(super) fn rotate(&mut self, source: BitwiseSource, rotate: impl Fn(&mut Cpu, u8) -> u8) {
        match source {
            BitwiseSource::B => {
                let n = rotate(self, self.register.get_b());
                self.register.set_b(n);
            }
            BitwiseSource::C => {
                let n = rotate(self, self.register.get_c());
                self.register.set_c(n);
            }
            BitwiseSource::D => {
                let n = rotate(self, self.register.get_d());
                self.register.set_d(n);
            }
            BitwiseSource::E => {
                let n = rotate(self, self.register.get_e());
                self.register.set_e(n);
            }
            BitwiseSource::H => {
                let n = rotate(self, self.register.get_h());
                self.register.set_h(n);
            }
            BitwiseSource::L => {
                let n = rotate(self, self.register.get_l());
                self.register.set_l(n);
            }
            BitwiseSource::A => {
                let n = rotate(self, self.register.get_a());
                self.register.set_a(n);
            }
            BitwiseSource::HLAddr => {
                let n = rotate(self, self.read(&self.register.get_hl()));
                self.write(&self.register.get_hl(), n);
            }
        };
    }
    pub(super) fn sub_carry(&mut self, source: SubtractCarrySource) {
        let source = match source {
            SubtractCarrySource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n
            }
            SubtractCarrySource::A => self.register.get_a(),
            SubtractCarrySource::B => self.register.get_b(),
            SubtractCarrySource::C => self.register.get_c(),
            SubtractCarrySource::D => self.register.get_d(),
            SubtractCarrySource::E => self.register.get_e(),
            SubtractCarrySource::H => self.register.get_h(),
            SubtractCarrySource::L => self.register.get_l(),
            SubtractCarrySource::HLAddr => self.read(&self.register.get_hl()),
        };
        let source = source + self.get_flag(Flag::C) as u8;

        let (result, c, h) = Self::subtract(&self.register.get_a(), &source);

        self.register.set_a(result);
        self.update_flag(Flag::C, c);
        self.update_flag(Flag::H, h);
        self.update_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N);
    }
    pub(super) fn swap(&mut self, source: BitwiseSource) {
        let swap = |cpu: &mut Cpu, value: u8| {
            cpu.clear_flag(Flag::C);
            cpu.clear_flag(Flag::H);
            cpu.clear_flag(Flag::N);
            cpu.update_flag(Flag::Z, value == 0);

            let lower = value >> 4;
            let upper = value << 4;
            lower | upper
        };
        match source {
            BitwiseSource::B => {
                let n = swap(self, self.register.get_b());
                self.register.set_b(n);
            }
            BitwiseSource::C => {
                let n = swap(self, self.register.get_c());
                self.register.set_c(n);
            }
            BitwiseSource::D => {
                let n = swap(self, self.register.get_d());
                self.register.set_d(n);
            }
            BitwiseSource::E => {
                let n = swap(self, self.register.get_e());
                self.register.set_e(n);
            }
            BitwiseSource::H => {
                let n = swap(self, self.register.get_h());
                self.register.set_h(n);
            }
            BitwiseSource::L => {
                let n = swap(self, self.register.get_l());
                self.register.set_l(n);
            }
            BitwiseSource::A => {
                let n = swap(self, self.register.get_a());
                self.register.set_a(n);
            }
            BitwiseSource::HLAddr => {
                let n = swap(self, self.read(&self.register.get_hl()));
                self.write(&self.register.get_hl(), n);
            }
        }
    }
    pub(super) fn or(&mut self, source: OrSource) {
        let source = match source {
            OrSource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n
            }
            OrSource::A => self.register.get_a(),
            OrSource::B => self.register.get_b(),
            OrSource::C => self.register.get_c(),
            OrSource::D => self.register.get_d(),
            OrSource::E => self.register.get_e(),
            OrSource::H => self.register.get_h(),
            OrSource::L => self.register.get_l(),
            OrSource::HLAddr => self.read(&self.register.get_hl()),
        };

        let n = self.register.get_a() | source;
        self.clear_flag(Flag::C);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);
        self.update_flag(Flag::Z, n == 0);
        self.register.set_a(n);
    }
    pub(super) fn jump(&mut self, condition: JumpCondition) {
        if condition == JumpCondition::None {
            let n = self.read(&self.program_counter) as u16;
            self.program_counter += 1;
            let n = n | ((self.read(&self.program_counter) as u16) << 8);
            self.program_counter = n;
            return;
        }

        let condition = match condition {
            JumpCondition::None => unreachable!(),
            JumpCondition::Z => self.get_flag(Flag::Z),
            JumpCondition::C => self.get_flag(Flag::C),
            JumpCondition::NZ => !self.get_flag(Flag::Z),
            JumpCondition::NC => !self.get_flag(Flag::C),
        };

        if condition {
            self.instruction = Instruction::Jump(JumpCondition::None);
            self.step_count = 1;
        } else {
            self.program_counter += 2;
        }
    }
}
