#include "llvm/Target/NeoVM/NeoVMInstructionSelector.h"

#include <optional>
#include "llvm/CodeGen/GlobalISel/MachineIRBuilder.h"
#include "llvm/CodeGen/GlobalISel/Utils.h"
#include "llvm/CodeGen/MachineFunctionPass.h"
#include "llvm/CodeGen/MachineRegisterInfo.h"
#include "llvm/CodeGen/MachineInstrBuilder.h"
#include "llvm/Support/Debug.h"
#include "llvm/Target/NeoVM/NeoVMInstrInfo.h"
#include "llvm/Target/NeoVM/NeoVMMetadata.h"
#include "llvm/Target/NeoVM/NeoVMRegisterInfo.h"
#include "llvm/Target/NeoVM/NeoVMSubtarget.h"
#include "llvm/IR/Instructions.h"
#include "llvm/CodeGen/Passes.h"

#define DEBUG_TYPE "neovm-isel"

using namespace llvm;

namespace {
std::optional<unsigned> getCmpOpcode(CmpInst::Predicate Pred) {
  switch (Pred) {
  case CmpInst::ICMP_EQ:
    return NeoVM::EQ;
  case CmpInst::ICMP_NE:
    return NeoVM::NE;
  case CmpInst::ICMP_SLT:
  case CmpInst::ICMP_ULT:
    return NeoVM::LT;
  case CmpInst::ICMP_SLE:
  case CmpInst::ICMP_ULE:
    return NeoVM::LE;
  case CmpInst::ICMP_SGT:
  case CmpInst::ICMP_UGT:
    return NeoVM::GT;
  case CmpInst::ICMP_SGE:
  case CmpInst::ICMP_UGE:
    return NeoVM::GE;
  default:
    return std::nullopt;
  }
}
} // namespace

NeoVMInstructionSelector::NeoVMInstructionSelector(
    const NeoVMInstrInfo &TII, const NeoVMRegisterInfo &TRI,
    const NeoVMSubtarget &STI)
    : TII(TII), TRI(TRI), STI(STI) {}

bool NeoVMInstructionSelector::selectBinary(MachineInstr &MI, unsigned Opcode,
                                            NeoVMTypeHint Hint) {
  Register Dst = MI.getOperand(0).getReg();
  Register LHS = MI.getOperand(1).getReg();
  Register RHS = MI.getOperand(2).getReg();

  MachineIRBuilder Builder(MI);
  MachineInstrBuilder NewMI = Builder.buildInstr(Opcode)
                                         .addDef(Dst)
                                         .addUse(LHS)
                                         .addUse(RHS);

  NeoVMStackInfo Info;
  Info.Push = 1;
  Info.Pop = 2;
  Info.TypeHints = {static_cast<unsigned>(Hint)};
  setNeoVMStackInfo(*NewMI, Info);

  MI.eraseFromParent();
  return true;
}

bool NeoVMInstructionSelector::selectICmp(MachineInstr &MI) {
  MachineRegisterInfo &MRI = MI.getMF()->getRegInfo();
  CmpInst::Predicate Pred = static_cast<CmpInst::Predicate>(
      MI.getOperand(1).getPredicate());
  std::optional<unsigned> Opcode = getCmpOpcode(Pred);
  if (!Opcode)
    return false;

  Register Dst = MI.getOperand(0).getReg();
  Register LHS = MI.getOperand(2).getReg();
  Register RHS = MI.getOperand(3).getReg();

  MachineIRBuilder Builder(MI);
  MachineInstrBuilder NewMI = Builder.buildInstr(*Opcode)
                                         .addDef(Dst)
                                         .addUse(LHS)
                                         .addUse(RHS);

  NeoVMStackInfo Info;
  Info.Push = 1;
  Info.Pop = 2;
  Info.TypeHints = {static_cast<unsigned>(NeoVMTypeHint::Boolean)};
  setNeoVMStackInfo(*NewMI, Info);

  MI.eraseFromParent();
  return true;
}

