#pragma once

#include "llvm/TargetParser/Triple.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/CodeGen/TargetFrameLowering.h"
#include "llvm/CodeGen/TargetInstrInfo.h"
#include "llvm/CodeGen/TargetRegisterInfo.h"
#include "llvm/CodeGen/TargetSchedule.h"
#include "llvm/Target/NeoVM/NeoVMSyscalls.h"
#include "llvm/CodeGen/TargetSubtargetInfo.h"
#include <memory>
#include <string>

#define GET_SUBTARGETINFO_HEADER
#include "NeoVMGenSubtargetInfo.inc"
#undef GET_SUBTARGETINFO_HEADER

#define GET_SUBTARGETINFO_ENUM
#include "NeoVMGenSubtargetInfo.inc"
#undef GET_SUBTARGETINFO_ENUM

// Structure defined in docs/neo-n3-backend.md (NeoVMSubtarget Responsibilities).

namespace llvm {

class NeoVMInstrInfo;
class NeoVMFrameLowering;
class NeoVMRegisterInfo;
class NeoVMTargetLowering;
class NeoVMTargetMachine;

class NeoVMSubtarget final : public NeoVMGenSubtargetInfo {
public:
  NeoVMSubtarget(const Triple &TT, StringRef CPU, StringRef FS,
                 const NeoVMTargetMachine &TM);
  ~NeoVMSubtarget();

  const NeoVMInstrInfo &getNeoVMInstrInfo() const;
  const NeoVMFrameLowering &getNeoVMFrameLowering() const;
  const NeoVMRegisterInfo &getNeoVMRegisterInfo() const;

  const TargetInstrInfo *getInstrInfo() const override;
  const TargetFrameLowering *getFrameLowering() const override;
  const TargetRegisterInfo *getRegisterInfo() const override;
  const TargetLowering *getTargetLowering() const override;
  const NeoVMTargetLowering &getNeoVMTargetLowering() const;

  void ParseSubtargetFeatures(StringRef CPU, StringRef TuneCPU,
                              StringRef FS);

  bool hasTryCatch() const { return HasTryCatch; }
  bool hasManifestHints() const { return HasManifestHints; }
  bool hasBigIntegers() const { return HasBigIntegers; }

  StringRef getABIRevision() const { return ABIRevision; }

  // Syscall metadata accessor
  const NeoVMSyscallRegistry& getSyscallRegistry() const;

private:
  Triple TargetTriple;
  void initializeSubtargetDependencies(StringRef CPU, StringRef FS);

  std::unique_ptr<NeoVMInstrInfo> InstrInfo;
  std::unique_ptr<NeoVMFrameLowering> FrameLowering;
  std::unique_ptr<NeoVMRegisterInfo> RegInfo;
  std::unique_ptr<NeoVMTargetLowering> TLInfo;

  bool HasTryCatch = false;
  bool HasManifestHints = false;
  bool HasBigIntegers = true;
  std::string ABIRevision;
};

} // namespace llvm
