// NeoVM Target Tests
// Comprehensive unit tests for NeoVM LLVM backend

#include "NeoVM.h"
#include "NeoVMTargetMachine.h"
#include "NeoVMInstrInfo.h"
#include "NeoVMRegisterInfo.h"
#include "NeoVMSubtarget.h"
#include "NeoVMFrameLowering.h"
#include "NeoVMAsmPrinter.h"
#include "NeoVMNEF.h"
#include "NeoVMSyscalls.h"

#include "llvm/ADT/Triple.h"
#include "llvm/CodeGen/MachineFunction.h"
#include "llvm/CodeGen/MachineInstr.h"
#include "llvm/CodeGen/MachineModuleInfo.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/BasicBlock.h"
#include "llvm/IR/Instructions.h"
#include "llvm/Support/TargetRegistry.h"
#include "llvm/Support/TargetSelect.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/Support/MemoryBuffer.h"
#include "llvm/Support/Error.h"

#include <gtest/gtest.h>
#include <memory>
#include <vector>

using namespace llvm;

class NeoVMTargetTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Initialize LLVM
        LLVMInitializeNeoVMTargetInfo();
        LLVMInitializeNeoVMTarget();
        LLVMInitializeNeoVMTargetMC();
        LLVMInitializeNeoVMAsmPrinter();
        LLVMInitializeNeoVMAsmParser();
        LLVMInitializeNeoVMDisassembler();
        
        // Create target machine
        std::string Error;
        const Target *Target = TargetRegistry::lookupTarget("neovm-unknown-neo3", Triple("neovm-unknown-neo3"), Error);
        ASSERT_NE(Target, nullptr) << "Failed to create NeoVM target: " << Error;
        
        TargetMachine = Target->createTargetMachine("neovm-unknown-neo3", "neovm", "", TargetOptions(), std::nullopt);
        ASSERT_NE(TargetMachine, nullptr) << "Failed to create NeoVM target machine";
        
        // Create context and module
        Context = std::make_unique<LLVMContext>();
        Module = std::make_unique<Module>("test", *Context);
        Module->setTargetTriple("neovm-unknown-neo3");
        Module->setDataLayout(TargetMachine->createDataLayout());
    }
    
    void TearDown() override {
        TargetMachine.reset();
        Module.reset();
        Context.reset();
    }
    
    std::unique_ptr<LLVMContext> Context;
    std::unique_ptr<Module> Module;
    std::unique_ptr<TargetMachine> TargetMachine;
};

// Test target registration
TEST_F(NeoVMTargetTest, TargetRegistration) {
    std::string Error;
    const Target *Target = TargetRegistry::lookupTarget("neovm-unknown-neo3", Triple("neovm-unknown-neo3"), Error);
    ASSERT_NE(Target, nullptr) << "NeoVM target not registered: " << Error;
    EXPECT_EQ(Target->getName(), "NeoVM");
}

// Test target machine creation
TEST_F(NeoVMTargetTest, TargetMachineCreation) {
    ASSERT_NE(TargetMachine, nullptr);
    EXPECT_EQ(TargetMachine->getTargetTriple().str(), "neovm-unknown-neo3");
    EXPECT_EQ(TargetMachine->getTargetCPU().str(), "neovm");
}

// Test instruction info
TEST_F(NeoVMTargetTest, InstructionInfo) {
    const NeoVMInstrInfo *InstrInfo = static_cast<const NeoVMInstrInfo*>(TargetMachine->getInstrInfo());
    ASSERT_NE(InstrInfo, nullptr);
    
    // Test basic instruction properties
    EXPECT_GT(InstrInfo->getNumOpcodes(), 0);
    EXPECT_TRUE(InstrInfo->isPseudo(NeoVM::NEOVM_LOAD_PSEUDO));
    EXPECT_FALSE(InstrInfo->isPseudo(NeoVM::PUSH0));
}

// Test register info
TEST_F(NeoVMTargetTest, RegisterInfo) {
    const NeoVMRegisterInfo *RegInfo = static_cast<const NeoVMRegisterInfo*>(TargetMachine->getRegisterInfo());
    ASSERT_NE(RegInfo, nullptr);
    
    // Test register properties
    EXPECT_GT(RegInfo->getNumRegs(), 0);
    EXPECT_EQ(RegInfo->getFrameRegister(), Register());
    EXPECT_TRUE(RegInfo->getReservedRegs(*Module->getFunction("test")).test(NeoVM::STACK));
}

// Test subtarget
TEST_F(NeoVMTargetTest, Subtarget) {
    const NeoVMSubtarget *Subtarget = static_cast<const NeoVMSubtarget*>(TargetMachine->getSubtargetImpl());
    ASSERT_NE(Subtarget, nullptr);
    
    // Test subtarget properties
    EXPECT_TRUE(Subtarget->hasFeature(NeoVM::FeatureNeoVM));
    EXPECT_FALSE(Subtarget->hasFeature(NeoVM::FeatureTryCatch));
}