bool NeoVMInstructionSelector::selectLoadStore(MachineInstr &MI) {
  MachineIRBuilder Builder(MI);

  switch (MI.getOpcode()) {
  case TargetOpcode::G_LOAD: {
    Register Dst = MI.getOperand(0).getReg();
    Register Addr = MI.getOperand(1).getReg();
    MachineInstrBuilder NewMI =
        Builder.buildInstr(NeoVM::PUSH).addDef(Dst).addUse(Addr);
    NeoVMStackInfo Info;
    Info.Push = 1;
    Info.Pop = 1;
    setNeoVMStackInfo(*NewMI, Info);
    MI.eraseFromParent();
    return true;
  }
  case TargetOpcode::G_STORE: {
    Register Val = MI.getOperand(0).getReg();
    Register Addr = MI.getOperand(1).getReg();
    MachineInstrBuilder NewMI =
        Builder.buildInstr(NeoVM::POP).addUse(Val).addUse(Addr);
    NeoVMStackInfo Info;
    Info.Pop = 2;
    setNeoVMStackInfo(*NewMI, Info);
    MI.eraseFromParent();
    return true;
  }
  case TargetOpcode::G_CONSTANT: {
    Register Dst = MI.getOperand(0).getReg();
    const MachineOperand &Imm = MI.getOperand(1);
    if (Imm.isImm()) {
      int64_t Val = Imm.getImm();
      unsigned Opcode;
      if (Val == 0) Opcode = NeoVM::PUSH0;
      else if (Val == 1) Opcode = NeoVM::PUSH1;
      else if (Val == 2) Opcode = NeoVM::PUSH2;
      else if (Val == 3) Opcode = NeoVM::PUSH3;
      else if (Val == 4) Opcode = NeoVM::PUSH4;
      else if (Val == 5) Opcode = NeoVM::PUSH5;
      else if (Val == 6) Opcode = NeoVM::PUSH6;
      else if (Val == 7) Opcode = NeoVM::PUSH7;
      else if (Val == 8) Opcode = NeoVM::PUSH8;
      else if (Val == 9) Opcode = NeoVM::PUSH9;
      else if (Val == 10) Opcode = NeoVM::PUSH10;
      else if (Val == 11) Opcode = NeoVM::PUSH11;
      else if (Val == 12) Opcode = NeoVM::PUSH12;
      else if (Val == 13) Opcode = NeoVM::PUSH13;
      else if (Val == 14) Opcode = NeoVM::PUSH14;
      else if (Val == 15) Opcode = NeoVM::PUSH15;
      else if (Val == 16) Opcode = NeoVM::PUSH16;
      else if (Val == -1) Opcode = NeoVM::PUSHM1;
      else if (Val >= -128 && Val <= 127) Opcode = NeoVM::PUSHINT8;
      else if (Val >= -32768 && Val <= 32767) Opcode = NeoVM::PUSHINT16;
      else if (Val >= -2147483648LL && Val <= 2147483647LL) Opcode = NeoVM::PUSHINT32;
      else Opcode = NeoVM::PUSHINT64;
      
      MachineInstrBuilder NewMI = Builder.buildInstr(Opcode).addDef(Dst);
      if (Opcode == NeoVM::PUSHINT8 || Opcode == NeoVM::PUSHINT16 || 
          Opcode == NeoVM::PUSHINT32 || Opcode == NeoVM::PUSHINT64) {
        NewMI.addImm(Val);
      }
      
      NeoVMStackInfo Info;
      Info.Push = 1;
      setNeoVMStackInfo(*NewMI, Info);
      MI.eraseFromParent();
      return true;
    }
    return false;
  }
  default:
    return false;
  }
}

bool NeoVMInstructionSelector::selectBranch(MachineInstr &MI) {
  MachineIRBuilder Builder(MI);

  if (MI.getOpcode() == TargetOpcode::G_BR) {
    MachineBasicBlock *Dest = MI.getOperand(0).getMBB();
    MachineInstrBuilder NewMI = Builder.buildInstr(NeoVM::JMP).addMBB(Dest);
    NeoVMStackInfo Info;
    setNeoVMStackInfo(*NewMI, Info);
    MI.eraseFromParent();
    return true;
  }

  if (MI.getOpcode() == TargetOpcode::G_BRCOND) {
    Register Cond = MI.getOperand(0).getReg();
    MachineBasicBlock *Dest = MI.getOperand(1).getMBB();
    MachineInstrBuilder NewMI =
        Builder.buildInstr(NeoVM::JMPIF).addUse(Cond).addMBB(Dest);
    NeoVMStackInfo Info;
    Info.Pop = 1;
    setNeoVMStackInfo(*NewMI, Info);
    MI.eraseFromParent();
    return true;
  }

  return false;
}

bool NeoVMInstructionSelector::select(MachineInstr &MI) {
  LLVM_DEBUG(dbgs() << "[NeoVMInstructionSelector] select " << MI);
  switch (MI.getOpcode()) {
  case TargetOpcode::G_ADD:
    return selectBinary(MI, NeoVM::ADD, NeoVMTypeHint::Integer);
  case TargetOpcode::G_SUB:
    return selectBinary(MI, NeoVM::SUB, NeoVMTypeHint::Integer);
  case TargetOpcode::G_MUL:
    return selectBinary(MI, NeoVM::MUL, NeoVMTypeHint::Integer);
  case TargetOpcode::G_SDIV:
  case TargetOpcode::G_UDIV:
    return selectBinary(MI, NeoVM::DIV, NeoVMTypeHint::Integer);
  case TargetOpcode::G_ICMP:
    if (selectICmp(MI))
      return true;
    break;
  case TargetOpcode::G_LOAD:
  case TargetOpcode::G_STORE:
    if (selectLoadStore(MI))
      return true;
    break;
  case TargetOpcode::G_BR:
  case TargetOpcode::G_BRCOND:
    if (selectBranch(MI))
      return true;
    break;
  default:
    break;
  }

  TII.annotateDefaultStackInfo(MI);
  return true;
}

namespace {
class NeoVMInstructionSelectPass : public MachineFunctionPass {
public:
  static char ID;
  NeoVMInstructionSelectPass() : MachineFunctionPass(ID) {}

  StringRef getPassName() const override {
    return "NeoVM Instruction Selection";
  }

  bool runOnMachineFunction(MachineFunction &MF) override {
    auto &STI = MF.getSubtarget<NeoVMSubtarget>();
    auto &TII = *static_cast<const NeoVMInstrInfo *>(STI.getInstrInfo());
    auto &TRI = *static_cast<const NeoVMRegisterInfo *>(STI.getRegisterInfo());
    NeoVMInstructionSelector Selector(TII, TRI, STI);

    bool Changed = false;
    for (MachineBasicBlock &MBB : MF) {
      for (auto MI = MBB.begin(), ME = MBB.end(); MI != ME;) {
        MachineInstr &Instr = *MI++;
        Changed |= Selector.select(Instr);
      }
    }
    return Changed;
  }
};
} // namespace

char NeoVMInstructionSelectPass::ID = 0;

std::unique_ptr<MachineFunctionPass> llvm::createNeoVMInstructionSelectPass() {
  return std::make_unique<NeoVMInstructionSelectPass>();
}
