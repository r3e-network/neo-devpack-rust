#include "llvm/Target/NeoVM/NeoVMInstrInfo.h"

#include "llvm/CodeGen/MachineInstr.h"
#include "llvm/CodeGen/MachineInstrBuilder.h"
#include "llvm/Target/NeoVM/NeoVMMetadata.h"
#include "llvm/CodeGen/Passes.h"

#define GET_INSTRINFO_CTOR_DTOR
#include "NeoVMGenInstrInfo.inc"
#undef GET_INSTRINFO_CTOR_DTOR

// Stub respecting docs/neo-n3-backend.md (NeoVMInstrInfo & NeoVMRegisterInfo Responsibilities).

using namespace llvm;

NeoVMInstrInfo::NeoVMInstrInfo()
    : NeoVMGenInstrInfo(), RegInfo(nullptr) {}

NeoVMInstrInfo::~NeoVMInstrInfo() = default;

unsigned NeoVMInstrInfo::getPushCount(const MachineInstr &MI) const {
  const MCInstrDesc &Desc = MI.getDesc();
  return Desc.TSFlags & 0xff;
}

unsigned NeoVMInstrInfo::getPopCount(const MachineInstr &MI) const {
  const MCInstrDesc &Desc = MI.getDesc();
  return (Desc.TSFlags >> 8) & 0xff;
}

bool NeoVMInstrInfo::isPureStackOp(unsigned Opcode) const {
  switch (Opcode) {
  // Stack operations
  case NeoVM::DUP:
  case NeoVM::SWAP:
  case NeoVM::OVER:
  case NeoVM::ROT:
  case NeoVM::ROLL:
  case NeoVM::REVERSE3:
  case NeoVM::REVERSE4:
  case NeoVM::REVERSEN:
  case NeoVM::DROP:
  case NeoVM::DROPN:
  case NeoVM::CLEAR:
  case NeoVM::TOALTSTACK:
  case NeoVM::FROMALTSTACK:
  case NeoVM::DUPFROMALTSTACK:
    return true;
  // Constant operations
  case NeoVM::PUSH0:
  case NeoVM::PUSH1:
  case NeoVM::PUSH2:
  case NeoVM::PUSH3:
  case NeoVM::PUSH4:
  case NeoVM::PUSH5:
  case NeoVM::PUSH6:
  case NeoVM::PUSH7:
  case NeoVM::PUSH8:
  case NeoVM::PUSH9:
  case NeoVM::PUSH10:
  case NeoVM::PUSH11:
  case NeoVM::PUSH12:
  case NeoVM::PUSH13:
  case NeoVM::PUSH14:
  case NeoVM::PUSH15:
  case NeoVM::PUSH16:
  case NeoVM::PUSHM1:
    return true;
  default:
    return false;
  }
}

unsigned NeoVMInstrInfo::getUncondBranchOpcode() const {
  return NeoVM::JMP;
}

unsigned NeoVMInstrInfo::getCondBranchOpcode(bool IsNegated) const {
  return IsNegated ? NeoVM::JMPIFNOT : NeoVM::JMPIF;
}

bool NeoVMInstrInfo::expandPostRAPseudo(MachineInstr &MI) const {
  // Expand pseudo instructions into real stack operations
  // For NeoVM, most instructions are already real stack operations
  // Handle any pseudo instructions that need expansion
  switch (MI.getOpcode()) {
    case NeoVM::NEOVM_LOAD_PSEUDO:
      // Convert pseudo load to PUSH instruction
      MI.setDesc(get(NeoVM::PUSH));
      return true;
    case NeoVM::NEOVM_STORE_PSEUDO:
      // Convert pseudo store to POP instruction
      MI.setDesc(get(NeoVM::POP));
      return true;
    default:
      // No expansion needed for real instructions
      return false;
  }
}

void NeoVMInstrInfo::annotateDefaultStackInfo(MachineInstr &MI) const {
  NeoVMStackInfo Info;
  Info.Push = getPushCount(MI);
  Info.Pop = getPopCount(MI);
  Info.TypeHints.assign(Info.Push, static_cast<unsigned>(NeoVMTypeHint::Integer));
  setNeoVMStackInfo(MI, Info);
}

void NeoVMInstrInfo::setRegisterInfo(const NeoVMRegisterInfo *RI) {
  RegInfo = RI;
}
