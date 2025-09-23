#pragma once

#include "llvm/CodeGen/TargetFrameLowering.h"

// Structured according to docs/neo-n3-backend.md (NeoVMFrameLowering & Stack Discipline Passes).

namespace llvm {

class MachineFunction;
class MachineBasicBlock;
class MachineInstrBuilder;

class NeoVMFrameLowering final : public TargetFrameLowering {
public:
  NeoVMFrameLowering();
  ~NeoVMFrameLowering() override;

  void emitPrologue(MachineFunction &MF, MachineBasicBlock &MBB) const override;
  void emitEpilogue(MachineFunction &MF, MachineBasicBlock &MBB) const override;

  bool hasFP(const MachineFunction &) const override { return false; }

  MachineBasicBlock::iterator
  eliminateCallFramePseudoInstr(MachineFunction &, MachineBasicBlock &,
                                MachineBasicBlock::iterator) const override;

  // Helper used by NeoVMRegisterInfo::eliminateFrameIndex.
  void materializeFrameObject(MachineBasicBlock &MBB,
                              MachineBasicBlock::iterator MI,
                              int FrameIndex, int64_t Offset) const;
};

} // namespace llvm
