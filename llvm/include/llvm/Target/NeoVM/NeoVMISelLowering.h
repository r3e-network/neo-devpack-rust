#pragma once

// SelectionDAG / GlobalISel interfaces for NeoVM will live here.
// Metadata emission strategy described in docs/neo-n3-backend.md (Metadata Emission in Instruction Selection).

#include "llvm/CodeGen/TargetLowering.h"
#include "llvm/IR/CallingConv.h"

namespace llvm {

class NeoVMTargetMachine;
class NeoVMSubtarget;

class NeoVMTargetLowering : public TargetLowering {
public:
  NeoVMTargetLowering(const NeoVMTargetMachine &TM, const NeoVMSubtarget &STI);

  SDValue LowerOperation(SDValue Op, SelectionDAG &DAG) const override;
  bool isOffsetFoldingLegal(const GlobalAddressSDNode *GA) const override {
    return false;
  }

  Register getExceptionPointerRegister(const Constant *) const override {
    return Register();
  }

  Register getExceptionSelectorRegister(const Constant *) const override {
    return Register();
  }
};

} // namespace llvm
