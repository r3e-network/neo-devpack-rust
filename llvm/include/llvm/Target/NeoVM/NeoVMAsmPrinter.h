#pragma once

#include "llvm/CodeGen/AsmPrinter.h"
#include "llvm/Target/NeoVM/NeoVMNEF.h"

// Stub reflecting docs/neo-n3-backend.md (MC Emission Workflow).

namespace llvm {

class NeoVMAsmPrinter final : public AsmPrinter {
public:
  NeoVMAsmPrinter(TargetMachine &TM, std::unique_ptr<MCStreamer> Streamer);
  ~NeoVMAsmPrinter() override;

  StringRef getPassName() const override;

  void emitInstruction(const MachineInstr *) override;
  void emitFunctionEntryLabel() override;

  bool runOnMachineFunction(MachineFunction &MF) override;

private:
  // Scratch container used to materialise NEF artefacts for tests.
  std::unique_ptr<NEFContainer> NefContainer;

  void writeScript(const MachineFunction &MF);
  void writeManifest(const MachineFunction &MF);
};

} // namespace llvm
