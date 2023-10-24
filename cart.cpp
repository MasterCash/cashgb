#include "cart.h"
#include <iomanip>
#include <fstream>

namespace cash::GB
{
  Cart::~Cart()
  {
    delete[] rom;
  }
  Cart::Cart(string fileName)
  {
    std::fstream file(fileName);
    size = 0;
    while (!file.eof())
    {
      size++;
      file.get();
    }
    size--;
    rom = new bit8[size];
    file.close();
    file.open(fileName);
    int pos = 0;
    while (!file.eof() && pos < size)
    {
      rom[pos] = file.get();
      pos++;
    }
    file.close();
    if (!hasLogo())
      invalid = BootFailure::LOGO;

    bit8 *t = &rom[0x0134];
    for (bit16 i = 0; i <= 15; i++)
    {
      if (t[i] == 0x00)
        break;
      title[i] = t[i];
    }
    supportsCGB = (rom[0x0143] & 0x80) > 0;
    licensee = getLicensee(rom[0x014B], rom[0x0144], rom[0x0144]);
    cartType = getCartType(rom[0x0147]);
    bit8 romSizeByte = rom[0x0148];
    romSize = (1 << romSizeByte) * 0x8000;
    romBanks = (1 << romSize) * 2;
    bit8 ramSizeByte = rom[0x0149];

    if (this->cartType.ram)
    {
      switch (ramSizeByte)
      {
      case 0x02:
        ramSize = 0x2000;
        ramBanks = 1;
        break;
      case 0x03:
        ramSize = 0x8000;
        ramBanks = 4;
        break;
      case 0x04:
        ramSize = 0x20000;
        ramBanks = 16;
        break;
      case 0x05:
        ramSize = 0x10000;
        ramBanks = 8;
        break;
      }
    }
    ram = new bit8[ramSize];
    destination = rom[0x014A];
    version = rom[0x014C];
    bit8 headerChecksum = 0;
    for (bit16 addr = 0x0134; addr <= 0x014C; addr++)
      headerChecksum = headerChecksum - rom[addr] - 1;

    if (headerChecksum != rom[0x014D])
      invalid = BootFailure::HEADER_CHECK_SUM;

    bit16 globalChecksum = 0;
    for (bit32 addr = 0; addr < size; addr++)
    {
      if (addr == 0x014E || addr == 0x014F)
        continue;
      globalChecksum += rom[addr];
    }
    if (globalChecksum != ((rom[0x014E] << 8) | rom[0x014F]))
      invalid = BootFailure::GLOBAL_CHECK_SUM;
  }

  std::ostream &operator<<(std::ostream &out, const Cart &cart)
  {
    using std::endl;
    out << "===========Cart===========" << endl;
    out << "failed: \t" << (cart.invalid == BootFailure::NONE ? "No" : std::to_string((int)cart.invalid)) << endl;
    out << "title: \t" << cart.title << endl;
    out << "CGB: \t" << (cart.supportsCGB ? "Yes" : "No") << endl;
    out << "licensee: \t" << (cart.licensee) << endl;
    out << "cartType: " << endl;
    out << "\t mapper: \t" << (int)cart.cartType.mapper << endl;
    out << "\t ram:    \t" << (cart.cartType.ram ? "Yes" : "No") << endl;
    out << "\t battery: \t" << (cart.cartType.battery ? "Yes" : "No") << endl;
    out << "\t timer: \t" << (cart.cartType.timer ? "Yes" : "No") << endl;
    out << "\t rumble: \t" << (cart.cartType.rumble ? "Yes" : "No") << endl;
    out << "\t sensor: \t" << (cart.cartType.sensor ? "Yes" : "No") << endl;
    out << "romSize: \t" << cart.romSize << endl;
    out << "romBank: \t" << cart.romBanks << endl;
    out << "ramSize: \t" << cart.ramSize << endl;
    out << "ramBank: \t" << cart.ramBanks << endl;
    out << "destination: \t" << (cart.destination ? "Japan" : "Worldwide") << endl;
    out << "version: \t" << +cart.version << endl;
    out << "global checksum:" << std::hex << ((cart.rom[0x014E] << 8) | cart.rom[0x014F]) << endl;

    return out;
  }

