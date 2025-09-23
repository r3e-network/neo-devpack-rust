#include "llvm/Target/NeoVM/NeoVMMetadata.h"

using namespace llvm;

std::optional<NeoVMStackInfo> getNeoVMStackInfo(const MachineInstr &) {
  return std::nullopt;
}

void setNeoVMStackInfo(MachineInstr &, const NeoVMStackInfo &) {}

void emitNeoVMStackSync(const TargetInstrInfo &, MachineBasicBlock &,
                       MachineBasicBlock::iterator, const DebugLoc &,
                       const std::optional<NeoVMStackInfo> &) {}
