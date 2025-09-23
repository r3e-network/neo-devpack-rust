#include "llvm/Target/NeoVM/NeoVMStackify.h"

#include "llvm/ADT/SmallVector.h"
#include "llvm/CodeGen/MachineFunctionPass.h"
#include "llvm/CodeGen/MachineInstrBuilder.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/Support/Debug.h"
#include "llvm/CodeGen/TargetInstrInfo.h"
#include "llvm/CodeGen/TargetSubtargetInfo.h"
#include "llvm/Target/NeoVM/NeoVMMetadata.h"
#include "llvm/Target/NeoVM/NeoVMSyscalls.h"
#include "llvm/CodeGen/Passes.h"

#include <functional>

#define DEBUG_TYPE "neovm-stackify"

// Stubs for passes outlined in docs/neo-n3-backend.md (NeoVMFrameLowering & Stack Discipline Passes).

using namespace llvm;

namespace {
class NeoVMStackifyPass : public MachineFunctionPass {
public:
  static char ID;
  NeoVMStackifyPass() : MachineFunctionPass(ID) {}

  StringRef getPassName() const override { return "NeoVM Stackify"; }

  bool runOnMachineFunction(MachineFunction &MF) override {
    bool Changed = false;
    for (MachineBasicBlock &MBB : MF) {
      Changed |= processBlock(MBB);
    }
    return Changed;
  }

private:
  struct StackEntry {
    unsigned VReg = 0;
    MachineInstr *Producer = nullptr;
  };

  bool ensureOperandsOnStack(MachineBasicBlock &MBB, MachineInstr &MI,
                             SmallVectorImpl<StackEntry> &Stack,
                             unsigned Pop, unsigned Push) {
    LLVM_DEBUG(dbgs() << "[NeoVMStackify] ensure operands for " << MI);
    bool Changed = false;
    if (Pop > Stack.size())
      return Changed;

    MachineFunction &MF = *MBB.getParent();
    const TargetInstrInfo *TII = MF.getSubtarget().getInstrInfo();

    SmallVector<Register, 4> UseRegs;
    for (const MachineOperand &MO : MI.operands()) {
      if (!MO.isReg() || !MO.getReg() || MO.isDef())
        continue;
      UseRegs.push_back(MO.getReg());
    }

    unsigned Required = std::min<unsigned>(UseRegs.size(), Pop);
    for (int I = static_cast<int>(Required) - 1; I >= 0; --I) {
      Register Reg = UseRegs[I];
      int Index = -1;
      for (int S = static_cast<int>(Stack.size()) - 1; S >= 0; --S) {
        if (Stack[S].VReg == Reg) {
          Index = S;
          break;
        }
      }

      if (Index == -1) {
        LLVM_DEBUG(dbgs() << "[NeoVMStackify] operand vreg " << Reg
                          << " not found on stack before " << MI);
        continue;
      }

      unsigned Depth = Stack.size() - 1 - Index;
      if (Depth == 0)
        continue;

      BuildMI(MBB, MI, MI.getDebugLoc(), TII->get(NeoVM::SWAP))
          .addImm(Depth)
          .addImm(1);
      StackEntry Entry = Stack[Index];
      Stack.erase(Stack.begin() + Index);
      Stack.push_back(Entry);
      Changed = true;
    }

    (void)Push;
    return Changed;
  }

  bool spillIfNeeded(MachineBasicBlock &MBB, MachineInstr &MI,
                     SmallVectorImpl<StackEntry> &Stack) {
    const unsigned SpillThreshold = 512;
    if (Stack.size() <= SpillThreshold)
      return false;

    LLVM_DEBUG(dbgs() << "[NeoVMStackify] spill triggered at depth "
                      << Stack.size() << " before " << MI);
    
    // For NeoVM, we simply limit stack depth to prevent overflow
    // In a real implementation, this would spill to VM storage
    MachineFunction &MF = *MBB.getParent();
    const TargetInstrInfo *TII = MF.getSubtarget().getInstrInfo();
    
    // Emit POP instructions to reduce stack depth
    unsigned Excess = Stack.size() - SpillThreshold;
    for (unsigned i = 0; i < Excess; ++i) {
      BuildMI(MBB, MI, MI.getDebugLoc(), TII->get(NeoVM::POP));
    }
    
    Stack.resize(SpillThreshold);
    return true;
  }

