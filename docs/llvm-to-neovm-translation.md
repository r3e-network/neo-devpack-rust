# LLVM to NeoVM Translation: Register-Based to Stack-Based

## Overview

This document explains how the Neo LLVM project translates from LLVM's register-based intermediate representation (IR) to NeoVM's stack-based execution model. This is one of the most critical and complex aspects of the project.

## Translation Flow Diagram

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Rust Code     │    │   LLVM IR        │    │   NeoVM         │
│                 │    │   (Register-     │    │   (Stack-       │
│ fn add(a, b) {  │───▶│    Based)        │───▶│    Based)       │
│   return a + b  │    │                  │    │                 │
│ }               │    │ %1 = add %a, %b  │    │ PUSH a          │
└─────────────────┘    │ ret %1           │    │ PUSH b          │
                       │                  │    │ ADD             │
                       └──────────────────┘    │ RET             │
                                               └─────────────────┘
```

### Translation Process Steps:

1. **Rust → LLVM IR**: Rust compiler generates register-based LLVM IR
2. **LLVM IR → Stackification**: Our custom pass converts to stack operations
3. **Stack Operations → NeoVM**: Generate NeoVM bytecode with stack instructions

## The Challenge

### LLVM IR (Register-Based)
- **SSA Form**: Static Single Assignment - each value is assigned exactly once
- **Virtual Registers**: Values are stored in virtual registers
- **Explicit Dependencies**: Data flow is explicit through register dependencies
- **Example**:
```llvm
%1 = add i32 %a, %b
%2 = mul i32 %1, %c
%3 = sub i32 %2, %d
```

### NeoVM (Stack-Based)
- **Stack Operations**: All operations work on a stack
- **Implicit Dependencies**: Data flow is implicit through stack position
- **Push/Pop Model**: Values are pushed onto stack, operations consume from stack
- **Example**:
```neovm
PUSH a
PUSH b
ADD
PUSH c
MUL
PUSH d
SUB
```

## Translation Strategy

### 1. Stackification Pass

The core translation happens in the **Stackification Pass** (`NeoVMStackify.cpp`), which transforms SSA form into explicit stack operations.

#### Key Components:

**A. Stack Height Tracking**
```cpp
class NeoVMStackHeightTracker {
    std::map<const MachineBasicBlock*, int> StackHeights;
    
    int getStackHeight(const MachineBasicBlock* MBB) {
        // Calculate current stack height for each basic block
    }
    
    void updateStackHeight(const MachineBasicBlock* MBB, int delta) {
        // Update stack height after operations
    }
};
```

**B. Value to Stack Position Mapping**
```cpp
class NeoVMValueMapper {
    std::map<const Value*, int> ValueToStackPos;
    
    int getStackPosition(const Value* V) {
        // Map LLVM values to their stack positions
    }
    
