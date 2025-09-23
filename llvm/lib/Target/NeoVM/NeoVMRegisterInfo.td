// NeoVM register definitions (NeoVMBase.td already included)

def STACK : NeoVMReg<"STACK"> {
  let HWEncoding = 0;
}

def GR32 : RegisterClass<"NeoVM", [i32], 32, (add STACK)>;
