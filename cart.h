#pragma once
#include <stdint.h>
#include <inttypes.h>
#include <string>
#include <array>
#include <iostream>

using bit8 = std::uint8_t;
using bit16 = std::uint16_t;
using bit32 = std::uint32_t;
using std::string;

namespace cash::GB
{

  enum struct MapperType
  {
    NONE,
    MBC1,
    MBC2,
    MMM01,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
    POCKET_CAMERA,
    BANDAI_TAMA5,
    HuC3,
    HuC1,
  };

  enum struct BootFailure
  {
    NONE,
    HEADER_CHECK_SUM,
    GLOBAL_CHECK_SUM,
    LOGO,
  };

  struct CartType
  {
    MapperType mapper = MapperType::NONE;
    bool ram = false;
    bool battery = false;
    bool timer = false;
    bool rumble = false;
    bool sensor = false;
  };

  class Cart
  {
  public:
    Cart(string fileName);
    ~Cart();

  public:
    friend std::ostream &operator<<(std::ostream &out, const Cart &cart);
    bit8 read(const bit16 &addr) const;

  private:
    BootFailure invalid = BootFailure::NONE;
    bit8 *rom;
    bit8 *ram;
    bit8 bank = 0x00;
    string title = string(16, ' ');
    bit32 size = 0;
    bool supportsCGB = 0;
    string licensee;
    bool supportSGB = 0;
    CartType cartType;
    bit32 romSize = 0;
    bit16 romBanks = 2;
    bit16 curRomBank = 1;
    bit32 ramSize = 0;
    bit16 ramBanks = 0;
    bit16 curRamBank = 1;
    bool destination = true;
    bit8 version = 0;

  private:
    static std::array<string, 256> oldLicensee;
    static string getLicensee(const bit8 &code, const char &one, const char &two);
    bool hasLogo();
    static CartType getCartType(const bit8 &type);
  };

} // namespace cash::GB