    void updateStackPosition(const Value* V, int pos) {
        // Update stack position after operations
    }
};
```

### 2. Translation Process

#### Step 1: Analyze LLVM IR
```cpp
void NeoVMStackifyPass::analyzeIR(MachineFunction &MF) {
    for (auto &MBB : MF) {
        for (auto &MI : MBB) {
            // Analyze each instruction
            analyzeInstruction(MI);
        }
    }
}
```

#### Step 2: Calculate Stack Operations
```cpp
void NeoVMStackifyPass::calculateStackOps(MachineInstr &MI) {
    switch (MI.getOpcode()) {
        case TargetOpcode::G_ADD:
            // Convert: %result = add %a, %b
            // To: PUSH %a, PUSH %b, ADD
            emitPush(MI.getOperand(1)); // Push %a
            emitPush(MI.getOperand(2)); // Push %b
            emitAdd();                  // ADD operation
            break;
            
        case TargetOpcode::G_MUL:
            // Convert: %result = mul %a, %b
            // To: PUSH %a, PUSH %b, MUL
            emitPush(MI.getOperand(1)); // Push %a
            emitPush(MI.getOperand(2)); // Push %b
            emitMul();                  // MUL operation
            break;
    }
}
```

#### Step 3: Emit NeoVM Instructions
```cpp
void NeoVMStackifyPass::emitPush(const MachineOperand &Op) {
    if (Op.isImm()) {
        // Immediate value
        if (Op.getImm() >= 0 && Op.getImm() <= 16) {
            emitInstruction(NEOVM_PUSH0 + Op.getImm());
        } else {
            emitInstruction(NEOVM_PUSHINT8);
            emitByte(Op.getImm());
        }
    } else if (Op.isReg()) {
        // Register value - need to load from stack
        int stackPos = getStackPosition(Op.getReg());
        emitLoadFromStack(stackPos);
    }
}
```

### 3. Stack Management

#### A. Stack Spilling
When the stack becomes too deep, we need to spill values:

```cpp
void NeoVMStackifyPass::handleStackSpill() {
    if (getCurrentStackHeight() > MAX_STACK_DEPTH) {
        // Spill excess values to storage
        int spillCount = getCurrentStackHeight() - MAX_STACK_DEPTH;
        for (int i = 0; i < spillCount; i++) {
            emitInstruction(NEOVM_STLOC, i); // Store to local slot
        }
    }
}
```

#### B. Stack Restoration
```cpp
void NeoVMStackifyPass::restoreFromSpill() {
    // Restore spilled values when needed
    for (int i = 0; i < spillCount; i++) {
        emitInstruction(NEOVM_LDLOC, i); // Load from local slot
    }
}
```

### 4. Control Flow Translation

#### A. Basic Blocks
```cpp
void NeoVMStackifyPass::translateBasicBlock(MachineBasicBlock &MBB) {
    // Ensure consistent stack height at block entry
    int targetHeight = getTargetStackHeight(&MBB);
    int currentHeight = getCurrentStackHeight();
    
    if (currentHeight < targetHeight) {
        // Need to push dummy values
        for (int i = currentHeight; i < targetHeight; i++) {
            emitInstruction(NEOVM_PUSH0);
        }
    } else if (currentHeight > targetHeight) {
        // Need to pop excess values
        for (int i = currentHeight; i > targetHeight; i--) {
            emitInstruction(NEOVM_DROP);
        }
    }
}
```

#### B. Branches and Jumps
```cpp
void NeoVMStackifyPass::translateBranch(MachineInstr &MI) {
    // Convert LLVM branch to NeoVM jump
    if (MI.isConditionalBranch()) {
        // Conditional branch: JMPIF or JMPIFNOT
        emitInstruction(NEOVM_JMPIF, getTargetAddress(MI));
    } else {
        // Unconditional branch: JMP
        emitInstruction(NEOVM_JMP, getTargetAddress(MI));
    }
}
```

### 5. Function Call Translation

#### A. Parameter Passing
```cpp
void NeoVMStackifyPass::translateCall(MachineInstr &MI) {
    // Push arguments in reverse order (NeoVM convention)
    for (int i = MI.getNumOperands() - 1; i >= 1; i--) {
        emitPush(MI.getOperand(i));
    }
    
    // Emit call instruction
    emitInstruction(NEOVM_CALL, getFunctionAddress(MI));
    
    // Handle return value
    if (MI.getNumDefs() > 0) {
        // Return value is on top of stack
        mapValueToStack(MI.getOperand(0), getCurrentStackHeight() - 1);
    }
}
```

#### B. Return Value Handling
```cpp
void NeoVMStackifyPass::translateReturn(MachineInstr &MI) {
    if (MI.getNumOperands() > 0) {
        // Push return value
        emitPush(MI.getOperand(0));
    }
    
    // Emit return instruction
    emitInstruction(NEOVM_RET);
}
```

### 6. Memory Operations

#### A. Load Operations
```cpp
void NeoVMStackifyPass::translateLoad(MachineInstr &MI) {
    // Convert: %result = load %ptr
    // To: PUSH %ptr, LOAD
    emitPush(MI.getOperand(1)); // Push address
    emitInstruction(NEOVM_LOAD);
}
```

#### B. Store Operations
```cpp
void NeoVMStackifyPass::translateStore(MachineInstr &MI) {
    // Convert: store %value, %ptr
    // To: PUSH %value, PUSH %ptr, STORE
    emitPush(MI.getOperand(0)); // Push value
    emitPush(MI.getOperand(1)); // Push address
    emitInstruction(NEOVM_STORE);
}
```

### 7. Optimization Strategies

#### A. Stack Height Optimization
```cpp
void NeoVMStackifyPass::optimizeStackHeight() {
    // Minimize stack operations by reordering instructions
    // when possible without changing semantics
}
```

#### B. Dead Code Elimination
```cpp
void NeoVMStackifyPass::eliminateDeadCode() {
    // Remove instructions that don't affect final result
    // This reduces stack operations
}
```

#### C. Constant Folding
```cpp
void NeoVMStackifyPass::foldConstants() {
    // Evaluate constant expressions at compile time
    // Reduces runtime stack operations
}
```

### 8. Example Translation

#### Original Rust Code:
```rust
fn calculate(a: i32, b: i32) -> i32 {
    let sum = a + b;
    let doubled = sum * 2;
    let result = doubled - 1;
    result
}
```

#### LLVM IR (Register-Based):
```llvm
define i32 @calculate(i32 %a, i32 %b) {
entry:
  %1 = add nsw i32 %a, %b
  %2 = mul nsw i32 %1, 2
  %3 = sub nsw i32 %2, 1
  ret i32 %3
}
```

#### Stackification Process:
```cpp
// Step 1: Analyze the LLVM IR
void analyzeInstruction(MachineInstr &MI) {
    switch (MI.getOpcode()) {
        case TargetOpcode::G_ADD:
            // %1 = add %a, %b
            // Stack operations: PUSH %a, PUSH %b, ADD
            emitPush(MI.getOperand(1)); // Push %a
            emitPush(MI.getOperand(2)); // Push %b
            emitInstruction(NEOVM_ADD);
            break;
            
        case TargetOpcode::G_MUL:
            // %2 = mul %1, 2
            // Stack operations: PUSH %1, PUSH 2, MUL
            emitPush(MI.getOperand(1)); // Push %1 (already on stack)
            emitPushImmediate(2);       // Push constant 2
            emitInstruction(NEOVM_MUL);
            break;
            
        case TargetOpcode::G_SUB:
            // %3 = sub %2, 1
            // Stack operations: PUSH %2, PUSH 1, SUB
            emitPush(MI.getOperand(1)); // Push %2 (already on stack)
            emitPushImmediate(1);       // Push constant 1
            emitInstruction(NEOVM_SUB);
            break;
    }
}
```

#### Final NeoVM Bytecode:
```neovm
; Function: calculate(a: i32, b: i32) -> i32
; Parameters are already on stack: [a, b]

