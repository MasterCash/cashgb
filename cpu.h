#pragma once
#include <cstdint>
#include <string>
#include "bus.h"
using bit8 = std::uint8_t;
using bit16 = std::uint16_t;
using std::string;

namespace cash::GB
{
  enum struct Register
  {
    NONE,
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    SP,
    PC,
    HL,
    AF,
    BC,
    DE,
  };

  enum struct InstType
  {
    Invalid,
    NOP,
    LD,
    LDH,
    LDHLSP,
    PUSH,
    POP,
    ADD,
    ADC,
    SUB,
    SBC,
    CP,
    INC,
    DEC,
    AND,
    OR,
    XOR,
    CCF,
    DAA,
    CPL,
    JP,
    JR,
    CALL,
    RET,
    RETI,
    RST,
    HALT,
    STOP,
    DI,
    EI,
    RLC,
    RRC,
    RL,
    RR,
    SCF,
  };

  enum struct AddrMode
  {
    Impl,
    Reg,
    RegToReg,
    RegToMemReg,
    RegToMemRegDec,
    RegToMemRegInc,
    RegToMemBit8,
    RegToMemBit16,
    RegToBit16,
    MemBit8ToReg,
    MemBit16ToReg,
    MemReg,
    MemRegToReg,
    MemRegToMemReg,
    MemRegDecToReg,
    MemRegIncToReg,
    Bit8,
    Bit8ToReg,
    Bit16,
    Bit16ToReg,
    Bit8ToMemReg,
  };

  enum struct JmpCondition
  {
    None,
    Z,
    NZ,
    C,
    NC,
  };

  struct Instruction
  {
    string name;
    bit8 cycles = 0;
    InstType type = InstType::Invalid;
    AddrMode addrMode = AddrMode::Impl;
    Register destReg = Register::NONE;
    Register startReg = Register::NONE;
    JmpCondition jmp = JmpCondition::None;
    bit8 rstAddr = 0x00;
  };

  struct Flags
  {
    bit8 : 4;
    bit8 C : 1;
    bit8 H : 1;
    bit8 N : 1;
    bit8 Z : 1;
  };

  class CPU
  {
  public:
    CPU(Bus &bus) : bus(bus), F(*(bit8 *)&flags)
    {
      bus.attachCpu(*this);
    }

  public:
    bit8 A = 0x00;
    bit8 B = 0x00;
    bit8 C = 0x00;
    bit8 D = 0x00;
    bit8 E = 0x00;
    bit8 H = 0x00;
    bit8 L = 0x00;
    bit16 SP = 0x00;
    bit16 PC = 0x00;
    bit8 &F;

    void reset();
    void clock();
    bit8 read(const bit16 &addr) const;

  private:
    static const Instruction instructions[256];
    static const Instruction prefixedInst[256];

  private:
    bool isMemAddr = false;
    bit16 memLoc = 0x0000;
    bit16 value = 0x0000;
    bit8 cycle = 0x00;
    const Instruction *currentInstruction;
    bool IME = false;
    bool stopSystem = false;
    bool stopMain = false;
    bool bootOff = false;
    Flags flags = {0, 0, 0, 0};
    Bus &bus;

  private:
    bit8 fetch(const Instruction &inst);
    bit16 regRead(const Register &reg) const;
    void regWrite(const Register &reg, const bit16 &value);
    bool isBit16(const Register &reg) const;
    void process(const Instruction &inst);
  private:
    void LD(const Instruction &inst);
    void LDH(const Instruction &inst);
    void LDHLSP(const Instruction & inst);
    void PUSH(const Instruction &inst);
    void POP(const Instruction &inst);
    void ADD(const Instruction &inst);
    void ADC(const Instruction &inst);
    void SUB(const Instruction &inst);
    void SBC(const Instruction &inst);
    void CP(const Instruction &inst);
    void INC(const Instruction &inst);
    void DEC(const Instruction &inst);
    void AND(const Instruction &inst);
    void OR(const Instruction &inst);
    void XOR(const Instruction &inst);
    void CCF(const Instruction &inst);
    void DAA(const Instruction &inst);
    void CPL(const Instruction &inst);
    void JP(const Instruction &inst);
    void JR(const Instruction &inst);
    void CALL(const Instruction &inst);
    void RET(const Instruction &inst);
    void RETI(const Instruction &inst);
    void RST(const Instruction &inst);
    void HALT(const Instruction &inst);
    void STOP(const Instruction &inst);
    void DI(const Instruction &inst);
    void EI(const Instruction &inst);
    void NOP(const Instruction &inst);
    void RLC(const Instruction &inst);
    void RRC(const Instruction &inst);
    void RR(const Instruction &inst);
    void RL(const Instruction &inst);
    void SCF(const Instruction &inst);
  };

}