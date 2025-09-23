#include "llvm/Target/NeoVM/NeoVMFrameLowering.h"

#include "llvm/CodeGen/MachineBasicBlock.h"
#include "llvm/CodeGen/MachineFunction.h"
#include "llvm/CodeGen/MachineInstrBuilder.h"
#include "llvm/CodeGen/TargetInstrInfo.h"
#include "llvm/Target/NeoVM/NeoVMMetadata.h"
#include "llvm/CodeGen/Passes.h"

// Stub conforming to docs/neo-n3-backend.md (NeoVMFrameLowering & Stack Discipline Passes).

using namespace llvm;

NeoVMFrameLowering::NeoVMFrameLowering()
    : TargetFrameLowering(StackGrowsDown, Align(1), 0) {}

NeoVMFrameLowering::~NeoVMFrameLowering() = default;

void NeoVMFrameLowering::emitPrologue(MachineFunction &MF, MachineBasicBlock &MBB) const {
  // Ensure stack height baseline established and manifest context initialized
  MachineBasicBlock::iterator MBBI = MBB.begin();
  DebugLoc DL = MBBI != MBB.end() ? MBBI->getDebugLoc() : DebugLoc();
  
  // Initialize stack height to 0
  const TargetInstrInfo *TII = MF.getSubtarget().getInstrInfo();
  BuildMI(MBB, MBBI, DL, TII->get(NeoVM::PUSH0));
  
  // Initialize manifest context if needed
  if (MF.getFunction().hasFnAttribute("neovm-manifest")) {
    // Emit manifest initialization code
    BuildMI(MBB, MBBI, DL, TII->get(NeoVM::PUSH0));
  }
}

void NeoVMFrameLowering::emitEpilogue(MachineFunction &MF, MachineBasicBlock &MBB) const {
  // Verify stack height returns to zero before RET emission
  MachineBasicBlock::iterator MBBI = MBB.getLastNonDebugInstr();
  DebugLoc DL = MBBI != MBB.end() ? MBBI->getDebugLoc() : DebugLoc();
  
  (void)MF;
  (void)MBB;
}

MachineBasicBlock::iterator NeoVMFrameLowering::eliminateCallFramePseudoInstr(
    MachineFunction &, MachineBasicBlock &,
    MachineBasicBlock::iterator MI) const {
  // No dedicated call frame pseudos required for NeoVM.
  return MI;
}

void NeoVMFrameLowering::materializeFrameObject(MachineBasicBlock &MBB,
                                                MachineBasicBlock::iterator MI,
                                                int FrameIndex, int64_t Offset) const {
  // Serialize frame objects via VM array/storage operations
  DebugLoc DL = MI != MBB.end() ? MI->getDebugLoc() : DebugLoc();
  const TargetInstrInfo *TII = MBB.getParent()->getSubtarget().getInstrInfo();
  
  // Emit frame object materialization
  // For NeoVM, frame objects are handled through stack operations
  BuildMI(MBB, MI, DL, TII->get(NeoVM::PUSH))
      .addImm(FrameIndex)
      .addImm(Offset);
}
