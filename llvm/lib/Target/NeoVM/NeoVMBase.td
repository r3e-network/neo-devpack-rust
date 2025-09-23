// NeoVM base definitions shared across TableGen files.

include "llvm/Target/Target.td"

class NeoVMReg<string Name> : Register<Name> {
  let Namespace = "NeoVM";
}

class NeoVMInst<dag outs, dag ins, string asmstr, list<dag> pattern = []>
    : Instruction {
  let Namespace = "NeoVM";
  let OutOperandList = outs;
  let InOperandList = ins;
  let AsmString = asmstr;
  let Pattern = pattern;
}
