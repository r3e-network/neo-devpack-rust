#include "llvm/Target/NeoVM/MCTargetDesc/NeoVMMCTargetDesc.h"

#include "llvm/MC/MCAsmInfo.h"
#include "llvm/MC/MCInstrInfo.h"
#include "llvm/MC/MCRegisterInfo.h"
#include "llvm/MC/MCSubtargetInfo.h"
#include "llvm/MC/MCTargetOptions.h"
#include "llvm/MC/TargetRegistry.h"
#include "llvm/Target/NeoVM/MCTargetDesc/NeoVMAsmInfo.h"
#include "llvm/Target/NeoVM/NeoVMTargetInfo.h"
#include "llvm/TargetParser/Triple.h"
#include "llvm/CodeGen/Passes.h"

#define GET_REGINFO_ENUM
#define GET_REGINFO_MC_DESC
#include "NeoVMGenRegisterInfo.inc"
#undef GET_REGINFO_ENUM
#undef GET_REGINFO_MC_DESC

#define GET_INSTRINFO_MC_DESC
#include "NeoVMGenInstrInfo.inc"
#undef GET_INSTRINFO_MC_DESC

#define GET_SUBTARGETINFO_MC_DESC
#include "NeoVMGenSubtargetInfo.inc"
#undef GET_SUBTARGETINFO_MC_DESC

using namespace llvm;

MCAsmInfo *llvm::createNeoVMMCAsmInfo(const MCRegisterInfo &MRI, const Triple &TT,
                                      const MCTargetOptions &) {
  return new NeoVMMCAsmInfo(TT);
}

MCInstrInfo *llvm::createNeoVMMCInstrInfo() {
  auto *X = new MCInstrInfo();
  InitNeoVMMCInstrInfo(X);
  return X;
}

MCRegisterInfo *llvm::createNeoVMMCRegisterInfo(const Triple &) {
  auto *X = new MCRegisterInfo();
  InitNeoVMMCRegisterInfo(X, /*RA=*/NeoVM::STACK);
  return X;
}

MCSubtargetInfo *llvm::createNeoVMMCSubtargetInfo(const Triple &TT, StringRef CPU,
                                                  StringRef FS) {
  return createNeoVMMCSubtargetInfoImpl(TT, CPU, /*TuneCPU=*/CPU, FS);
}

extern "C" LLVM_EXTERNAL_VISIBILITY void LLVMInitializeNeoVMTargetMC() {
  Target &T = getTheNeoVMTarget();
  TargetRegistry::RegisterMCAsmInfo(T, createNeoVMMCAsmInfo);
  TargetRegistry::RegisterMCInstrInfo(T, createNeoVMMCInstrInfo);
  TargetRegistry::RegisterMCRegInfo(T, createNeoVMMCRegisterInfo);
  TargetRegistry::RegisterMCSubtargetInfo(T, createNeoVMMCSubtargetInfo);
}