  bit8 Cart::read(const bit16 &addr) const
  {
    // ROM Bank 00
    if (addr <= 0x3FFF)
      return rom[addr];
    if (addr <= 0x7FFF)
    {
      bit16 modifiedAddr = (addr + ((curRomBank - 1) * 0x4000));
      if (modifiedAddr >= romSize)
      {
        std::cout << "Error invalid Cart ROM Access: " << std::hex << addr << ", " << modifiedAddr << std::endl;
        return 0x00;
      }
      return rom[modifiedAddr];
    }
    if (addr <= 0x9FFF)
      return 0x00;
    if (addr <= 0xBFFF)
    {
      bit16 modifiedAddr = (addr + ((curRamBank - 1) * 0x2000));
      if (modifiedAddr >= ramSize)
      {
        std::cout << "Error invalid Cart RAM Access" << modifiedAddr << std::endl;
        return 0x00;
      }
      return ram[modifiedAddr];
    }

    return 0x00;
  }

  bool Cart::hasLogo()
  {
    bit16 loc = 0x0104;
    return rom[loc++] == 0xCE &&
           rom[loc++] == 0xED &&
           rom[loc++] == 0x66 &&
           rom[loc++] == 0x66 &&
           rom[loc++] == 0xCC &&
           rom[loc++] == 0x0D &&
           rom[loc++] == 0x00 &&
           rom[loc++] == 0x0B &&
           rom[loc++] == 0x03 &&
           rom[loc++] == 0x73 &&
           rom[loc++] == 0x00 &&
           rom[loc++] == 0x83 &&
           rom[loc++] == 0x00 &&
           rom[loc++] == 0x0C &&
           rom[loc++] == 0x00 &&
           rom[loc++] == 0x0D &&
           rom[loc++] == 0x00 &&
           rom[loc++] == 0x08 &&
           rom[loc++] == 0x11 &&
           rom[loc++] == 0x1F &&
           rom[loc++] == 0x88 &&
           rom[loc++] == 0x89 &&
           rom[loc++] == 0x00 &&
           rom[loc++] == 0x0E &&
           rom[loc++] == 0xDC &&
           rom[loc++] == 0xCC &&
           rom[loc++] == 0x6E &&
           rom[loc++] == 0xE6 &&
           rom[loc++] == 0xDD &&
           rom[loc++] == 0xDD &&
           rom[loc++] == 0xD9 &&
           rom[loc++] == 0x99 &&
           rom[loc++] == 0xBB &&
           rom[loc++] == 0xBB &&
           rom[loc++] == 0x67 &&
           rom[loc++] == 0x63 &&
           rom[loc++] == 0x6E &&
           rom[loc++] == 0x0E &&
           rom[loc++] == 0xEC &&
           rom[loc++] == 0xCC &&
           rom[loc++] == 0xDD &&
           rom[loc++] == 0xDC &&
           rom[loc++] == 0x99 &&
           rom[loc++] == 0x9F &&
           rom[loc++] == 0xBB &&
           rom[loc++] == 0xB9 &&
           rom[loc++] == 0x33 &&
           rom[loc++] == 0x3E;
  }

