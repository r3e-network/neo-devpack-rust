#include "llvm/TargetParser/Triple.h"
#include "llvm/MC/TargetRegistry.h"
#include "llvm/Target/NeoVM/NeoVMTargetInfo.h"
#include <iostream>

extern "C" void LLVMInitializeNeoVMTargetInfo();

int main() {
  std::cout << "Testing NeoVM target registration...\n";
  
  // Initialize target info
  LLVMInitializeNeoVMTargetInfo();
  std::cout << "Target info initialized\n";
  
  // Look up target
  llvm::Triple TT("neovm");
  std::string Error;
  const llvm::Target *T = llvm::TargetRegistry::lookupTarget("", TT, Error);
  if (!T) {
    std::cerr << "Target lookup failed: " << Error << "\n";
    return 1;
  }
  
  std::cout << "Target found: " << T->getName() << "\n";
  std::cout << "Target description: " << T->getShortDescription() << "\n";
  
  return 0;
}
