// NeoVM instruction set definitions

class NeoVMInstBase<dag outs, dag ins, string Asm>
    : NeoVMInst<outs, ins, Asm, []> {
  let isPseudo = 1;
  let hasSideEffects = 0;
  let hasNoSchedulingInfo = 1;
}

// Core stack manipulation with explicit operands used by lowering code.
def PUSH            : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$src), "PUSH $src">;
def PUSHINT8        : NeoVMInstBase<(outs GR32:$dst), (ins i32imm:$imm), "PUSHINT8 $imm">;
def PUSHINT16       : NeoVMInstBase<(outs GR32:$dst), (ins i32imm:$imm), "PUSHINT16 $imm">;
def PUSHINT32       : NeoVMInstBase<(outs GR32:$dst), (ins i32imm:$imm), "PUSHINT32 $imm">;
def PUSHINT64       : NeoVMInstBase<(outs GR32:$dst), (ins i64imm:$imm), "PUSHINT64 $imm">;

def PUSHM1          : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSHM1">;
def PUSH0           : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH0">;
def PUSH1           : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH1">;
def PUSH2           : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH2">;
def PUSH3           : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH3">;
def PUSH4           : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH4">;
def PUSH5           : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH5">;
def PUSH6           : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH6">;
def PUSH7           : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH7">;
def PUSH8           : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH8">;
def PUSH9           : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH9">;
def PUSH10          : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH10">;
def PUSH11          : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH11">;
def PUSH12          : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH12">;
def PUSH13          : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH13">;
def PUSH14          : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH14">;
def PUSH15          : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH15">;
def PUSH16          : NeoVMInstBase<(outs GR32:$dst), (ins), "PUSH16">;

def POP             : NeoVMInstBase<(outs), (ins GR32:$val, GR32:$addr), "POP">;
def DUP             : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$src), "DUP">;
def SWAP            : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$a, GR32:$b), "SWAP">;
def OVER            : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$src), "OVER">;
def ROT             : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$a, GR32:$b, GR32:$c), "ROT">;
def ROLL            : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$depth, GR32:$count), "ROLL">;
def REVERSE3        : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$a, GR32:$b, GR32:$c), "REVERSE3">;
def REVERSE4        : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$a, GR32:$b, GR32:$c, GR32:$d), "REVERSE4">;
def REVERSEN        : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$n), "REVERSEN">;
def DROP            : NeoVMInstBase<(outs), (ins GR32:$val), "DROP">;
def DROPN           : NeoVMInstBase<(outs), (ins GR32:$n), "DROPN">;
def CLEAR           : NeoVMInstBase<(outs), (ins), "CLEAR">;
def TOALTSTACK      : NeoVMInstBase<(outs), (ins GR32:$val), "TOALTSTACK">;
def FROMALTSTACK    : NeoVMInstBase<(outs GR32:$dst), (ins), "FROMALTSTACK">;
def DUPFROMALTSTACK : NeoVMInstBase<(outs GR32:$dst), (ins), "DUPFROMALTSTACK">;

def ADD             : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$lhs, GR32:$rhs), "ADD">;
def SUB             : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$lhs, GR32:$rhs), "SUB">;
def MUL             : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$lhs, GR32:$rhs), "MUL">;
def DIV             : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$lhs, GR32:$rhs), "DIV">;
def EQ              : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$lhs, GR32:$rhs), "EQ">;
def NE              : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$lhs, GR32:$rhs), "NE">;
def LT              : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$lhs, GR32:$rhs), "LT">;
def LE              : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$lhs, GR32:$rhs), "LE">;
def GT              : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$lhs, GR32:$rhs), "GT">;
def GE              : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$lhs, GR32:$rhs), "GE">;
def ASSERT          : NeoVMInstBase<(outs), (ins GR32:$cond), "ASSERT">;

def JMP             : NeoVMInstBase<(outs), (ins i32imm:$dst), "JMP $dst">;
def JMPIF           : NeoVMInstBase<(outs), (ins GR32:$cond, i32imm:$dst), "JMPIF $dst">;
def JMPIFNOT        : NeoVMInstBase<(outs), (ins GR32:$cond, i32imm:$dst), "JMPIFNOT $dst">;
def CALL            : NeoVMInstBase<(outs), (ins i32imm:$dst), "CALL $dst">;
def RET             : NeoVMInstBase<(outs), (ins), "RET">;

def SYSCALL         : NeoVMInstBase<(outs), (ins i32imm:$hash), "SYSCALL $hash">;

def NEOVM_LOAD_PSEUDO  : NeoVMInstBase<(outs GR32:$dst), (ins GR32:$src), "NEOVM_LOAD_PSEUDO">;
def NEOVM_STORE_PSEUDO : NeoVMInstBase<(outs), (ins GR32:$val, GR32:$addr), "NEOVM_STORE_PSEUDO">;

include "NeoVMInstrAuto.td"
