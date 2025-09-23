set(LLVM_TARGET_DEFINITIONS NeoVM.td)

tablegen(LLVM NeoVMGenInstrInfo.inc -gen-instr-info)
tablegen(LLVM NeoVMGenRegisterInfo.inc -gen-register-info)
tablegen(LLVM NeoVMGenSubtargetInfo.inc -gen-subtarget)

add_public_tablegen_target(NeoVMCommonTableGen)

add_library(NeoVMCodeGen STATIC
  NeoVMTargetMachine.cpp
  NeoVMInstrInfo.cpp
  NeoVMInstructionSelector.cpp
  NeoVMRegisterInfo.cpp
  NeoVMFrameLowering.cpp
  NeoVMAsmPrinter.cpp
  NeoVMISelLowering.cpp
  NeoVMStackify.cpp
  NeoVMMetadata.cpp
  NeoVMNEF.cpp
  NeoVMSyscalls.cpp
  MCTargetDesc/NeoVMMCTargetDesc.cpp
  MCTargetDesc/NeoVMAsmInfo.cpp
  NeoVMTargetObjectFile.cpp
  NeoVMSubtarget.cpp
  NeoVMTargetInfo.cpp
)

add_dependencies(NeoVMCodeGen NeoVMCommonTableGen)

llvm_map_components_to_libnames(NeoVM_LINK_LIBS
  Analysis
  AsmPrinter
  CodeGen
  Core
  GlobalISel
  MC
  SelectionDAG
  Support
  Target
)

target_include_directories(NeoVMCodeGen
  PRIVATE
    ${LLVM_INCLUDE_DIRS}
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/..
    ${PROJECT_SOURCE_DIR}/llvm/include
    ${PROJECT_BINARY_DIR}/llvm/lib/Target/NeoVM
)

target_compile_definitions(NeoVMCodeGen PRIVATE ${LLVM_DEFINITIONS})

target_link_libraries(NeoVMCodeGen PRIVATE ${NeoVM_LINK_LIBS})
