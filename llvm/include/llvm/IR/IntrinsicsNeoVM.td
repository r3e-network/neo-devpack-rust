// NeoVM intrinsic definitions

let TargetPrefix = "neovm" in {

def int_neovm_syscall : Intrinsic<[llvm_i32_ty], [llvm_i32_ty], [IntrNoMem]>;

} // TargetPrefix = "neovm"
