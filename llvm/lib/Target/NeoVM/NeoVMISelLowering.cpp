#include "llvm/Target/NeoVM/NeoVMISelLowering.h"

#include "llvm/CodeGen/SelectionDAG.h"
#include "llvm/IR/Type.h"
#include "llvm/Target/NeoVM/NeoVMTargetMachine.h"
#include "llvm/Target/NeoVM/NeoVMSubtarget.h"
#include "llvm/Target/NeoVM/NeoVMRegisterInfo.h"
#include "llvm/CodeGen/Passes.h"

#define GET_REGINFO_ENUM
#include "NeoVMGenRegisterInfo.inc"
#undef GET_REGINFO_ENUM

using namespace llvm;

NeoVMTargetLowering::NeoVMTargetLowering(const NeoVMTargetMachine &TM,
                                         const NeoVMSubtarget &STI)
    : TargetLowering(TM) {
  addRegisterClass(MVT::i32,
                   STI.getRegisterInfo()->getRegClass(NeoVM::GR32));

  setBooleanContents(ZeroOrOneBooleanContent);
  setBooleanVectorContents(ZeroOrNegativeOneBooleanContent);

  for (auto VT : {MVT::i32}) {
    setOperationAction(ISD::ADD, VT, Legal);
    setOperationAction(ISD::SUB, VT, Legal);
    setOperationAction(ISD::MUL, VT, Legal);
    setOperationAction(ISD::SDIV, VT, Legal);
    setOperationAction(ISD::UDIV, VT, Legal);
    setOperationAction(ISD::SREM, VT, Legal);
    setOperationAction(ISD::UREM, VT, Legal);
    setOperationAction(ISD::AND, VT, Legal);
    setOperationAction(ISD::OR, VT, Legal);
    setOperationAction(ISD::XOR, VT, Legal);
    setOperationAction(ISD::SHL, VT, Legal);
    setOperationAction(ISD::SRA, VT, Legal);
    setOperationAction(ISD::SRL, VT, Legal);
  }

  setOperationAction(ISD::BRCOND, MVT::Other, Legal);
  setOperationAction(ISD::BR, MVT::Other, Legal);

  computeRegisterProperties(STI.getRegisterInfo());
}

SDValue
NeoVMTargetLowering::LowerOperation(SDValue Op, SelectionDAG &DAG) const {
  return SDValue();
}
