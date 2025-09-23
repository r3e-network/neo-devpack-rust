#include "llvm/Target/NeoVM/NeoVMRegisterInfo.h"

#include "llvm/CodeGen/MachineFunction.h"
#include "llvm/CodeGen/TargetSubtargetInfo.h"
#include "llvm/CodeGen/MachineInstrBuilder.h"
#include "llvm/CodeGen/TargetInstrInfo.h"
#include "llvm/ADT/BitVector.h"
#include "llvm/Target/NeoVM/NeoVMFrameLowering.h"
#include "llvm/Target/NeoVM/NeoVMMetadata.h"
#include "llvm/CodeGen/Passes.h"

#define GET_REGINFO_ENUM
#include "NeoVMGenRegisterInfo.inc"
#undef GET_REGINFO_ENUM

#define GET_REGINFO_TARGET_DESC
#include "NeoVMGenRegisterInfo.inc"
#undef GET_REGINFO_TARGET_DESC

// Stub reflecting docs/neo-n3-backend.md (NeoVMInstrInfo & NeoVMRegisterInfo Responsibilities).

using namespace llvm;

NeoVMRegisterInfo::NeoVMRegisterInfo()
    : NeoVMGenRegisterInfo(/*RA=*/0, /*Dwarf=*/0, /*EH=*/0, /*PC=*/0,
                           /*HwMode=*/0) {}

NeoVMRegisterInfo::~NeoVMRegisterInfo() = default;

const MCPhysReg *
NeoVMRegisterInfo::getCalleeSavedRegs(const MachineFunction *) const {
  // No callee-saved registers: stack fully managed per-call.
  return nullptr;
}

const uint32_t *NeoVMRegisterInfo::getCallPreservedMask(const MachineFunction &,
                                                        CallingConv::ID) const {
  return nullptr;
}

bool NeoVMRegisterInfo::eliminateFrameIndex(MachineBasicBlock::iterator II,
                                             int SPAdj, unsigned FIOperandNum,
                                             RegScavenger *RS) const {
  // Translate frame index references into VM storage operations
  MachineInstr &MI = *II;
  MachineFunction &MF = *MI.getParent()->getParent();
  MachineBasicBlock &MBB = *MI.getParent();
  const TargetInstrInfo *TII = MF.getSubtarget().getInstrInfo();
  
  // Get frame index from the instruction
  int FrameIndex = MI.getOperand(FIOperandNum).getIndex();
  
  // For NeoVM, frame indices are handled through stack operations
  // Convert frame index to stack offset
  int64_t StackOffset = FrameIndex * 4; // 4 bytes per stack slot
  
  if (SPAdj != 0) {
    // Adjust stack pointer if needed
    StackOffset += SPAdj;
  }
  
  // Emit stack operations to access frame index
  if (StackOffset != 0) {
    // Push stack offset
    if (StackOffset >= -128 && StackOffset <= 127) {
      BuildMI(MBB, II, MI.getDebugLoc(), TII->get(NeoVM::PUSHINT8))
          .addImm(StackOffset);
    } else if (StackOffset >= -32768 && StackOffset <= 32767) {
      BuildMI(MBB, II, MI.getDebugLoc(), TII->get(NeoVM::PUSHINT16))
          .addImm(StackOffset);
    } else {
      BuildMI(MBB, II, MI.getDebugLoc(), TII->get(NeoVM::PUSHINT32))
          .addImm(StackOffset);
    }
  }
  
  // Replace frame index operand with stack offset
  MI.getOperand(FIOperandNum).ChangeToImmediate(StackOffset);
  
  return true;
}

BitVector NeoVMRegisterInfo::getReservedRegs(const MachineFunction &) const {
  BitVector Reserved(getNumRegs());
  // Reserve the stack register for NeoVM evaluation stack
  Reserved.set(NeoVM::STACK);
  
  // Reserve any additional registers that should not be allocated
  // For NeoVM, we only have the stack register, so no additional reservations needed
  
  return Reserved;
}

Register NeoVMRegisterInfo::getFrameRegister(const MachineFunction &) const {
  // NeoVM currently models the evaluation stack; no dedicated frame register.
  return Register();
}
