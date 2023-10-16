#include "cpu.h"

namespace cash::GB
{
  bool CPU::isBit16(const Register &reg) const
  {
    switch (reg)
    {
    case Register::AF:
      return true;
    case Register::BC:
      return true;
    case Register::DE:
      return true;
    case Register::HL:
      return true;
    default:
      return false;
    }
  }

  void CPU::process(const Instruction &inst)
  {
    switch (inst.type)
    {
    case InstType::Invalid:
      throw std::logic_error("invalid instruction: " + inst.name);
      break;
    case InstType::NOP:
      NOP(inst);
      break;
    case InstType::LD:
      LD(inst);
      break;
    case InstType::LDH:
      LDH(inst);
      break;
    case InstType::LDHLSP:
      LDHLSP(inst);
      break;
    case InstType::PUSH:
      PUSH(inst);
      break;
    case InstType::POP:
      POP(inst);
      break;
    case InstType::ADD:
      ADD(inst);
      break;
    case InstType::ADC:
      ADC(inst);
      break;
    case InstType::SUB:
      SUB(inst);
      break;
    case InstType::SBC:
      SBC(inst);
      break;
    case InstType::CP:
      CP(inst);
      break;
    case InstType::INC:
      INC(inst);
      break;
    case InstType::DEC:
      DEC(inst);
      break;
    case InstType::AND:
      AND(inst);
      break;
    case InstType::OR:
      OR(inst);
      break;
    case InstType::XOR:
      XOR(inst);
      break;
    case InstType::CCF:
      CCF(inst);
      break;
    case InstType::DAA:
      DAA(inst);
      break;
    case InstType::CPL:
      CPL(inst);
      break;
    case InstType::JP:
      JP(inst);
      break;
    case InstType::JR:
      JR(inst);
      break;
    case InstType::CALL:
      CALL(inst);
      break;
    case InstType::RET:
      RET(inst);
      break;
    case InstType::RETI:
      RETI(inst);
      break;
    case InstType::RST:
      RST(inst);
      break;
    case InstType::HALT:
      HALT(inst);
      break;
    case InstType::STOP:
      STOP(inst);
      break;
    case InstType::DI:
      DI(inst);
      break;
    case InstType::EI:
      EI(inst);
      break;
    case InstType::RLC:
      RLC(inst);
      break;
    case InstType::RRC:
      RRC(inst);
      break;
    case InstType::RL:
      RL(inst);
      break;
    case InstType::RR:
      RR(inst);
      break;
    case InstType::SCF:
      SCF(inst);
      break;

    default:
      break;
    }
  }

  bit16 CPU::regRead(const Register &reg) const
  {
    switch (reg)
    {
    case Register::A:
      return A;
    case Register::B:
      return B;
    case Register::C:
      return C;
    case Register::D:
      return D;
    case Register::E:
      return E;
    case Register::F:
      return F;
    case Register::H:
      return H;
    case Register::L:
      return L;
    case Register::SP:
      return SP;
    case Register::PC:
      return PC;
    case Register::AF:
      return (A << 8) | F;
    case Register::BC:
      return (B << 8) | C;
    case Register::DE:
      return (D << 8) | E;
    case Register::HL:
      return (H << 8) | L;
    default:
      return 0x00;
    }
  }

  void CPU::clock()
  {
    if (cycle > 0)
    {
      cycle--;
      return;
    }

    bit8 nextInst = bus.read(PC++);

    currentInstruction = &instructions[nextInst];
    cycle = currentInstruction->cycles;
  }

  bit8 CPU::read(const bit16 &addr) const
  {
    // todo: CPU registers ?
    return 0x00;
  }

  bit8 CPU::fetch(const Instruction &instr)
  {
    isMemAddr = false;
    switch (instr.addrMode)
    {
    case AddrMode::Impl:
      break;
    case AddrMode::Reg:
    case AddrMode::RegToReg:
      value = regRead(instr.startReg);
      break;
    case AddrMode::Bit8:
    case AddrMode::Bit8ToReg:
      value = bus.read(PC++);
      break;
    case AddrMode::MemReg:
    case AddrMode::MemRegToReg:
    {
      bit16 loc = regRead(instr.startReg);
      if (!isBit16(instr.startReg))
        loc |= 0xFF00;
      value = bus.read(loc);
      if (isBit16(instr.destReg))
        value |= bus.read(loc + 1) << 8;
      break;
    }
    case AddrMode::MemRegToMemReg:
    {
      bit16 loc = regRead(instr.startReg);
      value = bus.read(loc);
      memLoc = regRead(instr.destReg);
      isMemAddr = true;
    }
    case AddrMode::RegToMemReg:
      value = regRead(instr.startReg);
      memLoc = regRead(instr.destReg);
      if (!isBit16(instr.destReg))
        memLoc |= 0xFF00;

      isMemAddr = true;
      break;
    case AddrMode::Bit8ToMemReg:
      value = bus.read(PC++);
      memLoc = regRead(instr.destReg);
      isMemAddr = true;
      break;
    case AddrMode::MemBit16ToReg:
    {
      bit16 loc = bus.read(PC++) | (bus.read(PC++) << 8);
      value = bus.read(loc);
      break;
    }
    case AddrMode::RegToMemBit16:
      value = regRead(instr.startReg);
      memLoc = bus.read(PC++) | (bus.read(PC++) << 8);
      isMemAddr = true;
      break;

    case AddrMode::MemBit8ToReg:
    {
      auto loc = 0xFF00 | bus.read(PC++);
      value = bus.read(loc);
      break;
    }
    case AddrMode::RegToMemBit8:
      value = regRead(instr.startReg);
      memLoc = 0xFF00 | bus.read(PC++);
      isMemAddr = true;
      break;
    case AddrMode::MemRegDecToReg:
    {
      auto loc = regRead(instr.startReg);
      value = bus.read(loc--);
      regWrite(instr.startReg, loc);
      break;
    }
    case AddrMode::RegToMemRegDec:
    {
      value = regRead(instr.startReg);
      memLoc = regRead(instr.startReg);
      regWrite(instr.startReg, memLoc - 1);
      break;
    }
    case AddrMode::RegToMemRegInc:
    {
      memLoc = regRead(instr.destReg);
      value = regRead(instr.startReg);
      isMemAddr = true;
      regWrite(instr.destReg, memLoc + 1);
      break;
    }
    case AddrMode::MemRegIncToReg:
    {
      auto loc = regRead(instr.startReg);
      value = bus.read(loc++);
      regWrite(instr.startReg, loc);
      break;
    }
    case AddrMode::Bit16:
    case AddrMode::Bit16ToReg:
      value = bus.read(PC++) | (bus.read(PC++) << 8);
      break;
    case AddrMode::RegToBit16:
      value = regRead(instr.startReg);
      memLoc = bus.read(PC++) | (bus.read(PC++) << 8);
      isMemAddr = true;
      break;
    }

    return 0x00;
  }

