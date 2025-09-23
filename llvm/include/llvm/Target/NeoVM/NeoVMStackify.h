#pragma once

#include <memory>

namespace llvm {

class FunctionPass;
class MachineFunctionPass;

std::unique_ptr<MachineFunctionPass> createNeoVMStackifyPass();
std::unique_ptr<MachineFunctionPass> createNeoVMStackHeightVerifierPass();
std::unique_ptr<FunctionPass> createNeoVMIntrinsicLoweringPass();

} // namespace llvm

