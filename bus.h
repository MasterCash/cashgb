#pragma once
#include <cstdint>
#include "cart.h"

using bit8 = std::uint8_t;
using bit16 = std::uint16_t;

namespace cash::GB
{
  class CPU;
  class Bus
  {
  public:
    Bus(Cart *cart);
    bit8 read(const bit16 &addr) const;
    void write(const bit16 &addr, const bit8 value);
    void attachCpu(CPU& cpu);

  private:
    Cart *cart;
    CPU *cpu;
  };
} // namespace cash::GB
