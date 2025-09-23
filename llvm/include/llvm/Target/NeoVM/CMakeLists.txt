# Header install list for NeoVM backend per docs/neo-n3-backend.md (Build Integration).
set(NEOVM_HEADERS
  NeoVMTargetInfo.h
  NeoVMTargetMachine.h
  NeoVMInstrInfo.h
  NeoVMRegisterInfo.h
  NeoVMFrameLowering.h
  NeoVMAsmPrinter.h
  NeoVMStackify.h
  NeoVMISelLowering.h
  NeoVMSubtarget.h
  MCTargetDesc/NeoVMAsmInfo.h
  MCTargetDesc/NeoVMMCTargetDesc.h
  NeoVMTargetObjectFile.h
)

# Public headers for NeoVM target
llvm_add_public_headers(NeoVM
  NeoVMAsmPrinter.h
  NeoVMFrameLowering.h
  NeoVMInstrInfo.h
  NeoVMInstructionSelector.h
  NeoVMISelLowering.h
  NeoVMMetadata.h
  NeoVMNEF.h
  NeoVMRegisterInfo.h
  NeoVMStackify.h
  NeoVMSubtarget.h
  NeoVMSyscalls.h
  NeoVMTargetMachine.h
  NeoVMTargetObjectFile.h
  MCTargetDesc/NeoVMAsmInfo.h
  MCTargetDesc/NeoVMMCTargetDesc.h
)
