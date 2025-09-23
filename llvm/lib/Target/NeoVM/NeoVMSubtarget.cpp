#include "llvm/Target/NeoVM/NeoVMSubtarget.h"

#include "llvm/Support/Debug.h"
#include "llvm/Target/NeoVM/NeoVMFrameLowering.h"
#include "llvm/Target/NeoVM/NeoVMInstrInfo.h"
#include "llvm/Target/NeoVM/NeoVMISelLowering.h"
#include "llvm/Target/NeoVM/NeoVMRegisterInfo.h"
#include "llvm/Target/NeoVM/NeoVMTargetMachine.h"
#include "llvm/Target/NeoVM/NeoVMTargetObjectFile.h"
#include "llvm/Target/NeoVM/NeoVMSyscalls.h"
#include "llvm/CodeGen/Passes.h"

#define DEBUG_TYPE "neovm-subtarget"

#define GET_SUBTARGETINFO_TARGET_DESC
#include "NeoVMGenSubtargetInfo.inc"
#undef GET_SUBTARGETINFO_TARGET_DESC

#define GET_SUBTARGETINFO_CTOR
#include "NeoVMGenSubtargetInfo.inc"
#undef GET_SUBTARGETINFO_CTOR

// Stub consistent with docs/neo-n3-backend.md (NeoVMSubtarget Responsibilities).

using namespace llvm;

NeoVMSubtarget::NeoVMSubtarget(const Triple &TT, StringRef CPU, StringRef FS,
                               const NeoVMTargetMachine &TM)
    : NeoVMGenSubtargetInfo(TT, CPU, CPU, FS), TargetTriple(TT) {
  initializeSubtargetDependencies(CPU, FS);
  TLInfo = std::make_unique<NeoVMTargetLowering>(TM, *this);
}

NeoVMSubtarget::~NeoVMSubtarget() = default;

const NeoVMInstrInfo &NeoVMSubtarget::getNeoVMInstrInfo() const {
  return *InstrInfo;
}

const NeoVMFrameLowering &NeoVMSubtarget::getNeoVMFrameLowering() const {
  return *FrameLowering;
}

const NeoVMRegisterInfo &NeoVMSubtarget::getNeoVMRegisterInfo() const {
  return *RegInfo;
}

const TargetInstrInfo *NeoVMSubtarget::getInstrInfo() const {
  return InstrInfo.get();
}

const TargetFrameLowering *NeoVMSubtarget::getFrameLowering() const {
  return FrameLowering.get();
}

const TargetRegisterInfo *NeoVMSubtarget::getRegisterInfo() const {
  return RegInfo.get();
}

const TargetLowering *NeoVMSubtarget::getTargetLowering() const {
  return TLInfo.get();
}

const NeoVMTargetLowering &NeoVMSubtarget::getNeoVMTargetLowering() const {
  return *TLInfo;
}

void NeoVMSubtarget::initializeSubtargetDependencies(StringRef CPU,
                                                     StringRef FS) {
  // Parse CPU/FS strings into feature flags
  if (CPU == "neovm-v1" || CPU == "neovm") {
    HasTryCatch = true;
    HasManifestHints = true;
    HasBigIntegers = true;
  } else if (CPU == "neovm-v2") {
    HasTryCatch = true;
    HasManifestHints = true;
    HasBigIntegers = true;
  }
  
  // Parse feature string
  if (FS.contains("+try-catch")) {
    HasTryCatch = true;
  }
  if (FS.contains("+manifest-hints")) {
    HasManifestHints = true;
  }
  if (FS.contains("+big-integers")) {
    HasBigIntegers = true;
  }

  // Default ABI revision tracks the triple's environment.
  ABIRevision = TargetTriple.getEnvironmentName();

  ParseSubtargetFeatures(CPU, CPU, FS);

  RegInfo = std::make_unique<NeoVMRegisterInfo>();
  InstrInfo = std::make_unique<NeoVMInstrInfo>();
  InstrInfo->setRegisterInfo(RegInfo.get());
  FrameLowering = std::make_unique<NeoVMFrameLowering>();
}

const NeoVMSyscallRegistry& NeoVMSubtarget::getSyscallRegistry() const {
  return NeoVMSyscallRegistry::getInstance();
}