  void CPU::regWrite(const Register &reg, const bit16 &value)
  {
    bit8 high = value >> 8;
    bit8 low = value;
    switch (reg)
    {
    case Register::A:
      A = low;
      break;
    case Register::B:
      B = low;
      break;
    case Register::C:
      C = low;
      break;
    case Register::D:
      D = low;
      break;
    case Register::E:
      E = low;
      break;
    case Register::F:
      F = low;
      break;
    case Register::H:
      H = low;
      break;
    case Register::L:
      L = low;
      break;
    case Register::SP:
      SP = value;
      break;
    case Register::PC:
      PC = value;
      break;
    case Register::AF:
      A = high;
      F = low;
      break;
    case Register::BC:
      B = high;
      C = low;
      break;
    case Register::DE:
      D = high;
      E = low;
      break;
    case Register::HL:
      H = high;
      L = low;
      break;
    }
  }

  void CPU::LD(const Instruction &inst)
  {
    if (isMemAddr)
    {
      bus.write(memLoc, value);
      if (isBit16(inst.startReg))
        bus.write(memLoc + 1, value >> 8);
    }
    else
      regWrite(inst.destReg, value);
    return;
  }

  void CPU::LDH(const Instruction &inst)
  {
    if (isMemAddr)
      bus.write(memLoc, value);
    else
      regWrite(inst.destReg, value);
    return;
  }

  void CPU::LDHLSP(const Instruction & inst)
  {
    auto sum = value + SP;
    auto carry = value ^ SP ^ sum;
    regWrite(Register::HL, sum);

    flags.C = carry >> 7;
    flags.H = carry >> 3;
    flags.Z = 0;
    flags.N = 0;
  }

  void CPU::PUSH(const Instruction &inst)
  {
    SP--;
    bus.write(SP--, value >> 8);
    bus.write(SP, value);
    return;
  }

  void CPU::POP(const Instruction &inst)
  {
    regWrite(inst.destReg, value);
    SP += 2;
    return;
  }

  void CPU::ADD(const Instruction &inst)
  {
    auto existing = regRead(inst.destReg);
    auto sum = existing + value;
    auto carry = existing ^ value ^ sum;

    flags.Z = sum == 0;
    flags.C = carry >> 7;
    flags.H = carry >> 3;
    flags.N = 0;

    regWrite(inst.destReg, sum);
  }

  void CPU::ADC(const Instruction &inst)
  {
    auto existing = regRead(inst.destReg);
    value += flags.C;
    auto sum = existing + value;
    auto carry = existing ^ value ^ sum;
    flags.Z = sum == 0;
    flags.C = carry >> 7;
    flags.H = carry >> 3;
    flags.N = 0;

    regWrite(inst.destReg, sum);
  }

  void CPU::SUB(const Instruction &inst)
  {
    auto existing = regRead(inst.destReg);
    auto sum = existing - value;
    flags.Z = sum == 0;
    flags.C = existing < value;
    flags.H = (existing & 0x0F) < (value & 0x0F);
    flags.N = 1;

    regWrite(inst.destReg, sum);
  }

  void CPU::SBC(const Instruction &inst)
  {
    auto existing = regRead(inst.destReg);
    value = value + flags.C;
    auto sum = existing - value;
    flags.Z = sum == 0;
    flags.C = existing < value;
    flags.H = existing & 0x0F < value & 0x0F;
    flags.N = 1;

    regWrite(inst.destReg, sum);
  }

  void CPU::CP(const Instruction &inst)
  {
    auto existing = regRead(inst.destReg);
    value = ~value + 1;
    auto sum = existing + value;
    auto carry = existing ^ value ^ sum;
    flags.Z = sum == 0;
    flags.C = carry >> 7;
    flags.H = carry >> 3;
    flags.N = 1;
  }

  void CPU::INC(const Instruction &inst)
  {
    auto sum = value + 1;
    auto carry = value ^ 1 ^ sum;
    flags.Z = sum == 0;
    flags.H = carry & 0x10;
    flags.N = 0;

    regWrite(inst.startReg, sum);
  }