// Test frame lowering
TEST_F(NeoVMTargetTest, FrameLowering) {
    const NeoVMFrameLowering *FrameLowering = static_cast<const NeoVMFrameLowering*>(TargetMachine->getFrameLowering());
    ASSERT_NE(FrameLowering, nullptr);
    
    // Test frame lowering properties
    EXPECT_TRUE(FrameLowering->hasReservedCallFrame(*Module->getFunction("test")));
    EXPECT_FALSE(FrameLowering->hasFP(*Module->getFunction("test")));
}

// Test assembly printer
TEST_F(NeoVMTargetTest, AssemblyPrinter) {
    const NeoVMAsmPrinter *AsmPrinter = static_cast<const NeoVMAsmPrinter*>(TargetMachine->getAsmPrinter());
    ASSERT_NE(AsmPrinter, nullptr);
    
    // Test assembly printer properties
    EXPECT_TRUE(AsmPrinter->isNeoVM());
}

// Test NEF container
TEST_F(NeoVMTargetTest, NEFContainer) {
    NEFContainer container;
    
    // Test basic NEF operations
    container.script = {0x00, 0x01, 0x02, 0x03};
    container.manifest = "{\"name\":\"TestContract\"}";
    
    EXPECT_FALSE(container.script.empty());
    EXPECT_FALSE(container.manifest.empty());
    
    // Test serialization
    auto serialized = container.serialize();
    EXPECT_FALSE(serialized.empty());
    
    // Test deserialization
    auto deserialized = NEFContainer::deserialize(serialized);
    ASSERT_TRUE(deserialized.has_value());
    EXPECT_EQ(deserialized->script, container.script);
    EXPECT_EQ(deserialized->manifest, container.manifest);
}

// Test NEF manifest generator
TEST_F(NeoVMTargetTest, NEFManifestGenerator) {
    NEFManifestGenerator generator;
    generator.contractName = "TestContract";
    generator.version = "1.0.0";
    generator.author = "test";
    generator.description = "Test contract";
    
    generator.addMethod("test_method", {"Integer"}, "Boolean");
    generator.addEvent("TestEvent", {"String"});
    
    auto manifest = generator.generate();
    EXPECT_FALSE(manifest.empty());
    EXPECT_TRUE(manifest.find("TestContract") != std::string::npos);
    EXPECT_TRUE(manifest.find("test_method") != std::string::npos);
    EXPECT_TRUE(manifest.find("TestEvent") != std::string::npos);
}

// Test syscall registry
TEST_F(NeoVMTargetTest, SyscallRegistry) {
    NeoVMSyscallRegistry registry;
    
    // Test syscall loading
    EXPECT_TRUE(registry.loadFromFile("neo_syscalls.json"));
    EXPECT_GT(registry.getAllSyscallNames().size(), 0);
    
    // Test syscall lookup
    auto getTime = registry.getSyscall("System.Runtime.GetTime");
    ASSERT_NE(getTime, nullptr);
    EXPECT_EQ(getTime->name, "System.Runtime.GetTime");
    EXPECT_EQ(getTime->hash, 0x68b4c4c1);
    
    // Test syscall lowering
    auto module = std::make_unique<Module>("test", *Context);
    auto function = Function::Create(
        FunctionType::get(Type::getInt32Ty(*Context), {}, false),
        GlobalValue::ExternalLinkage,
        "test_function",
        module.get()
    );
    
    auto lowered = NeoVMSyscallLowering::lowerToSyscall(function, getTime);
    EXPECT_TRUE(lowered);
}

// Test complete compilation pipeline
TEST_F(NeoVMTargetTest, CompilationPipeline) {
    // Create a simple function
    auto function = Function::Create(
        FunctionType::get(Type::getInt32Ty(*Context), {}, false),
        GlobalValue::ExternalLinkage,
        "test_function",
        Module.get()
    );
    
    auto entry = BasicBlock::Create(*Context, "entry", function);
    auto ret = ReturnInst::Create(*Context, ConstantInt::get(Type::getInt32Ty(*Context), 42), entry);
    
    // Test that the function can be created
    EXPECT_NE(function, nullptr);
    EXPECT_NE(entry, nullptr);
    EXPECT_NE(ret, nullptr);
    
    // Test that the module is valid
    EXPECT_FALSE(verifyModule(*Module, &errs()));
}

