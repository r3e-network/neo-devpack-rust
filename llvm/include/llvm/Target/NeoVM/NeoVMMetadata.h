#pragma once

#include "llvm/ADT/SmallVector.h"
#include "llvm/CodeGen/MachineBasicBlock.h"
#include "llvm/CodeGen/MachineInstr.h"
#include <optional>

#define GET_INSTRINFO_ENUM
#include "NeoVMGenInstrInfo.inc"
#undef GET_INSTRINFO_ENUM

#include "llvm/Target/NeoVM/Generated/NeoVMOpcodeBytes.inc"

namespace llvm {

struct NeoVMStackInfo {
  unsigned Push = 0;
  unsigned Pop = 0;
  SmallVector<unsigned, 4> TypeHints;
};

enum class NeoVMTypeHint : unsigned {
  Integer = 0,
  Boolean = 1,
  ByteString = 2,
  Array = 3,
  Map = 4,
  Struct = 5,
  Interface = 6,
  Void = 7,
};

std::optional<NeoVMStackInfo> getNeoVMStackInfo(const MachineInstr &MI);
void setNeoVMStackInfo(MachineInstr &MI, const NeoVMStackInfo &Info);

void emitNeoVMStackSync(const TargetInstrInfo &TII, MachineBasicBlock &MBB,
                       MachineBasicBlock::iterator I, const DebugLoc &DL,
                       const std::optional<NeoVMStackInfo> &Info);

} // namespace llvm