  void CPU::DEC(const Instruction &inst)
  {
    auto sum = value + 0xFF;
    auto carry = value ^ 0xFF ^ sum;
    flags.Z = sum == 0;
    flags.H = carry & 0x10;
    flags.N = 1;

    regWrite(inst.startReg, sum);
  }

  void CPU::AND(const Instruction &inst)
  {
    bit16 existing = regRead(inst.destReg);
    bit16 results = existing & value;
    flags.Z = results == 0;
    flags.N = 0;
    flags.H = 1;
    flags.C = 0;

    regWrite(inst.destReg, results);
  }

  void CPU::OR(const Instruction &inst)
  {
    bit16 existing = regRead(inst.destReg);
    bit16 results = existing | value;
    flags.Z = results == 0;
    flags.N = 0;
    flags.H = 0;
    flags.C = 0;

    regWrite(inst.destReg, results);
  }

  void CPU::XOR(const Instruction &inst)
  {
    bit16 existing = regRead(inst.destReg);
    bit16 results = existing ^ value;
    flags.Z = results == 0;
    flags.N = 0;
    flags.H = 0;
    flags.C = 0;

    regWrite(inst.destReg, results);
  }

  void CPU::CCF(const Instruction &inst)
  {
    flags.C = ~flags.C;
    flags.N = 0;
    flags.H = 0;
  }

  void CPU::DAA(const Instruction &inst)
  {
    bit8 low = A & 0x0F;
    bit8 high = A >> 4;
    bit8 add = 0x0;
    if ((low > 9 & !flags.N) || flags.H)
      add += 0x06;
    if ((high > 9 & !flags.N) || flags.C)
    {
      add += 0x60;
      flags.C = 1;
    }

    A += (flags.N ? -add : add);

    flags.Z = A == 0;
    flags.H = 0;
  }

  void CPU::CPL(const Instruction &inst)
  {
    flags.N = 1;
    flags.H = 1;
    A = ~A;
  }

  void CPU::JP(const Instruction &inst)
  {
    bool condition = false;
    switch (inst.jmp)
    {
    case JmpCondition::C:
      condition = flags.C;
      break;
    case JmpCondition::NC:
      condition = !flags.C;
      break;
    case JmpCondition::Z:
      condition = flags.Z;
      break;
    case JmpCondition::NZ:
      condition = !flags.Z;
      break;
    }
    if (condition)
      cycle += 1;
    if (condition || inst.jmp == JmpCondition::None)
      PC = value;
  }

  void CPU::JR(const Instruction &inst)
  {
    bool condition = false;
    switch (inst.jmp)
    {
    case JmpCondition::C:
      condition = flags.C;
      break;
    case JmpCondition::NC:
      condition = !flags.C;
      break;
    case JmpCondition::Z:
      condition = flags.Z;
      break;
    case JmpCondition::NZ:
      condition = !flags.Z;
      break;
    }
    if (condition)
      cycle += 1;
    if (condition || inst.jmp == JmpCondition::None)
      PC += value;
  }

  void CPU::CALL(const Instruction &inst)
  {
    bool condition = false;
    switch (inst.jmp)
    {
    case JmpCondition::C:
      condition = flags.C;
      break;
    case JmpCondition::NC:
      condition = !flags.C;
      break;
    case JmpCondition::Z:
      condition = flags.Z;
      break;
    case JmpCondition::NZ:
      condition = !flags.Z;
      break;
    }
    if (condition)
      cycle += 3;
    if (condition || inst.jmp == JmpCondition::None)
    {
      SP--;
      bus.write(SP--, PC >> 8);
      bus.write(SP, PC);
      PC = value;
    }
  }

  void CPU::RET(const Instruction &inst)
  {
    bool condition = false;
    switch (inst.jmp)
    {
    case JmpCondition::C:
      condition = flags.C;
      break;
    case JmpCondition::NC:
      condition = !flags.C;
      break;
    case JmpCondition::Z:
      condition = flags.Z;
      break;
    case JmpCondition::NZ:
      condition = !flags.Z;
      break;
    }
    if (condition)
      cycle += 3;
    if (condition || inst.jmp == JmpCondition::None)
      PC = bus.read(SP++) | (bus.read(SP++) << 8);
  }

  void CPU::RETI(const Instruction &inst)
  {
    IME = true;
    PC = bus.read(SP++) | (bus.read(SP++) << 8);
  }

  void CPU::RST(const Instruction &inst)
  {
    SP--;
    bus.write(SP--, PC >> 8);
    bus.write(SP, PC);
    PC = inst.rstAddr;
  }

  void CPU::HALT(const Instruction &inst)
  {
    stopSystem = true;
  }

  void CPU::STOP(const Instruction &inst)
  {
    stopMain = true;
    stopSystem = true;
  }

  void CPU::DI(const Instruction &inst)
  {
    IME = false;
  }

  void CPU::EI(const Instruction &inst)
  {
    IME = true;
  }

  void CPU::NOP(const Instruction &inst)
  {
  }

  void CPU::RLC(const Instruction &inst)
  {
    auto results = value << 1;
    flags.C = (value >> 7);
    results |= flags.C;
    if (isMemAddr)
      bus.write(memLoc, results);
    else
      regWrite(inst.destReg, results);
    flags.H = 0;
    flags.N = 0;
    if (inst.destReg != Register::A)
      flags.Z = results == 0;
  }