  string Cart::getLicensee(const bit8 &code, const char &one, const char &two)
  {
    if (code != 0x33)
      return oldLicensee[code];

    switch (one)
    {
    case '0':
      switch (two)
      {
      case '0':
        return "None";
      case '1':
        return "Nintendo R&D1";
      case '8':
        return "Capcom";
      };
      break;
    case '1':
      switch (two)
      {
      case '3':
        return "Electronic Arts";
      case '8':
        return "Hudson Soft";
      case '9':
        return "b-ai";
      }
      break;
    case '2':
      switch (two)
      {
      case '0':
        return "kss";
      case '2':
        return "pow";
      case '4':
        return "PCM Complete";
      case '5':
        return "san-x";
      case '8':
        return "Kemco Japan";
      case '9':
        return "seta";
      }
      break;
    case '3':
      switch (two)
      {
      case '0':
        return "Viacom";
      case '1':
        return "Nintendo";
      case '2':
        return "Bandai";
      case '3':
        return "Ocean/Acclaim";
      case '4':
        return "Konami";
      case '5':
        return "Hector";
      case '7':
        return "Taito";
      case '8':
        return "Hudson";
      case '9':
        return "Banpresto";
      }
      break;
    case '4':
      switch (two)
      {
      case '1':
        return "Ubi Soft";
      case '2':
        return "Atlus";
      case '4':
        return "Malibu";
      case '6':
        return "angel";
      case '7':
        return "Bullet-Proof";
      case '9':
        return "irem";
      }
      break;
    case '5':
      switch (two)
      {
      case '0':
        return "Absolute";
      case '1':
        return "Acclaim";
      case '2':
        return "Activision";
      case '3':
        return "American sammy";
      case '4':
        return "Konami";
      case '5':
        return "Hi tech entertainment";
      case '6':
        return "LJN";
      case '7':
        return "Matchbox";
      case '8':
        return "Mattel";
      case '9':
        return "Milton Bradley";
      }
      break;
    case '6':
      switch (two)
      {
      case '0':
        return "Titus";
      case '1':
        return "Virgin";
      case '4':
        return "LucasArts";
      case '7':
        return "Ocean";
      case '9':
        return "Electronic Arts";
      }
      break;
    case '7':
      switch (two)
      {
      case '0':
        return "Infogrames";
      case '1':
        return "Interplay";
      case '2':
        return "Broderbund";
      case '3':
        return "sculptured";
      case '5':
        return "sci";
      case '8':
        return "THQ";
      case '9':
        return "Accolade";
      }
      break;
    case '8':
      switch (two)
      {
      case '0':
        return "misawa";
      case '3':
        return "lozc";
      case '6':
        return "Tokuma Shoten Intermedia";
      case '7':
        return "Tsukuda Original";
      }
      break;
    case '9':
      switch (two)
      {
      case '1':
        return "Chunsoft";
      case '2':
        return "Video system";
      case '3':
        return "Ocean/Acclaim";
      case '5':
        return "Varie";
      case '6':
        return "Yonezawa/s'pal";
      case '7':
        return "Kaneko";
      case '9':
        return "Pack in soft";
      case 'H':
        return "Bottom Up";
      }
      break;
    case 'A':
      if (two == '4')
        return "Konami (Yu-Gi-Oh!)";
      break;
    }

    return "";
  }

  CartType Cart::getCartType(const bit8 &type)
  {
    CartType cartType;
    switch (type)
    {
    case 0x03:
      cartType.battery = true;
    case 0x02:
      cartType.ram = true;
    case 0x01:
      cartType.mapper = MapperType::MBC1;
      break;
    case 0x06:
      cartType.battery = true;
    case 0x05:
      cartType.mapper = MapperType::MBC2;
      break;
    case 0x09:
      cartType.battery = true;
    case 0x08:
      cartType.ram = true;
      break;
    case 0x0D:
      cartType.battery = true;
    case 0x0C:
      cartType.ram = true;
    case 0x0B:
      cartType.mapper = MapperType::MMM01;
      break;
    case 0x10:
      cartType.ram = true;
    case 0x0F:
      cartType.timer = true;
      cartType.battery = true;
      cartType.mapper = MapperType::MBC3;
      break;
    case 0x13:
      cartType.battery = true;
    case 0x12:
      cartType.ram = true;
    case 0x11:
      cartType.mapper = MapperType::MBC3;
      break;
    case 0x1B:
      cartType.battery = true;
    case 0x1A:
      cartType.ram = true;
    case 0x19:
      cartType.mapper = MapperType::MBC5;
      break;
    case 0x1E:
      cartType.battery = true;
    case 0x1D:
      cartType.ram = true;
    case 0x1C:
      cartType.rumble = true;
      cartType.mapper = MapperType::MBC5;
      break;
    case 0x20:
      cartType.mapper = MapperType::MBC6;
      break;
    case 0x22:
      cartType.mapper = MapperType::MBC7;
      cartType.sensor = true;
      cartType.rumble = true;
      cartType.ram = true;
      cartType.battery = true;
      break;
    case 0xFC:
      cartType.mapper = MapperType::POCKET_CAMERA;
      break;
    case 0xFD:
      cartType.mapper = MapperType::BANDAI_TAMA5;
      break;
    case 0xFE:
      cartType.mapper = MapperType::HuC3;
      break;
    case 0xFF:
      cartType.mapper = MapperType::HuC1;
      cartType.ram = true;
      cartType.battery = true;
      break;
    }

    return cartType;
  };