  bool processBlock(MachineBasicBlock &MBB) {
    MachineFunction &MF = *MBB.getParent();
    bool Changed = false;
    SmallVector<StackEntry, 16> Stack;
    for (auto MI = MBB.begin(), E = MBB.end(); MI != E; ++MI) {
     unsigned Push = MI->getDesc().TSFlags & 0xff;
     unsigned Pop = (MI->getDesc().TSFlags >> 8) & 0xff;

      std::optional<NeoVMStackInfo> Info = getNeoVMStackInfo(*MI);
      if (Info) {
        Push = Info->Push;
        Pop = Info->Pop;
      } else {
        NeoVMStackInfo Auto;
        Auto.Push = Push;
        Auto.Pop = Pop;
        setNeoVMStackInfo(*MI, Auto);
        Info = Auto;
      }

      Changed |= ensureOperandsOnStack(MBB, *MI, Stack, Pop, Push);
      Changed |= spillIfNeeded(MBB, *MI, Stack);

      if (Stack.size() < Pop) {
        LLVM_DEBUG(dbgs() << "[NeoVMStackify] stack underflow in block "
                          << MBB.getName() << " before instruction: " << *MI);
        Stack.clear();
      } else {
        for (unsigned I = 0; I < Pop; ++I)
          Stack.pop_back();
      }

      for (unsigned I = 0, DefIdx = 0; I < Push; ++I) {
        StackEntry Entry;
        Entry.Producer = &*MI;
        Entry.VReg = 0;
        for (; DefIdx < MI->getNumOperands(); ++DefIdx) {
          const MachineOperand &MO = MI->getOperand(DefIdx);
          if (MO.isReg() && MO.isDef() && MO.getReg()) {
            Entry.VReg = MO.getReg();
            ++DefIdx;
            break;
          }
        }
        Stack.push_back(Entry);
      }

     if (MI->isTerminator()) {
       emitNeoVMStackSync(*MF.getSubtarget().getInstrInfo(), MBB, std::next(MI),
                          MI->getDebugLoc(), Info);
        Changed = true;
      }
    }
    return Changed;
  }
};

char NeoVMStackifyPass::ID = 0;

class NeoVMStackHeightVerifierPass : public MachineFunctionPass {
public:
  static char ID;
  NeoVMStackHeightVerifierPass() : MachineFunctionPass(ID) {}

  StringRef getPassName() const override { return "NeoVM Stack Height Verifier"; }

  bool runOnMachineFunction(MachineFunction &MF) override {
    bool IssueFound = false;
    for (MachineBasicBlock &MBB : MF) {
      int Depth = 0;
    for (MachineInstr &MI : MBB) {
      unsigned Push = MI.getDesc().TSFlags & 0xff;
      unsigned Pop = (MI.getDesc().TSFlags >> 8) & 0xff;
      if (auto Info = getNeoVMStackInfo(MI)) {
        Push = Info->Push;
        Pop = Info->Pop;
      }

      if (Depth < static_cast<int>(Pop)) {
        LLVM_DEBUG(dbgs() << "[NeoVMVerifier] stack underflow in block "
                          << MBB.getName() << " at instruction " << MI);
          IssueFound = true;
          Depth = 0;
        } else {
          Depth -= Pop;
        }
        Depth += Push;
      }
    }
    return IssueFound;
  }
};

char NeoVMStackHeightVerifierPass::ID = 0;

class NeoVMIntrinsicLoweringPass : public FunctionPass {
public:
  static char ID;
  NeoVMIntrinsicLoweringPass() : FunctionPass(ID) {}

  StringRef getPassName() const override { return "NeoVM Intrinsic Lowering"; }

  bool runOnFunction(Function &) override { return false; }
};

char NeoVMIntrinsicLoweringPass::ID = 0;
} // namespace

std::unique_ptr<MachineFunctionPass> llvm::createNeoVMStackifyPass() {
  return std::make_unique<NeoVMStackifyPass>();
}

std::unique_ptr<MachineFunctionPass>
llvm::createNeoVMStackHeightVerifierPass() {
  return std::make_unique<NeoVMStackHeightVerifierPass>();
}

std::unique_ptr<FunctionPass> llvm::createNeoVMIntrinsicLoweringPass() {
  return std::make_unique<NeoVMIntrinsicLoweringPass>();
}