  void CPU::RL(const Instruction &inst)
  {
    auto results = value << 1;
    results |= flags.C;
    flags.C = (value >> 7);
    regWrite(inst.destReg, results);
    flags.H = 0;
    flags.N = 0;
    if (inst.destReg != Register::A)
      flags.Z = results == 0;
  }

  void CPU::RRC(const Instruction &inst)
  {
    auto results = value >> 1;
    flags.C = value;
    results |= flags.C << 7;
    regWrite(inst.destReg, results);
    flags.H = 0;
    flags.N = 0;
  }

  void CPU::RR(const Instruction &inst)
  {
    auto results = value >> 1;
    results |= flags.C << 7;
    flags.C = value;
    regWrite(inst.destReg, results);
    flags.H = 0;
    flags.N = 0;
  }

  void CPU::SCF(const Instruction &inst)
  {
    flags.H = 0;
    flags.N = 0;
    flags.C = 1;
  }

  const Instruction CPU::instructions[256] = {
    //$0x
    {"NOP", 1, InstType::NOP},
    {"LD BC, n16", 3, InstType::LD, AddrMode::Bit16ToReg, Register::BC, Register::NONE},
    {"LD [BC], A",2,InstType::LD,AddrMode::RegToMemReg,Register::BC,Register::A,},
    {"INC BC",2,InstType::INC,AddrMode::Reg,Register::BC,Register::BC,},
    {"INC B",1,InstType::INC,AddrMode::Reg,Register::B,Register::B,},
    {"DEC B",1,InstType::DEC,AddrMode::Reg,Register::B,Register::B,},
    {"LD B, n8",2,InstType::LD,AddrMode::Bit8ToReg,Register::B},
    {"RLCA",1,InstType::RLC,AddrMode::Reg,Register::A,Register::A,},
    {"LD [a16], SP",5,InstType::LD,AddrMode::RegToMemBit16,Register::NONE,Register::SP},
    {"ADD HL, BC",2,InstType::ADD,AddrMode::RegToReg,Register::HL,Register::BC},
    {"LD A, [BC]",2,InstType::LD,AddrMode::MemRegToReg,Register::A,Register::BC},
    {"DEC BC",2,InstType::DEC,AddrMode::Reg,Register::BC,Register::BC},
    {"INC C",1,InstType::INC,AddrMode::Reg,Register::C,Register::C},
    {"DEC C",1,InstType::DEC,AddrMode::Reg,Register::C,Register::C,},
    {"LD C, n8",2,InstType::LD,AddrMode::Bit8ToReg,Register::C,},
    {"RRCA",1,InstType::RRC,AddrMode::Reg,Register::A,Register::A,},
    //$1x
    {"STOP n8",2,InstType::STOP,AddrMode::Bit8,},
    {"LD DE, n16",3,InstType::LD,AddrMode::Bit16ToReg,Register::DE,},
    {"LD [DE], A",2,InstType::LD,AddrMode::RegToMemReg,Register::DE,Register::A,},
    {"INC DE",2,InstType::INC,AddrMode::RegToReg,Register::DE,Register::DE},
    {"INC D",1,InstType::INC,AddrMode::RegToReg,Register::D,Register::D,},
    {"DEC D",1,InstType::DEC,AddrMode::RegToReg,Register::D,Register::D,},
    {"LD D, n8",2,InstType::LD,AddrMode::Bit8ToReg,Register::D,},
    {"RLA",1,InstType::RL,AddrMode::RegToReg,Register::A,Register::A,},
    {"JR e8",3,InstType::JR,AddrMode::Bit8,},
    {"ADD HL DE",2,InstType::ADD,AddrMode::RegToReg,Register::HL,Register::DE,},
    {"LD A [DE]",2,InstType::LD,AddrMode::MemRegToReg,Register::A,Register::DE,},
    {"DEC DE",2,InstType::DEC,AddrMode::RegToReg,Register::DE,Register::DE,},
    {"INC E",1,InstType::INC,AddrMode::RegToReg,Register::E,Register::E,},
    {"DEC E",1,InstType::DEC,AddrMode::RegToReg,Register::E,Register::E,},
    {"LD E, n8",2,InstType::LD,AddrMode::Bit8ToReg,Register::E,},
    {"RRA",1,InstType::RR,AddrMode::RegToReg,Register::A,Register::A,},
    //$2x
    {"JR NZ, e8",2,InstType::JR,AddrMode::Bit8,Register::NONE,Register::NONE,JmpCondition::NZ,},
    {"LD HL, n16",3,InstType::LD,AddrMode::Bit16ToReg,Register::HL,},
    {"LD [HL+], A",2,InstType::LD,AddrMode::RegToMemRegInc,Register::HL,Register::A},
    {"INC HL",2,InstType::INC,AddrMode::RegToReg,Register::HL,Register::HL,},
    {"INC H",1,InstType::INC,AddrMode::RegToReg,Register::H,Register::H,},
    {"DEC H",1,InstType::DEC,AddrMode::RegToReg,Register::H,Register::H,},
    {"LD H, n8",2,InstType::LD,AddrMode::Bit8ToReg,Register::H,},
    {"DAA",1,InstType::DAA,AddrMode::Impl,},
    {"JR Z, e8",2,InstType::JR,AddrMode::Bit8,Register::NONE,Register::NONE,JmpCondition::Z,},
    {"ADD HL, HL",2,InstType::ADD,AddrMode::RegToReg,Register::HL,Register::HL,},
    {"LD A, [HL+]",2,InstType::LD,AddrMode::MemRegIncToReg,Register::A,Register::HL},
    {"DEC HL",2,InstType::DEC,AddrMode::RegToReg,Register::HL,Register::HL},
    {"INC L",1,InstType::INC,AddrMode::RegToReg,Register::L,Register::L},
    {"DEC L",1,InstType::DEC,AddrMode::RegToReg,Register::L,Register::L},
    {"LD L, n8",1,InstType::LD,AddrMode::Bit8ToReg,Register::L},
    {"CPL",1,InstType::CPL,AddrMode::Impl},
    //$3x
    {"JR NC, e8",2,InstType::JR,AddrMode::Bit8,Register::NONE,Register::NONE,JmpCondition::NC},
    {"LD SP, n16",3,InstType::LD,AddrMode::Bit16ToReg,Register::SP},
    {"LD [HL-], A",2,InstType::LD,AddrMode::RegToMemRegDec,Register::HL,Register::A},
    {"INC SP",2,InstType::INC,AddrMode::RegToReg,Register::SP,Register::SP},
    {"INC [HL]",3,InstType::INC,AddrMode::MemRegToMemReg,Register::HL,Register::HL},
    {"DEC [HL]",3,InstType::DEC,AddrMode::MemRegToMemReg,Register::HL,Register::HL},
    {"LD [HL], n8",3,InstType::LD,AddrMode::Bit8ToMemReg,Register::HL},
    {"SCF",1,InstType::SCF,AddrMode::Impl},
    {"JR C, e8",2,InstType::JR,AddrMode::Bit8,Register::NONE,Register::NONE,JmpCondition::C},
    {"ADD HL, SP",2,InstType::ADD,AddrMode::RegToReg,Register::HL,Register::SP},
    {"LD A, [HL-]",2,InstType::LD,AddrMode::MemRegDecToReg,Register::A,Register::HL},
    {"DEC SP",2,InstType::DEC,AddrMode::RegToReg,Register::SP,Register::SP},
    {"INC A",1,InstType::INC,AddrMode::RegToReg,Register::A,Register::A},
    {"DEC A",1,InstType::DEC,AddrMode::RegToReg,Register::A,Register::A},
    {"LD A, n8",2,InstType::LD,AddrMode::Bit8ToReg,Register::A},
    {"CCF",1,InstType::CCF,AddrMode::Impl},
    //$4x
    {"LD B, B",1,InstType::LD,AddrMode::RegToReg,Register::B,Register::B},
    {"LD B, C",1,InstType::LD,AddrMode::RegToReg,Register::B,Register::C},
    {"LD B, D",1,InstType::LD,AddrMode::RegToReg,Register::B,Register::D},
    {"LD B, E",1,InstType::LD,AddrMode::RegToReg,Register::B,Register::E},
    {"LD B, H",1,InstType::LD,AddrMode::RegToReg,Register::B,Register::H},
    {"LD B, L",1,InstType::LD,AddrMode::RegToReg,Register::B,Register::L},
    {"LD B, [HL]",2,InstType::LD,AddrMode::MemRegToReg,Register::B,Register::HL},
    {"LD B, A",1,InstType::LD,AddrMode::RegToReg,Register::B,Register::A},
    {"LD C, B",1,InstType::LD,AddrMode::RegToReg,Register::C,Register::B},
    {"LD C, C",1,InstType::LD,AddrMode::RegToReg,Register::C,Register::C},
    {"LD C, D",1,InstType::LD,AddrMode::RegToReg,Register::C,Register::D},
    {"LD C, E",1,InstType::LD,AddrMode::RegToReg,Register::C,Register::E},
    {"LD C, H",1,InstType::LD,AddrMode::RegToReg,Register::C,Register::H},
    {"LD C, L",1,InstType::LD,AddrMode::RegToReg,Register::C,Register::L},
    {"LD C, [HL]",2,InstType::LD,AddrMode::MemRegToReg,Register::C,Register::HL},
    {"LD C, A",1,InstType::LD,AddrMode::RegToReg,Register::C,Register::A},
    //$5x
    {"LD D, B",1,InstType::LD,AddrMode::RegToReg,Register::D,Register::B},
    {"LD D, C",1,InstType::LD,AddrMode::RegToReg,Register::D,Register::C},
    {"LD D, D",1,InstType::LD,AddrMode::RegToReg,Register::D,Register::D},
    {"LD D, E",1,InstType::LD,AddrMode::RegToReg,Register::D,Register::E},
    {"LD D, H",1,InstType::LD,AddrMode::RegToReg,Register::D,Register::H},
    {"LD D, L",1,InstType::LD,AddrMode::RegToReg,Register::D,Register::L},
    {"LD D, [HL]",2,InstType::LD,AddrMode::MemRegToReg,Register::D,Register::HL},
    {"LD D, A",1,InstType::LD,AddrMode::RegToReg,Register::D,Register::A},
    {"LD E, B",1,InstType::LD,AddrMode::RegToReg,Register::E,Register::B},
    {"LD E, C",1,InstType::LD,AddrMode::RegToReg,Register::E,Register::C},
    {"LD E, D",1,InstType::LD,AddrMode::RegToReg,Register::E,Register::D},
    {"LD E, E",1,InstType::LD,AddrMode::RegToReg,Register::E,Register::E},
    {"LD E, H",1,InstType::LD,AddrMode::RegToReg,Register::E,Register::H},
    {"LD E, L",1,InstType::LD,AddrMode::RegToReg,Register::E,Register::L},
    {"LD E, [HL]",2,InstType::LD,AddrMode::MemRegToReg,Register::E,Register::HL},
    {"LD E, A",1,InstType::LD,AddrMode::RegToReg,Register::E,Register::A},
    //$6x
    {"LD H, B",1,InstType::LD,AddrMode::RegToReg,Register::H,Register::B},
    {"LD H, C",1,InstType::LD,AddrMode::RegToReg,Register::H,Register::C},
    {"LD H, D",1,InstType::LD,AddrMode::RegToReg,Register::H,Register::D},
    {"LD H, E",1,InstType::LD,AddrMode::RegToReg,Register::H,Register::E},
    {"LD H, H",1,InstType::LD,AddrMode::RegToReg,Register::H,Register::H},
    {"LD H, L",1,InstType::LD,AddrMode::RegToReg,Register::H,Register::L},
    {"LD H, [HL]",2,InstType::LD,AddrMode::MemRegToReg,Register::H,Register::HL},
    {"LD H, A",1,InstType::LD,AddrMode::RegToReg,Register::H,Register::A},
    {"LD L, B",1,InstType::LD,AddrMode::RegToReg,Register::L,Register::B},
    {"LD L, C",1,InstType::LD,AddrMode::RegToReg,Register::L,Register::C},
    {"LD L, D",1,InstType::LD,AddrMode::RegToReg,Register::L,Register::D},
    {"LD L, E",1,InstType::LD,AddrMode::RegToReg,Register::L,Register::E},
    {"LD L, H",1,InstType::LD,AddrMode::RegToReg,Register::L,Register::H},
    {"LD L, L",1,InstType::LD,AddrMode::RegToReg,Register::L,Register::L},
    {"LD L, [HL]",2,InstType::LD,AddrMode::MemRegToReg,Register::L,Register::HL},
    {"LD L, A",1,InstType::LD,AddrMode::RegToReg,Register::L,Register::A},
    //$7x
    {"LD [HL], B",2,InstType::LD,AddrMode::RegToMemReg,Register::HL,Register::B},
    {"LD [HL], C",2,InstType::LD,AddrMode::RegToMemReg,Register::HL,Register::C},
    {"LD [HL], D",2,InstType::LD,AddrMode::RegToMemReg,Register::HL,Register::D},
    {"LD [HL], E",2,InstType::LD,AddrMode::RegToMemReg,Register::HL,Register::E},
    {"LD [HL], H",2,InstType::LD,AddrMode::RegToMemReg,Register::HL,Register::H},
    {"LD [HL], L",2,InstType::LD,AddrMode::RegToMemReg,Register::HL,Register::L},
    {"HALT",1,InstType::HALT},
    {"LD [HL], A",2,InstType::LD,AddrMode::RegToMemReg,Register::HL,Register::A},
    {"LD A, B",1,InstType::LD,AddrMode::RegToReg,Register::A,Register::B},
    {"LD A, C",1,InstType::LD,AddrMode::RegToReg,Register::A,Register::C},
    {"LD A, D",1,InstType::LD,AddrMode::RegToReg,Register::A,Register::D},
    {"LD A, E",1,InstType::LD,AddrMode::RegToReg,Register::A,Register::E},
    {"LD A, H",1,InstType::LD,AddrMode::RegToReg,Register::A,Register::H},
    {"LD A, L",1,InstType::LD,AddrMode::RegToReg,Register::A,Register::L},
    {"LD A, [HL]",2,InstType::LD,AddrMode::MemRegToReg,Register::A,Register::HL},
    {"LD A, A",1,InstType::LD,AddrMode::RegToReg,Register::A,Register::A},
    //$8x
    {"ADD A, B",1,InstType::ADD,AddrMode::RegToReg,Register::A,Register::B},
    {"ADD A, C",1,InstType::ADD,AddrMode::RegToReg,Register::A,Register::C},
    {"ADD A, D",1,InstType::ADD,AddrMode::RegToReg,Register::A,Register::D},
    {"ADD A, E",1,InstType::ADD,AddrMode::RegToReg,Register::A,Register::E},
    {"ADD A, H",1,InstType::ADD,AddrMode::RegToReg,Register::A,Register::H},
    {"ADD A, L",1,InstType::ADD,AddrMode::RegToReg,Register::A,Register::L},
    {"ADD A, [HL]",2,InstType::ADD,AddrMode::MemRegToReg,Register::A,Register::HL},
    {"ADD A, A",1,InstType::ADD,AddrMode::RegToReg,Register::A,Register::A},
    {"ADC A, B",1,InstType::ADC,AddrMode::RegToReg,Register::A,Register::B},
    {"ADC A, C",1,InstType::ADC,AddrMode::RegToReg,Register::A,Register::C},
    {"ADC A, D",1,InstType::ADC,AddrMode::RegToReg,Register::A,Register::D},
    {"ADC A, E",1,InstType::ADC,AddrMode::RegToReg,Register::A,Register::E},
    {"ADC A, H",1,InstType::ADC,AddrMode::RegToReg,Register::A,Register::H},
    {"ADC A, L",1,InstType::ADC,AddrMode::RegToReg,Register::A,Register::L},
    {"ADC A, [HL]",2,InstType::ADC,AddrMode::MemRegToReg,Register::A,Register::HL},
    {"ADC A, A",1,InstType::ADC,AddrMode::RegToReg,Register::A,Register::A},
    //$9x
    {"SUB A, B",1,InstType::SUB,AddrMode::RegToReg,Register::A,Register::B},
    {"SUB A, C",1,InstType::SUB,AddrMode::RegToReg,Register::A,Register::C},
    {"SUB A, D",1,InstType::SUB,AddrMode::RegToReg,Register::A,Register::D},
    {"SUB A, E",1,InstType::SUB,AddrMode::RegToReg,Register::A,Register::E},
    {"SUB A, H",1,InstType::SUB,AddrMode::RegToReg,Register::A,Register::H},
    {"SUB A, L",1,InstType::SUB,AddrMode::RegToReg,Register::A,Register::L},
    {"SUB A, [HL]",2,InstType::SUB,AddrMode::MemRegToReg,Register::A,Register::HL},
    {"SUB A, A",1,InstType::SUB,AddrMode::RegToReg,Register::A,Register::A},
    {"SBC A, B",1,InstType::SBC,AddrMode::RegToReg,Register::A,Register::B},
    {"SBC A, C",1,InstType::SBC,AddrMode::RegToReg,Register::A,Register::C},
    {"SBC A, D",1,InstType::SBC,AddrMode::RegToReg,Register::A,Register::D},
    {"SBC A, E",1,InstType::SBC,AddrMode::RegToReg,Register::A,Register::E},
    {"SBC A, H",1,InstType::SBC,AddrMode::RegToReg,Register::A,Register::H},
    {"SBC A, L",1,InstType::SBC,AddrMode::RegToReg,Register::A,Register::L},
    {"SBC A, [HL]",2,InstType::SBC,AddrMode::MemRegToReg,Register::A,Register::HL},
    {"SBC A, A",1,InstType::SBC,AddrMode::RegToReg,Register::A,Register::A},
    //$Ax
    {"AND A, B",1,InstType::AND,AddrMode::RegToReg,Register::A,Register::B},
    {"AND A, C",1,InstType::AND,AddrMode::RegToReg,Register::A,Register::C},
    {"AND A, D",1,InstType::AND,AddrMode::RegToReg,Register::A,Register::D},
    {"AND A, E",1,InstType::AND,AddrMode::RegToReg,Register::A,Register::E},
    {"AND A, H",1,InstType::AND,AddrMode::RegToReg,Register::A,Register::H},
    {"AND A, L",1,InstType::AND,AddrMode::RegToReg,Register::A,Register::L},
    {"AND A, [HL]",2,InstType::AND,AddrMode::MemRegToReg,Register::A,Register::HL},
    {"AND A, A",1,InstType::AND,AddrMode::RegToReg,Register::A,Register::A},
    {"XOR A, B",1,InstType::XOR,AddrMode::RegToReg,Register::A,Register::B},
    {"XOR A, C",1,InstType::XOR,AddrMode::RegToReg,Register::A,Register::C},
    {"XOR A, D",1,InstType::XOR,AddrMode::RegToReg,Register::A,Register::D},
    {"XOR A, E",1,InstType::XOR,AddrMode::RegToReg,Register::A,Register::E},
    {"XOR A, H",1,InstType::XOR,AddrMode::RegToReg,Register::A,Register::H},
    {"XOR A, L",1,InstType::XOR,AddrMode::RegToReg,Register::A,Register::L},
    {"XOR A, [HL]",2,InstType::XOR,AddrMode::MemRegToReg,Register::A,Register::HL},
    {"XOR A, A",1,InstType::XOR,AddrMode::RegToReg,Register::A,Register::A},
    //$Bx
    {"OR A, B",1,InstType::OR,AddrMode::RegToReg,Register::A,Register::B},
    {"OR A, C",1,InstType::OR,AddrMode::RegToReg,Register::A,Register::C},
    {"OR A, D",1,InstType::OR,AddrMode::RegToReg,Register::A,Register::D},
    {"OR A, E",1,InstType::OR,AddrMode::RegToReg,Register::A,Register::E},
    {"OR A, H",1,InstType::OR,AddrMode::RegToReg,Register::A,Register::H},
    {"OR A, L",1,InstType::OR,AddrMode::RegToReg,Register::A,Register::L},
    {"OR A, [HL]",2,InstType::OR,AddrMode::MemRegToReg,Register::A,Register::HL},
    {"OR A, A",1,InstType::OR,AddrMode::RegToReg,Register::A,Register::A},
    {"CP A, B",1,InstType::CP,AddrMode::RegToReg,Register::A,Register::B},
    {"CP A, C",1,InstType::CP,AddrMode::RegToReg,Register::A,Register::C},
    {"CP A, D",1,InstType::CP,AddrMode::RegToReg,Register::A,Register::D},
    {"CP A, E",1,InstType::CP,AddrMode::RegToReg,Register::A,Register::E},
    {"CP A, H",1,InstType::CP,AddrMode::RegToReg,Register::A,Register::H},
    {"CP A, L",1,InstType::CP,AddrMode::RegToReg,Register::A,Register::L},
    {"CP A, [HL]",2,InstType::CP,AddrMode::MemRegToReg,Register::A,Register::HL},
    {"CP A, A",1,InstType::CP,AddrMode::RegToReg,Register::A,Register::A},
    //$Cx
    {"RET NZ",2,InstType::RET,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::NZ},
    {"POP BC",3,InstType::POP,AddrMode::MemRegToReg,Register::BC,Register::SP},
    {"JP NZ, a16",3,InstType::JP,AddrMode::Bit16,Register::NONE,Register::NONE,JmpCondition::NZ},
    {"JP a16",4,InstType::JP,AddrMode::Bit16,Register::NONE,Register::NONE},
    {"CALL NZ, a16",3,InstType::CALL,AddrMode::Bit16,Register::NONE,Register::NONE,JmpCondition::NZ},
    {"PUSH BC",4,InstType::PUSH,AddrMode::Reg,Register::NONE,Register::BC},
    {"ADD A, n8",2,InstType::ADD,AddrMode::Bit8ToReg,Register::A},
    {"RST $00",4,InstType::RST},
    {"RET Z",2,InstType::RET,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::Z},
    {"RET",4,InstType::RET},
    {"JP Z, a16",3,InstType::JP,AddrMode::Bit16,Register::NONE,Register::NONE,JmpCondition::Z},
    {"PREFIX"},
    {"CALL Z, a16",3,InstType::CALL,AddrMode::Bit16,Register::NONE,Register::NONE,JmpCondition::Z},
    {"CALL Z, a16",6,InstType::CALL,AddrMode::Bit16,Register::NONE,Register::NONE,JmpCondition::Z},
    {"ADC A, n8",2,InstType::ADC,AddrMode::Bit8ToReg,Register::A},
    {"RST $08",4,InstType::RST,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::None, 0x08},
    //$Dx
    {"RET Z",2,InstType::RET,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::NC},
    {"POP DE",3,InstType::POP,AddrMode::MemRegToReg,Register::DE,Register::SP},
    {"JP NC, a16",3,InstType::JP,AddrMode::Bit16,Register::NONE,Register::NONE,JmpCondition::NC},
    {"-"},
    {"CALL NC, a16",3,InstType::CALL,AddrMode::Bit16,Register::NONE,Register::NONE,JmpCondition::NC},
    {"PUSH DE",4,InstType::PUSH,AddrMode::Reg,Register::NONE,Register::DE},
    {"SUB A, n8",2,InstType::SUB,AddrMode::Bit8ToReg,Register::A},
    {"RST $10",4,InstType::RST,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::None, 0x10},
    {"RET C",2,InstType::RET,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::C},
    {"RETI",4,InstType::RETI},
    {"JP C, a16",3,InstType::JP,AddrMode::Bit16,Register::NONE,Register::NONE,JmpCondition::C},
    {"-"},
    {"CALL C, a16",3,InstType::CALL,AddrMode::Bit16,Register::NONE,Register::NONE,JmpCondition::C},
    {"-"},
    {"SBC A, n8",2,InstType::SBC,AddrMode::Bit8ToReg,Register::A},
    {"RST $18",4,InstType::RST,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::None, 0x18},
    //$Ex
    {"LDH [a8], A",3,InstType::LDH,AddrMode::RegToMemBit8,Register::NONE,Register::A},
    {"POP HL",3,InstType::POP,AddrMode::MemRegToReg,Register::HL,Register::SP},
    {"LD [C], A",2,InstType::LD,AddrMode::RegToMemReg,Register::C,Register::A},
    {"-"},
    {"-"},
    {"PUSH HL",4,InstType::PUSH,AddrMode::Reg,Register::NONE,Register::HL},
    {"AND A, n8",2,InstType::AND,AddrMode::Bit8ToReg,Register::A},
    {"RST $20",4,InstType::RST,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::None, 0x20},
    {"ADD SP, e8",4,InstType::ADD,AddrMode::Bit8ToReg,Register::SP},
    {"JP HL",1,InstType::JP,AddrMode::Reg,Register::NONE,Register::HL},
    {"LD [a16], A",4,InstType::LD,AddrMode::RegToMemBit16,Register::NONE,Register::A},
    {"-"},
    {"-"},
    {"-"},
    {"XOR A, n8",2,InstType::XOR,AddrMode::Bit8ToReg,Register::A},
    {"RST $28",4,InstType::RST,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::None, 0x28},
    //$Fx
    {"LDH A, [a8]",3,InstType::LDH,AddrMode::MemBit8ToReg,Register::A},
    {"POP AF",3,InstType::POP,AddrMode::MemRegToReg,Register::AF,Register::SP},
    {"LD A, [C]",2,InstType::LD,AddrMode::MemRegToReg,Register::C,Register::A},
    {"DI",1,InstType::DI},
    {"-"},
    {"PUSH AF",4,InstType::PUSH,AddrMode::Reg,Register::NONE,Register::AF},
    {"OR A, n8",2,InstType::OR,AddrMode::Bit8ToReg,Register::A},
    {"RST $30",4,InstType::RST,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::None, 0x30},
    {"LD HL, SP + e8",3,InstType::LDHLSP,AddrMode::Bit8,Register::HL,Register::SP},
    {"LD SP, HL",2,InstType::LD,AddrMode::RegToReg,Register::SP,Register::HL},
    {"LD A, [a16]",4,InstType::LD,AddrMode::MemBit16ToReg,Register::A},
    {"EI",1,InstType::EI},
    {"-"},
    {"-"},
    {"CP A, n8",2,InstType::CP,AddrMode::Bit8ToReg,Register::A},
    {"RST $38",4,InstType::RST,AddrMode::Impl,Register::NONE,Register::NONE,JmpCondition::None, 0x38},
  };
}
