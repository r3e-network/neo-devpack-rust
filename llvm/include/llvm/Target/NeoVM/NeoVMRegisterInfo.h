#pragma once

#include "llvm/CodeGen/TargetRegisterInfo.h"

#define GET_REGINFO_HEADER
#include "NeoVMGenRegisterInfo.inc"
#undef GET_REGINFO_HEADER

// Tracks design from docs/neo-n3-backend.md (NeoVMInstrInfo & NeoVMRegisterInfo Responsibilities).

namespace llvm {

class NeoVMRegisterInfo final : public NeoVMGenRegisterInfo {
public:
  NeoVMRegisterInfo();
  ~NeoVMRegisterInfo() override;

  const MCPhysReg *getCalleeSavedRegs(const MachineFunction *) const override;
  const uint32_t *getCallPreservedMask(const MachineFunction &, CallingConv::ID) const override;

  bool eliminateFrameIndex(MachineBasicBlock::iterator II, int SPAdj,
                           unsigned FIOperandNum,
                           RegScavenger *RS = nullptr) const override;

  bool requiresRegisterScavenging(const MachineFunction &) const override { return false; }
  bool trackLivenessAfterRegAlloc(const MachineFunction &) const override { return false; }
  BitVector getReservedRegs(const MachineFunction &MF) const override;
  Register getFrameRegister(const MachineFunction &MF) const override;
};

} // namespace llvm