; %1 = add %a, %b
; Stack: [a, b] -> [a, b] -> [sum]
ADD

; %2 = mul %1, 2
; Stack: [sum] -> [sum, 2] -> [doubled]
PUSHINT8 2
MUL

; %3 = sub %2, 1
; Stack: [doubled] -> [doubled, 1] -> [result]
PUSHINT8 1
SUB

; Return result
; Stack: [result] -> []
RET
```

#### Stack State Visualization:
```
Initial Stack: [a, b]
After ADD:     [sum]
After PUSH 2:  [sum, 2]
After MUL:     [doubled]
After PUSH 1:  [doubled, 1]
After SUB:     [result]
After RET:     []
```

### 9. Challenges and Solutions

#### A. Stack Depth Management
- **Challenge**: NeoVM has limited stack depth
- **Solution**: Implement stack spilling to local storage

#### B. Complex Control Flow
- **Challenge**: LLVM IR can have complex control flow
- **Solution**: Maintain stack height consistency across all paths

#### C. Function Calls
- **Challenge**: Parameter passing and return value handling
- **Solution**: Use NeoVM's call stack and parameter conventions

#### D. Memory Operations
- **Challenge**: LLVM's memory model vs NeoVM's storage model
- **Solution**: Map LLVM memory operations to NeoVM storage operations

### 10. Implementation Files

The translation is implemented in several key files:

#### A. Core Translation Files:
- **`NeoVMStackify.cpp`**: Main stackification pass that converts LLVM IR to stack operations
- **`NeoVMStackHeightVerifier.cpp`**: Verifies stack height consistency across all code paths
- **`NeoVMInstrInfo.cpp`**: Instruction information and pseudo-instruction expansion
- **`NeoVMRegisterInfo.cpp`**: Register to stack position mapping
- **`NeoVMFrameLowering.cpp`**: Frame management and stack operations

#### B. Key Implementation Details:

**NeoVMStackify.cpp - Main Translation Logic:**
```cpp
bool NeoVMStackifyPass::runOnMachineFunction(MachineFunction &MF) {
    // 1. Analyze the function's control flow
    analyzeControlFlow(MF);
    
    // 2. Calculate stack heights for each basic block
    calculateStackHeights(MF);
    
    // 3. Convert each instruction to stack operations
    for (auto &MBB : MF) {
        for (auto &MI : MBB) {
            convertToStackOps(MI);
        }
    }
    
    // 4. Verify stack height consistency
    verifyStackHeights(MF);
    
    return true;
}
```

**NeoVMStackHeightVerifier.cpp - Stack Consistency:**
```cpp
bool NeoVMStackHeightVerifier::verifyStackHeight(MachineFunction &MF) {
    for (auto &MBB : MF) {
        int entryHeight = getStackHeightAtEntry(MBB);
        int exitHeight = getStackHeightAtExit(MBB);
        
        // All paths to this block must have same stack height
        if (!isStackHeightConsistent(MBB, entryHeight)) {
            reportError("Inconsistent stack height at block entry");
            return false;
        }
        
        // All paths from this block must have same stack height
        if (!isStackHeightConsistent(MBB, exitHeight)) {
            reportError("Inconsistent stack height at block exit");
            return false;
        }
    }
    return true;
}
```

**NeoVMInstrInfo.cpp - Instruction Expansion:**
```cpp
void NeoVMInstrInfo::expandPostRAPseudo(MachineInstr &MI) const {
    switch (MI.getOpcode()) {
        case NEOVM_LOAD_PSEUDO:
            // Convert pseudo load to actual stack operations
            expandLoadPseudo(MI);
            break;
            
        case NEOVM_STORE_PSEUDO:
            // Convert pseudo store to actual stack operations
            expandStorePseudo(MI);
            break;
    }
}
```

**NeoVMRegisterInfo.cpp - Register Mapping:**
```cpp
void NeoVMRegisterInfo::eliminateFrameIndex(MachineInstr &MI, int SPAdj,
                                           unsigned FIOperandNum,
                                           RegScavenger *RS) const {
    // Convert frame index references to stack operations
    MachineOperand &FI = MI.getOperand(FIOperandNum);
    int FrameIndex = FI.getIndex();
    
    // Calculate stack offset
    int StackOffset = getFrameIndexOffset(MI.getParent()->getParent(), FrameIndex);
    
    // Emit stack operations to access the value
    if (StackOffset >= 0 && StackOffset <= 16) {
        // Use PUSH0-PUSH16 for small offsets
        MI.setDesc(get(NEOVM_PUSH0 + StackOffset));
    } else {
        // Use PUSHINT8 for larger offsets
        MI.setDesc(get(NEOVM_PUSHINT8));
        MI.getOperand(FIOperandNum).ChangeToImmediate(StackOffset);
    }
}
```

### 11. Testing and Verification

#### A. Stack Height Verification
```cpp
bool NeoVMStackHeightVerifier::verifyStackHeight(MachineFunction &MF) {
    for (auto &MBB : MF) {
        int height = calculateStackHeight(MBB);
        if (height < 0 || height > MAX_STACK_DEPTH) {
            return false;
        }
    }
    return true;
}
```

#### B. Functional Equivalence Testing
```cpp
void testTranslationEquivalence() {
    // Test that translated code produces same results
    // as original LLVM IR
}
```

### 12. Performance Considerations

#### A. Stack Operation Optimization
```cpp
void NeoVMStackifyPass::optimizeStackOperations() {
    // 1. Combine consecutive PUSH operations
    combinePushOperations();
    
    // 2. Eliminate redundant stack operations
    eliminateRedundantOps();
    
    // 3. Optimize stack depth usage
    optimizeStackDepth();
}
```

#### B. Memory Access Patterns
- **Local Variables**: Mapped to stack positions for fast access
- **Global Variables**: Mapped to NeoVM storage for persistence
- **Temporary Values**: Kept on stack for maximum performance

#### C. Function Call Overhead
- **Parameter Passing**: Optimized to minimize stack operations
- **Return Values**: Efficiently handled through stack conventions
- **Call Stack**: Managed by NeoVM's built-in call stack

### 13. Benefits of This Approach

#### A. Developer Benefits
- **Familiar Syntax**: Write code in Rust, C++, or other LLVM-supported languages
- **Rich Tooling**: Use existing LLVM tools for debugging and optimization
- **Type Safety**: Leverage LLVM's type system for compile-time checks
- **Optimization**: Benefit from LLVM's extensive optimization passes

#### B. Runtime Benefits
- **Efficient Execution**: Stack-based operations are naturally efficient
- **Memory Management**: NeoVM handles memory management automatically
- **Gas Optimization**: Optimized bytecode reduces gas consumption
- **Compatibility**: Full compatibility with Neo N3 ecosystem

#### C. Maintenance Benefits
- **Standard IR**: LLVM IR is a well-established standard
- **Tool Support**: Extensive tooling and debugging support
- **Documentation**: Rich documentation and community support
- **Future-Proof**: LLVM continues to evolve and improve

### 14. Comparison with Other Approaches

#### A. Direct Compilation
- **Pros**: Potentially faster compilation
- **Cons**: Requires custom compiler for each language
- **Our Approach**: Leverages existing LLVM infrastructure

#### B. Source-to-Source Translation
- **Pros**: Simpler translation process
- **Cons**: Limited optimization opportunities
- **Our Approach**: Full LLVM optimization pipeline

#### C. Bytecode Interpretation
- **Pros**: Simple implementation
- **Cons**: Runtime performance overhead
- **Our Approach**: Native NeoVM bytecode execution

## Conclusion

The translation from LLVM's register-based IR to NeoVM's stack-based execution model is a complex but well-defined process. The key is maintaining stack height consistency while preserving the semantic meaning of the original code. The stackification pass handles this translation automatically, allowing developers to write high-level code that gets efficiently translated to NeoVM bytecode.

This approach provides the best of both worlds:
- **Developer Experience**: Write code in familiar high-level languages
- **NeoVM Compatibility**: Generate efficient stack-based bytecode
- **Performance**: Optimized translation with minimal overhead
- **Maintainability**: Leverages proven LLVM infrastructure
- **Future-Proof**: Benefits from ongoing LLVM improvements

The Neo LLVM project successfully bridges the gap between modern high-level programming languages and the NeoVM execution environment, making Neo N3 smart contract development more accessible and efficient.
