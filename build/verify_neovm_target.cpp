#include "llvm/MC/TargetRegistry.h"
#include "llvm/Target/NeoVM/NeoVMTargetInfo.h"
#include <iostream>

extern "C" void LLVMInitializeNeoVMTargetInfo();
extern "C" void LLVMInitializeNeoVMTarget();
extern "C" void LLVMInitializeNeoVMTargetMC();

int main() {
  LLVMInitializeNeoVMTargetInfo();
  LLVMInitializeNeoVMTarget();
  LLVMInitializeNeoVMTargetMC();

  std::string Error;
  llvm::Triple TT("neovm");
  const llvm::Target *T = llvm::TargetRegistry::lookupTarget("", TT, Error);
  if (!T) {
    std::cerr << "lookup failed: " << Error << "\n";
    return 1;
  }
  std::cout << T->getName() << "\n";
  return 0;
}
