#include "llvm/ADT/Triple.h"
#include "llvm/Support/TargetRegistry.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/Target/NeoVM/NeoVMTargetInfo.h"

using namespace llvm;

// NeoVM target specification for Clang
extern "C" void LLVMInitializeNeoVMTargetInfo();
extern "C" void LLVMInitializeNeoVMTarget();
extern "C" void LLVMInitializeNeoVMTargetMC();
extern "C" void LLVMInitializeNeoVMAsmPrinter();

// Initialize NeoVM target for Clang
void initializeNeoVMClangTarget() {
  LLVMInitializeNeoVMTargetInfo();
  LLVMInitializeNeoVMTarget();
  LLVMInitializeNeoVMTargetMC();
  LLVMInitializeNeoVMAsmPrinter();
}

// NeoVM target triple validation
bool isValidNeoVMTriple(const Triple &TT) {
  return TT.getArch() == Triple::neovm;
}

// NeoVM data layout
const char* getNeoVMDataLayout() {
  return "e-m:e-p:32:32-i1:8-i8:8-i16:16-i32:32-i64:64-f32:32-f64:64-v16:16-v24:32-v32:32-v48:64-v96:128-v128:128-v256:256-v512:512-v1024:1024";
}

// NeoVM target features
void getNeoVMTargetFeatures(const Triple &TT, std::vector<std::string> &Features) {
  // NeoVM is a stack-based VM, so we don't need many features
  Features.push_back("+neovm");
  
  // Add basic integer support
  Features.push_back("+i32");
  Features.push_back("+i64");
  
  // Add boolean support
  Features.push_back("+i1");
  
  // Add byte string support
  Features.push_back("+i8");
}

// NeoVM target options
void getNeoVMTargetOptions(const Triple &TT, std::vector<std::string> &Options) {
  // NeoVM doesn't support JIT
  Options.push_back("-jit");
  
  // NeoVM doesn't support object files
  Options.push_back("-object");
  
  // NeoVM supports assembly output
  Options.push_back("+assembly");
  
  // NeoVM supports NEF output
  Options.push_back("+nef");
}
