#include "llvm/Target/NeoVM/NeoVMTargetInfo.h"

#include "llvm/MC/TargetRegistry.h"
#include "llvm/CodeGen/Passes.h"

using namespace llvm;

Target &llvm::getTheNeoVMTarget() {
  static Target TheNeoVMTarget;
  return TheNeoVMTarget;
}

extern "C" LLVM_EXTERNAL_VISIBILITY void LLVMInitializeNeoVMTargetInfo() {
  RegisterTarget<Triple::UnknownArch, /*HasJIT=*/false> X(getTheNeoVMTarget(),
                                                         "neovm", "Neo N3 VM",
                                                         "NeoVM");
}
