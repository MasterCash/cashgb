#include "cart.h"
#include "cpu.h"


int main(int argc, char **argv)
{
  using namespace cash::GB;
  std::cout << "Cash GB booting..." << std::endl;
  if (argc < 2) {
    std::cout << "missing args: found " << argc << " but expected 2" << std::endl;
    return -1;
  }

  cash::GB::Cart cart = Cart(std::string(argv[1]));
  std::cout << cart << std::endl;
  cash::GB::
  return 0;
}