#pragma once

#include "llvm/CodeGen/TargetInstrInfo.h"

#define GET_INSTRINFO_HEADER
#include "NeoVMGenInstrInfo.inc"
#undef GET_INSTRINFO_HEADER

// Mirrors the plan in docs/neo-n3-backend.md (NeoVMInstrInfo & NeoVMRegisterInfo Responsibilities).

namespace llvm {

class MachineInstr;
class MachineBasicBlock;
class NeoVMRegisterInfo;

class NeoVMInstrInfo final : public NeoVMGenInstrInfo {
public:
  NeoVMInstrInfo();
  ~NeoVMInstrInfo() override;

  // Stack effect helpers.
  unsigned getPushCount(const MachineInstr &MI) const;
  unsigned getPopCount(const MachineInstr &MI) const;
  bool isPureStackOp(unsigned Opcode) const;

  // Branch utilities.
  unsigned getUncondBranchOpcode() const;
  unsigned getCondBranchOpcode(bool IsNegated) const;

  // Pseudo expansion entry point.
  bool expandPostRAPseudo(MachineInstr &MI) const override;

  // Helper to attach default stack metadata derived from TSFlags.
  void annotateDefaultStackInfo(MachineInstr &MI) const;

  void setRegisterInfo(const NeoVMRegisterInfo *RI);

private:
  const NeoVMRegisterInfo *RegInfo; // Set by subtarget during construction.
};

} // namespace llvm
