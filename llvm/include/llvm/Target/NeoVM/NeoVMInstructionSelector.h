#pragma once

#include "llvm/CodeGen/GlobalISel/InstructionSelector.h"
#include "llvm/CodeGen/MachineFunctionPass.h"
#include "llvm/Target/NeoVM/NeoVMMetadata.h"
#include <memory>

namespace llvm {

class NeoVMInstrInfo;
class NeoVMRegisterInfo;
class NeoVMSubtarget;
class MachineFunction;

class NeoVMInstructionSelector : public InstructionSelector {
public:
  NeoVMInstructionSelector(const NeoVMInstrInfo &TII,
                           const NeoVMRegisterInfo &TRI,
                           const NeoVMSubtarget &STI);

  bool select(MachineInstr &MI) override;
  void setupGeneratedPerFunctionState(MachineFunction &) override {}

private:
  bool selectBinary(MachineInstr &MI, unsigned Opcode, NeoVMTypeHint Hint);
  bool selectICmp(MachineInstr &MI);
  bool selectBranch(MachineInstr &MI);
  bool selectLoadStore(MachineInstr &MI);

  const NeoVMInstrInfo &TII;
  const NeoVMRegisterInfo &TRI;
  const NeoVMSubtarget &STI;
};

std::unique_ptr<MachineFunctionPass> createNeoVMInstructionSelectPass();

} // namespace llvm
