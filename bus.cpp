#include "bus.h"
#include "cpu.h"
namespace cash::GB
{
  Bus::Bus(Cart *cart) : cart(cart)
  {
    cpu = new CPU(*this);
  }

  void Bus::write(const bit16 &addr, const bit8 value)
  {
  }

  void Bus::attachCpu(CPU &cpu)
  {
    this->cpu = &cpu;
  }

  bit8 Bus::read(const bit16 &addr) const
  {
    // ROM bank 00
    if (addr <= 0x3FFF)
      return cart->read(addr);
    // ROM bank 01~NN
    if (addr <= 0x7FFF)
      return cart->read(addr);
    // TODO: Video RAM
    if (addr <= 0x9FFF)
      return 0x00;
    // External RAM
    if (addr <= 0xBFFF)
      return cart->read(addr);
    // Work RAM
    if (addr <= 0xCFFF)
      return cpu->read(addr);
    // Work RAM
    if (addr <= 0xDFFF)
      return cpu->read(addr);
    // area prohibited
    // TODO: mirror $C000~$DDFF
    if (addr <= 0xFDFF)
      return 0x00;
    // OAM
    if (addr <= 0xFE9F)
      return cpu->read(addr);
    // Not Usable
    if (addr <= 0xFEFF)
      return 0x00;
    // I/O Registers
    // TODO: I/O
    if (addr <= 0xFF7F)
      return 0x00;
    // high ram
    if (addr <= 0xFFFE)
      return cpu->read(addr);
    // IE register
    if (addr <= 0xFFFF)
      return cpu->read(addr);
  }
} // namespace cash::GB