// Test instruction encoding
TEST_F(NeoVMTargetTest, InstructionEncoding) {
    // Test PUSH0 instruction
    auto encoding = NeoVMEncoding::getOpcodeEncoding(NeoVM::PUSH0);
    ASSERT_TRUE(encoding.has_value());
    EXPECT_EQ(encoding->Byte, 0x10);
    EXPECT_EQ(encoding->OperandSize, 0);
    
    // Test PUSH1 instruction
    auto encoding1 = NeoVMEncoding::getOpcodeEncoding(NeoVM::PUSH1);
    ASSERT_TRUE(encoding1.has_value());
    EXPECT_EQ(encoding1->Byte, 0x11);
    EXPECT_EQ(encoding1->OperandSize, 0);
    
    // Test ADD instruction
    auto addEncoding = NeoVMEncoding::getOpcodeEncoding(NeoVM::ADD);
    ASSERT_TRUE(addEncoding.has_value());
    EXPECT_EQ(addEncoding->Byte, 0x9E);
    EXPECT_EQ(addEncoding->OperandSize, 0);
}

// Test stack operations
TEST_F(NeoVMTargetTest, StackOperations) {
    const NeoVMInstrInfo *InstrInfo = static_cast<const NeoVMInstrInfo*>(TargetMachine->getInstrInfo());
    
    // Test stack operation detection
    EXPECT_TRUE(InstrInfo->isPureStackOp(NeoVM::PUSH0));
    EXPECT_TRUE(InstrInfo->isPureStackOp(NeoVM::PUSH1));
    EXPECT_TRUE(InstrInfo->isPureStackOp(NeoVM::ADD));
    EXPECT_TRUE(InstrInfo->isPureStackOp(NeoVM::SUB));
    EXPECT_TRUE(InstrInfo->isPureStackOp(NeoVM::DUP));
    EXPECT_TRUE(InstrInfo->isPureStackOp(NeoVM::SWAP));
    EXPECT_TRUE(InstrInfo->isPureStackOp(NeoVM::DROP));
    
    // Test non-stack operations
    EXPECT_FALSE(InstrInfo->isPureStackOp(NeoVM::JMP));
    EXPECT_FALSE(InstrInfo->isPureStackOp(NeoVM::CALL));
}

// Test syscall integration
TEST_F(NeoVMTargetTest, SyscallIntegration) {
    NeoVMSyscallRegistry registry;
    registry.loadFromFile("neo_syscalls.json");
    
    // Test syscall hash lookup
    auto getTimeHash = registry.getSyscallHash("System.Runtime.GetTime");
    ASSERT_TRUE(getTimeHash.has_value());
    EXPECT_EQ(getTimeHash.value(), 0x68b4c4c1);
    
    // Test syscall lowering
    auto module = std::make_unique<Module>("test", *Context);
    auto function = Function::Create(
        FunctionType::get(Type::getInt32Ty(*Context), {}, false),
        GlobalValue::ExternalLinkage,
        "System.Runtime.GetTime",
        module.get()
    );
    
    auto lowered = registry.lowerToSyscall(function);
    EXPECT_TRUE(lowered);
}

// Test NEF file generation
TEST_F(NeoVMTargetTest, NEFFileGeneration) {
    NEFContainer container;
    container.script = {0x00, 0x01, 0x02, 0x03, 0x04, 0x05};
    container.manifest = "{\"name\":\"TestContract\",\"version\":\"1.0.0\"}";
    
    // Test serialization
    auto serialized = container.serialize();
    EXPECT_FALSE(serialized.empty());
    EXPECT_GT(serialized.size(), 0);
    
    // Test deserialization
    auto deserialized = NEFContainer::deserialize(serialized);
    ASSERT_TRUE(deserialized.has_value());
    EXPECT_EQ(deserialized->script, container.script);
    EXPECT_EQ(deserialized->manifest, container.manifest);
    
    // Test validation
    EXPECT_TRUE(deserialized->isValid());
}

// Test error handling
TEST_F(NeoVMTargetTest, ErrorHandling) {
    // Test invalid NEF deserialization
    std::vector<uint8_t> invalidData = {0x00, 0x01, 0x02};
    auto result = NEFContainer::deserialize(invalidData);
    EXPECT_FALSE(result.has_value());
    
    // Test empty NEF container
    NEFContainer emptyContainer;
    EXPECT_FALSE(emptyContainer.isValid());
}

// Test performance
TEST_F(NeoVMTargetTest, Performance) {
    const int iterations = 1000;
    
    // Test NEF serialization performance
    NEFContainer container;
    container.script.resize(1024, 0x42);
    container.manifest = "{\"name\":\"PerformanceTest\"}";
    
    auto start = std::chrono::high_resolution_clock::now();
    for (int i = 0; i < iterations; ++i) {
        auto serialized = container.serialize();
        auto deserialized = NEFContainer::deserialize(serialized);
        EXPECT_TRUE(deserialized.has_value());
    }
    auto end = std::chrono::high_resolution_clock::now();
    
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    EXPECT_LT(duration.count(), 1000); // Should complete in less than 1 second
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