  std::array<string, 256> Cart::oldLicensee = std::array<string, 256>{
      "None",
      "Nintendo",
      "Capcom",
      "Hot-B",
      "Jaleco",
      "Coconuts Japan",
      "Elite Systems",
      "EA (Electronic Arts)",
      "Hudsonsoft",
      "ITC Entertainment",
      "Yanoman",
      "Japan Clary",
      "Virgin Interactive",
      "PCM Complete",
      "San-X",
      "Kotobuki Systems",
      "Seta",
      "Infogrames",
      "Nintendo",
      "Bandai",
      "",
      "Konami",
      "HectorSoft",
      "Capcom",
      "Banpresto",
      ".Entertainment i",
      "Gremlin",
      "Ubisoft",
      "Atlus",
      "Malibu",
      "Angel",
      "Spectrum Holoby",
      "Irem",
      "Virgin Interactive",
      "Malibu",
      "U.S. Gold",
      "Absolute",
      "Acclaim",
      "Activision",
      "American Sammy",
      "GameTek",
      "Park Place",
      "LJN",
      "Matchbox",
      "Milton Bradley",
      "Mindscape",
      "Romstar",
      "Naxat Soft",
      "Tradewest",
      "Titus",
      "Virgin Interactive",
      "Ocean Interactive",
      "EA (Electronic Arts)",
      "Elite Systems",
      "Electro Brain",
      "Infogrames",
      "Interplay",
      "Broderbund",
      "Sculptered Soft",
      "The Sales Curve",
      "t.hq",
      "Accolade",
      "Triffix Entertainment",
      "Microprose",
      "Kemco",
      "Misawa Entertainment",
      "Lozc",
      "Tokuma Shoten Intermedia",
      "Bullet-Proof Software",
      "Vic Tokai",
      "Ape",
      "I'Max",
      "Chunsoft Co.",
      "Video System",
      "Tsubaraya Productions Co.",
      "Varie Corporation",
      "Yonezawa/S'Pal",
      "Kaneko",
      "Arc",
      "Nihon Bussan",
      "Tecmo",
      "Imagineer",
      "Banpresto",
      "Nova",
      "Hori Electric",
      "Bandai",
      "Konami",
      "Kawada",
      "Takara",
      "Technos Japan",
      "Broderbund",
      "Toei Animation",
      "Toho",
      "Namco",
      "acclaim",
      "ASCII or Nexsoft",
      "Bandai",
      "Square Enix",
      "HAL Laboratory",
      "SNK",
      "Pony Canyon",
      "Culture Brain",
      "Sunsoft",
      "Sony Imagesoft",
      "Sammy",
      "Taito",
      "Kemco",
      "Squaresoft",
      "Tokuma Shoten Intermedia",
      "Data East",
      "Tonkinhouse",
      "Koei",
      "UFL",
      "Ultra",
      "Vap",
      "Use Corporation",
      "Meldac",
      ".Pony Canyon or",
      "Angel",
      "Taito",
      "Sofel",
      "Quest",
      "Sigma Enterprises",
      "ASK Kodansha Co.",
      "Naxat Soft",
      "Copya System",
      "Banpresto",
      "Tomy",
      "LJN",
      "NCS",
      "Human",
      "Altron",
      "Jaleco",
      "Towa Chiki",
      "Yutaka",
      "Varie",
      "Epcoh",
      "Athena",
      "Asmik ACE Entertainment",
      "Natsume",
      "King Records",
      "Atlus",
      "Epic/Sony Records",
      "IGS",
      "A Wave",
      "Extreme Entertainment",
      "LJN"};

} // namespace cash::GB