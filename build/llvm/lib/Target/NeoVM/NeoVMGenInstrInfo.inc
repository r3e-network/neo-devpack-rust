/*===- TableGen'erated file -------------------------------------*- C++ -*-===*\
|*                                                                            *|
|* Target Instruction Enum Values and Descriptors                             *|
|*                                                                            *|
|* Automatically generated file, do not edit!                                 *|
|*                                                                            *|
\*===----------------------------------------------------------------------===*/

#ifdef GET_INSTRINFO_ENUM
#undef GET_INSTRINFO_ENUM
namespace llvm {

namespace NeoVM {
  enum {
    PHI	= 0,
    INLINEASM	= 1,
    INLINEASM_BR	= 2,
    CFI_INSTRUCTION	= 3,
    EH_LABEL	= 4,
    GC_LABEL	= 5,
    ANNOTATION_LABEL	= 6,
    KILL	= 7,
    EXTRACT_SUBREG	= 8,
    INSERT_SUBREG	= 9,
    IMPLICIT_DEF	= 10,
    SUBREG_TO_REG	= 11,
    COPY_TO_REGCLASS	= 12,
    DBG_VALUE	= 13,
    DBG_VALUE_LIST	= 14,
    DBG_INSTR_REF	= 15,
    DBG_PHI	= 16,
    DBG_LABEL	= 17,
    REG_SEQUENCE	= 18,
    COPY	= 19,
    BUNDLE	= 20,
    LIFETIME_START	= 21,
    LIFETIME_END	= 22,
    PSEUDO_PROBE	= 23,
    ARITH_FENCE	= 24,
    STACKMAP	= 25,
    FENTRY_CALL	= 26,
    PATCHPOINT	= 27,
    LOAD_STACK_GUARD	= 28,
    PREALLOCATED_SETUP	= 29,
    PREALLOCATED_ARG	= 30,
    STATEPOINT	= 31,
    LOCAL_ESCAPE	= 32,
    FAULTING_OP	= 33,
    PATCHABLE_OP	= 34,
    PATCHABLE_FUNCTION_ENTER	= 35,
    PATCHABLE_RET	= 36,
    PATCHABLE_FUNCTION_EXIT	= 37,
    PATCHABLE_TAIL_CALL	= 38,
    PATCHABLE_EVENT_CALL	= 39,
    PATCHABLE_TYPED_EVENT_CALL	= 40,
    ICALL_BRANCH_FUNNEL	= 41,
    MEMBARRIER	= 42,
    JUMP_TABLE_DEBUG_INFO	= 43,
    G_ASSERT_SEXT	= 44,
    G_ASSERT_ZEXT	= 45,
    G_ASSERT_ALIGN	= 46,
    G_ADD	= 47,
    G_SUB	= 48,
    G_MUL	= 49,
    G_SDIV	= 50,
    G_UDIV	= 51,
    G_SREM	= 52,
    G_UREM	= 53,
    G_SDIVREM	= 54,
    G_UDIVREM	= 55,
    G_AND	= 56,
    G_OR	= 57,
    G_XOR	= 58,
    G_IMPLICIT_DEF	= 59,
    G_PHI	= 60,
    G_FRAME_INDEX	= 61,
    G_GLOBAL_VALUE	= 62,
    G_CONSTANT_POOL	= 63,
    G_EXTRACT	= 64,
    G_UNMERGE_VALUES	= 65,
    G_INSERT	= 66,
    G_MERGE_VALUES	= 67,
    G_BUILD_VECTOR	= 68,
    G_BUILD_VECTOR_TRUNC	= 69,
    G_CONCAT_VECTORS	= 70,
    G_PTRTOINT	= 71,
    G_INTTOPTR	= 72,
    G_BITCAST	= 73,
    G_FREEZE	= 74,
    G_CONSTANT_FOLD_BARRIER	= 75,
    G_INTRINSIC_FPTRUNC_ROUND	= 76,
    G_INTRINSIC_TRUNC	= 77,
    G_INTRINSIC_ROUND	= 78,
    G_INTRINSIC_LRINT	= 79,
    G_INTRINSIC_ROUNDEVEN	= 80,
    G_READCYCLECOUNTER	= 81,
    G_LOAD	= 82,
    G_SEXTLOAD	= 83,
    G_ZEXTLOAD	= 84,
    G_INDEXED_LOAD	= 85,
    G_INDEXED_SEXTLOAD	= 86,
    G_INDEXED_ZEXTLOAD	= 87,
    G_STORE	= 88,
    G_INDEXED_STORE	= 89,
    G_ATOMIC_CMPXCHG_WITH_SUCCESS	= 90,
    G_ATOMIC_CMPXCHG	= 91,
    G_ATOMICRMW_XCHG	= 92,
    G_ATOMICRMW_ADD	= 93,
    G_ATOMICRMW_SUB	= 94,
    G_ATOMICRMW_AND	= 95,
    G_ATOMICRMW_NAND	= 96,
    G_ATOMICRMW_OR	= 97,
    G_ATOMICRMW_XOR	= 98,
    G_ATOMICRMW_MAX	= 99,
    G_ATOMICRMW_MIN	= 100,
    G_ATOMICRMW_UMAX	= 101,
    G_ATOMICRMW_UMIN	= 102,
    G_ATOMICRMW_FADD	= 103,
    G_ATOMICRMW_FSUB	= 104,
    G_ATOMICRMW_FMAX	= 105,
    G_ATOMICRMW_FMIN	= 106,
    G_ATOMICRMW_UINC_WRAP	= 107,
    G_ATOMICRMW_UDEC_WRAP	= 108,
    G_FENCE	= 109,
    G_PREFETCH	= 110,
    G_BRCOND	= 111,
    G_BRINDIRECT	= 112,
    G_INVOKE_REGION_START	= 113,
    G_INTRINSIC	= 114,
    G_INTRINSIC_W_SIDE_EFFECTS	= 115,
    G_INTRINSIC_CONVERGENT	= 116,
    G_INTRINSIC_CONVERGENT_W_SIDE_EFFECTS	= 117,
    G_ANYEXT	= 118,
    G_TRUNC	= 119,
    G_CONSTANT	= 120,
    G_FCONSTANT	= 121,
    G_VASTART	= 122,
    G_VAARG	= 123,
    G_SEXT	= 124,
    G_SEXT_INREG	= 125,
    G_ZEXT	= 126,
    G_SHL	= 127,
    G_LSHR	= 128,
    G_ASHR	= 129,
    G_FSHL	= 130,
    G_FSHR	= 131,
    G_ROTR	= 132,
    G_ROTL	= 133,
    G_ICMP	= 134,
    G_FCMP	= 135,
    G_SELECT	= 136,
    G_UADDO	= 137,
    G_UADDE	= 138,
    G_USUBO	= 139,
    G_USUBE	= 140,
    G_SADDO	= 141,
    G_SADDE	= 142,
    G_SSUBO	= 143,
    G_SSUBE	= 144,
    G_UMULO	= 145,
    G_SMULO	= 146,
    G_UMULH	= 147,
    G_SMULH	= 148,
    G_UADDSAT	= 149,
    G_SADDSAT	= 150,
    G_USUBSAT	= 151,
    G_SSUBSAT	= 152,
    G_USHLSAT	= 153,
    G_SSHLSAT	= 154,
    G_SMULFIX	= 155,
    G_UMULFIX	= 156,
    G_SMULFIXSAT	= 157,
    G_UMULFIXSAT	= 158,
    G_SDIVFIX	= 159,
    G_UDIVFIX	= 160,
    G_SDIVFIXSAT	= 161,
    G_UDIVFIXSAT	= 162,
    G_FADD	= 163,
    G_FSUB	= 164,
    G_FMUL	= 165,
    G_FMA	= 166,
    G_FMAD	= 167,
    G_FDIV	= 168,
    G_FREM	= 169,
    G_FPOW	= 170,
    G_FPOWI	= 171,
    G_FEXP	= 172,
    G_FEXP2	= 173,
    G_FEXP10	= 174,
    G_FLOG	= 175,
    G_FLOG2	= 176,
    G_FLOG10	= 177,
    G_FLDEXP	= 178,
    G_FFREXP	= 179,
    G_FNEG	= 180,
    G_FPEXT	= 181,
    G_FPTRUNC	= 182,
    G_FPTOSI	= 183,
    G_FPTOUI	= 184,
    G_SITOFP	= 185,
    G_UITOFP	= 186,
    G_FABS	= 187,
    G_FCOPYSIGN	= 188,
    G_IS_FPCLASS	= 189,
    G_FCANONICALIZE	= 190,
    G_FMINNUM	= 191,
    G_FMAXNUM	= 192,
    G_FMINNUM_IEEE	= 193,
    G_FMAXNUM_IEEE	= 194,
    G_FMINIMUM	= 195,
    G_FMAXIMUM	= 196,
    G_GET_FPENV	= 197,
    G_SET_FPENV	= 198,
    G_RESET_FPENV	= 199,
    G_GET_FPMODE	= 200,
    G_SET_FPMODE	= 201,
    G_RESET_FPMODE	= 202,
    G_PTR_ADD	= 203,
    G_PTRMASK	= 204,
    G_SMIN	= 205,
    G_SMAX	= 206,
    G_UMIN	= 207,
    G_UMAX	= 208,
    G_ABS	= 209,
    G_LROUND	= 210,
    G_LLROUND	= 211,
    G_BR	= 212,
    G_BRJT	= 213,
    G_INSERT_VECTOR_ELT	= 214,
    G_EXTRACT_VECTOR_ELT	= 215,
    G_SHUFFLE_VECTOR	= 216,
    G_CTTZ	= 217,
    G_CTTZ_ZERO_UNDEF	= 218,
    G_CTLZ	= 219,
    G_CTLZ_ZERO_UNDEF	= 220,
    G_CTPOP	= 221,
    G_BSWAP	= 222,
    G_BITREVERSE	= 223,
    G_FCEIL	= 224,
    G_FCOS	= 225,
    G_FSIN	= 226,
    G_FSQRT	= 227,
    G_FFLOOR	= 228,
    G_FRINT	= 229,
    G_FNEARBYINT	= 230,
    G_ADDRSPACE_CAST	= 231,
    G_BLOCK_ADDR	= 232,
    G_JUMP_TABLE	= 233,
    G_DYN_STACKALLOC	= 234,
    G_STACKSAVE	= 235,
    G_STACKRESTORE	= 236,
    G_STRICT_FADD	= 237,
    G_STRICT_FSUB	= 238,
    G_STRICT_FMUL	= 239,
    G_STRICT_FDIV	= 240,
    G_STRICT_FREM	= 241,
    G_STRICT_FMA	= 242,
    G_STRICT_FSQRT	= 243,
    G_STRICT_FLDEXP	= 244,
    G_READ_REGISTER	= 245,
    G_WRITE_REGISTER	= 246,
    G_MEMCPY	= 247,
    G_MEMCPY_INLINE	= 248,
    G_MEMMOVE	= 249,
    G_MEMSET	= 250,
    G_BZERO	= 251,
    G_VECREDUCE_SEQ_FADD	= 252,
    G_VECREDUCE_SEQ_FMUL	= 253,
    G_VECREDUCE_FADD	= 254,
    G_VECREDUCE_FMUL	= 255,
    G_VECREDUCE_FMAX	= 256,
    G_VECREDUCE_FMIN	= 257,
    G_VECREDUCE_FMAXIMUM	= 258,
    G_VECREDUCE_FMINIMUM	= 259,
    G_VECREDUCE_ADD	= 260,
    G_VECREDUCE_MUL	= 261,
    G_VECREDUCE_AND	= 262,
    G_VECREDUCE_OR	= 263,
    G_VECREDUCE_XOR	= 264,
    G_VECREDUCE_SMAX	= 265,
    G_VECREDUCE_SMIN	= 266,
    G_VECREDUCE_UMAX	= 267,
    G_VECREDUCE_UMIN	= 268,
    G_SBFX	= 269,
    G_UBFX	= 270,
    ABORT	= 271,
    ABORTMSG	= 272,
    ABS	= 273,
    ADD	= 274,
    AND	= 275,
    APPEND	= 276,
    ASSERT	= 277,
    ASSERTMSG	= 278,
    BOOLAND	= 279,
    BOOLOR	= 280,
    CALL	= 281,
    CALLA	= 282,
    CALLT	= 283,
    CALL_L	= 284,
    CAT	= 285,
    CLEAR	= 286,
    CLEARITEMS	= 287,
    CONVERT	= 288,
    DEC	= 289,
    DEPTH	= 290,
    DIV	= 291,
    DROP	= 292,
    DROPN	= 293,
    DUP	= 294,
    DUPFROMALTSTACK	= 295,
    ENDFINALLY	= 296,
    ENDTRY	= 297,
    ENDTRY_L	= 298,
    EQ	= 299,
    EQUAL	= 300,
    FROMALTSTACK	= 301,
    GE	= 302,
    GT	= 303,
    HASKEY	= 304,
    INC	= 305,
    INITSLOT	= 306,
    INITSSLOT	= 307,
    INVERT	= 308,
    ISNULL	= 309,
    ISTYPE	= 310,
    JMP	= 311,
    JMPEQ	= 312,
    JMPEQ_L	= 313,
    JMPGE	= 314,
    JMPGE_L	= 315,
    JMPGT	= 316,
    JMPGT_L	= 317,
    JMPIF	= 318,
    JMPIFNOT	= 319,
    JMPIFNOT_L	= 320,
    JMPIF_L	= 321,
    JMPLE	= 322,
    JMPLE_L	= 323,
    JMPLT	= 324,
    JMPLT_L	= 325,
    JMPNE	= 326,
    JMPNE_L	= 327,
    JMP_L	= 328,
    KEYS	= 329,
    LDARG	= 330,
    LDARG0	= 331,
    LDARG1	= 332,
    LDARG2	= 333,
    LDARG3	= 334,
    LDARG4	= 335,
    LDARG5	= 336,
    LDARG6	= 337,
    LDLOC	= 338,
    LDLOC0	= 339,
    LDLOC1	= 340,
    LDLOC2	= 341,
    LDLOC3	= 342,
    LDLOC4	= 343,
    LDLOC5	= 344,
    LDLOC6	= 345,
    LDSFLD	= 346,
    LDSFLD0	= 347,
    LDSFLD1	= 348,
    LDSFLD2	= 349,
    LDSFLD3	= 350,
    LDSFLD4	= 351,
    LDSFLD5	= 352,
    LDSFLD6	= 353,
    LE	= 354,
    LEFT	= 355,
    LT	= 356,
    MAX	= 357,
    MEMCPY	= 358,
    MIN	= 359,
    MOD	= 360,
    MODMUL	= 361,
    MODPOW	= 362,
    MUL	= 363,
    NE	= 364,
    NEGATE	= 365,
    NEOVM_LOAD_PSEUDO	= 366,
    NEOVM_STORE_PSEUDO	= 367,
    NEWARRAY	= 368,
    NEWARRAY0	= 369,
    NEWARRAY_T	= 370,
    NEWBUFFER	= 371,
    NEWMAP	= 372,
    NEWSTRUCT	= 373,
    NEWSTRUCT0	= 374,
    NIP	= 375,
    NOP	= 376,
    NOT	= 377,
    NOTEQUAL	= 378,
    NUMEQUAL	= 379,
    NUMNOTEQUAL	= 380,
    NZ	= 381,
    OR	= 382,
    OVER	= 383,
    PACK	= 384,
    PACKMAP	= 385,
    PACKSTRUCT	= 386,
    PICK	= 387,
    PICKITEM	= 388,
    POP	= 389,
    POPITEM	= 390,
    POW	= 391,
    PUSH	= 392,
    PUSH0	= 393,
    PUSH1	= 394,
    PUSH10	= 395,
    PUSH11	= 396,
    PUSH12	= 397,
    PUSH13	= 398,
    PUSH14	= 399,
    PUSH15	= 400,
    PUSH16	= 401,
    PUSH2	= 402,
    PUSH3	= 403,
    PUSH4	= 404,
    PUSH5	= 405,
    PUSH6	= 406,
    PUSH7	= 407,
    PUSH8	= 408,
    PUSH9	= 409,
    PUSHA	= 410,
    PUSHDATA1	= 411,
    PUSHDATA2	= 412,
    PUSHDATA4	= 413,
    PUSHF	= 414,
    PUSHINT128	= 415,
    PUSHINT16	= 416,
    PUSHINT256	= 417,
    PUSHINT32	= 418,
    PUSHINT64	= 419,
    PUSHINT8	= 420,
    PUSHM1	= 421,
    PUSHNULL	= 422,
    PUSHT	= 423,
    REMOVE	= 424,
    RET	= 425,
    REVERSE3	= 426,
    REVERSE4	= 427,
    REVERSEITEMS	= 428,
    REVERSEN	= 429,
    RIGHT	= 430,
    ROLL	= 431,
    ROT	= 432,
    SETITEM	= 433,
    SHL	= 434,
    SHR	= 435,
    SIGN	= 436,
    SIZE	= 437,
    SQRT	= 438,
    STARG	= 439,
    STARG0	= 440,
    STARG1	= 441,
    STARG2	= 442,
    STARG3	= 443,
    STARG4	= 444,
    STARG5	= 445,
    STARG6	= 446,
    STLOC	= 447,
    STLOC0	= 448,
    STLOC1	= 449,
    STLOC2	= 450,
    STLOC3	= 451,
    STLOC4	= 452,
    STLOC5	= 453,
    STLOC6	= 454,
    STSFLD	= 455,
    STSFLD0	= 456,
    STSFLD1	= 457,
    STSFLD2	= 458,
    STSFLD3	= 459,
    STSFLD4	= 460,
    STSFLD5	= 461,
    STSFLD6	= 462,
    SUB	= 463,
    SUBSTR	= 464,
    SWAP	= 465,
    SYSCALL	= 466,
    THROW	= 467,
    TOALTSTACK	= 468,
    TRY	= 469,
    TRY_L	= 470,
    TUCK	= 471,
    UNPACK	= 472,
    VALUES	= 473,
    WITHIN	= 474,
    XDROP	= 475,
    XOR	= 476,
    INSTRUCTION_LIST_END = 477
  };

} // end namespace NeoVM
} // end namespace llvm
#endif // GET_INSTRINFO_ENUM

#ifdef GET_INSTRINFO_SCHED_ENUM
#undef GET_INSTRINFO_SCHED_ENUM
namespace llvm {

namespace NeoVM {
namespace Sched {
  enum {
    NoInstrModel	= 0,
    SCHED_LIST_END = 1
  };
} // end namespace Sched
} // end namespace NeoVM
} // end namespace llvm
#endif // GET_INSTRINFO_SCHED_ENUM

#if defined(GET_INSTRINFO_MC_DESC) || defined(GET_INSTRINFO_CTOR_DTOR)
namespace llvm {

struct NeoVMInstrTable {
  MCInstrDesc Insts[477];
  static_assert(alignof(MCInstrDesc) >= alignof(MCOperandInfo), "Unwanted padding between Insts and OperandInfo");
  MCOperandInfo OperandInfo[157];
  static_assert(alignof(MCOperandInfo) >= alignof(MCPhysReg), "Unwanted padding between OperandInfo and ImplicitOps");
  MCPhysReg ImplicitOps[1];
};

} // end namespace llvm
#endif // defined(GET_INSTRINFO_MC_DESC) || defined(GET_INSTRINFO_CTOR_DTOR)

#ifdef GET_INSTRINFO_MC_DESC
#undef GET_INSTRINFO_MC_DESC
namespace llvm {

static_assert(sizeof(MCOperandInfo) % sizeof(MCPhysReg) == 0);
static constexpr unsigned NeoVMImpOpBase = sizeof NeoVMInstrTable::OperandInfo / (sizeof(MCPhysReg));

extern const NeoVMInstrTable NeoVMDescs = {
  {
    { 476,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #476 = XOR
    { 475,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #475 = XDROP
    { 474,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #474 = WITHIN
    { 473,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #473 = VALUES
    { 472,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #472 = UNPACK
    { 471,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #471 = TUCK
    { 470,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #470 = TRY_L
    { 469,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #469 = TRY
    { 468,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #468 = TOALTSTACK
    { 467,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #467 = THROW
    { 466,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #466 = SYSCALL
    { 465,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #465 = SWAP
    { 464,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #464 = SUBSTR
    { 463,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #463 = SUB
    { 462,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #462 = STSFLD6
    { 461,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #461 = STSFLD5
    { 460,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #460 = STSFLD4
    { 459,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #459 = STSFLD3
    { 458,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #458 = STSFLD2
    { 457,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #457 = STSFLD1
    { 456,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #456 = STSFLD0
    { 455,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #455 = STSFLD
    { 454,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #454 = STLOC6
    { 453,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #453 = STLOC5
    { 452,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #452 = STLOC4
    { 451,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #451 = STLOC3
    { 450,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #450 = STLOC2
    { 449,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #449 = STLOC1
    { 448,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #448 = STLOC0
    { 447,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #447 = STLOC
    { 446,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #446 = STARG6
    { 445,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #445 = STARG5
    { 444,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #444 = STARG4
    { 443,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #443 = STARG3
    { 442,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #442 = STARG2
    { 441,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #441 = STARG1
    { 440,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #440 = STARG0
    { 439,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #439 = STARG
    { 438,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #438 = SQRT
    { 437,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #437 = SIZE
    { 436,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #436 = SIGN
    { 435,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #435 = SHR
    { 434,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #434 = SHL
    { 433,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #433 = SETITEM
    { 432,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	148,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #432 = ROT
    { 431,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #431 = ROLL
    { 430,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #430 = RIGHT
    { 429,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	144,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #429 = REVERSEN
    { 428,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #428 = REVERSEITEMS
    { 427,	5,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	152,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #427 = REVERSE4
    { 426,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	148,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #426 = REVERSE3
    { 425,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #425 = RET
    { 424,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #424 = REMOVE
    { 423,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #423 = PUSHT
    { 422,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #422 = PUSHNULL
    { 421,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #421 = PUSHM1
    { 420,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	146,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #420 = PUSHINT8
    { 419,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	146,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #419 = PUSHINT64
    { 418,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	146,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #418 = PUSHINT32
    { 417,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #417 = PUSHINT256
    { 416,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	146,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #416 = PUSHINT16
    { 415,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #415 = PUSHINT128
    { 414,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #414 = PUSHF
    { 413,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #413 = PUSHDATA4
    { 412,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #412 = PUSHDATA2
    { 411,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #411 = PUSHDATA1
    { 410,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #410 = PUSHA
    { 409,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #409 = PUSH9
    { 408,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #408 = PUSH8
    { 407,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #407 = PUSH7
    { 406,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #406 = PUSH6
    { 405,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #405 = PUSH5
    { 404,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #404 = PUSH4
    { 403,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #403 = PUSH3
    { 402,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #402 = PUSH2
    { 401,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #401 = PUSH16
    { 400,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #400 = PUSH15
    { 399,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #399 = PUSH14
    { 398,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #398 = PUSH13
    { 397,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #397 = PUSH12
    { 396,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #396 = PUSH11
    { 395,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #395 = PUSH10
    { 394,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #394 = PUSH1
    { 393,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #393 = PUSH0
    { 392,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	144,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #392 = PUSH
    { 391,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #391 = POW
    { 390,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #390 = POPITEM
    { 389,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	144,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #389 = POP
    { 388,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #388 = PICKITEM
    { 387,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #387 = PICK
    { 386,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #386 = PACKSTRUCT
    { 385,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #385 = PACKMAP
    { 384,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #384 = PACK
    { 383,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	144,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #383 = OVER
    { 382,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #382 = OR
    { 381,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #381 = NZ
    { 380,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #380 = NUMNOTEQUAL
    { 379,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #379 = NUMEQUAL
    { 378,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #378 = NOTEQUAL
    { 377,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #377 = NOT
    { 376,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #376 = NOP
    { 375,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #375 = NIP
    { 374,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #374 = NEWSTRUCT0
    { 373,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #373 = NEWSTRUCT
    { 372,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #372 = NEWMAP
    { 371,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #371 = NEWBUFFER
    { 370,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #370 = NEWARRAY_T
    { 369,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #369 = NEWARRAY0
    { 368,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #368 = NEWARRAY
    { 367,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	144,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #367 = NEOVM_STORE_PSEUDO
    { 366,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	144,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #366 = NEOVM_LOAD_PSEUDO
    { 365,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #365 = NEGATE
    { 364,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #364 = NE
    { 363,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #363 = MUL
    { 362,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #362 = MODPOW
    { 361,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #361 = MODMUL
    { 360,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #360 = MOD
    { 359,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #359 = MIN
    { 358,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #358 = MEMCPY
    { 357,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #357 = MAX
    { 356,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #356 = LT
    { 355,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #355 = LEFT
    { 354,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #354 = LE
    { 353,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #353 = LDSFLD6
    { 352,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #352 = LDSFLD5
    { 351,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #351 = LDSFLD4
    { 350,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #350 = LDSFLD3
    { 349,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #349 = LDSFLD2
    { 348,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #348 = LDSFLD1
    { 347,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #347 = LDSFLD0
    { 346,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #346 = LDSFLD
    { 345,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #345 = LDLOC6
    { 344,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #344 = LDLOC5
    { 343,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #343 = LDLOC4
    { 342,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #342 = LDLOC3
    { 341,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #341 = LDLOC2
    { 340,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #340 = LDLOC1
    { 339,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #339 = LDLOC0
    { 338,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #338 = LDLOC
    { 337,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #337 = LDARG6
    { 336,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #336 = LDARG5
    { 335,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #335 = LDARG4
    { 334,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #334 = LDARG3
    { 333,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #333 = LDARG2
    { 332,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #332 = LDARG1
    { 331,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #331 = LDARG0
    { 330,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #330 = LDARG
    { 329,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #329 = KEYS
    { 328,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #328 = JMP_L
    { 327,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #327 = JMPNE_L
    { 326,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #326 = JMPNE
    { 325,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #325 = JMPLT_L
    { 324,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #324 = JMPLT
    { 323,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #323 = JMPLE_L
    { 322,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #322 = JMPLE
    { 321,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #321 = JMPIF_L
    { 320,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #320 = JMPIFNOT_L
    { 319,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	146,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #319 = JMPIFNOT
    { 318,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	146,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #318 = JMPIF
    { 317,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #317 = JMPGT_L
    { 316,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #316 = JMPGT
    { 315,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #315 = JMPGE_L
    { 314,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #314 = JMPGE
    { 313,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #313 = JMPEQ_L
    { 312,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #312 = JMPEQ
    { 311,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #311 = JMP
    { 310,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #310 = ISTYPE
    { 309,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #309 = ISNULL
    { 308,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #308 = INVERT
    { 307,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #307 = INITSSLOT
    { 306,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #306 = INITSLOT
    { 305,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #305 = INC
    { 304,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #304 = HASKEY
    { 303,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #303 = GT
    { 302,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #302 = GE
    { 301,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #301 = FROMALTSTACK
    { 300,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #300 = EQUAL
    { 299,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #299 = EQ
    { 298,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #298 = ENDTRY_L
    { 297,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #297 = ENDTRY
    { 296,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #296 = ENDFINALLY
    { 295,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #295 = DUPFROMALTSTACK
    { 294,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	144,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #294 = DUP
    { 293,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #293 = DROPN
    { 292,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #292 = DROP
    { 291,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #291 = DIV
    { 290,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #290 = DEPTH
    { 289,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #289 = DEC
    { 288,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #288 = CONVERT
    { 287,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #287 = CLEARITEMS
    { 286,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #286 = CLEAR
    { 285,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #285 = CAT
    { 284,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #284 = CALL_L
    { 283,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #283 = CALLT
    { 282,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #282 = CALLA
    { 281,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #281 = CALL
    { 280,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #280 = BOOLOR
    { 279,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #279 = BOOLAND
    { 278,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #278 = ASSERTMSG
    { 277,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	143,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #277 = ASSERT
    { 276,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #276 = APPEND
    { 275,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #275 = AND
    { 274,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	140,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #274 = ADD
    { 273,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #273 = ABS
    { 272,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #272 = ABORTMSG
    { 271,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #271 = ABORT
    { 270,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	136,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #270 = G_UBFX
    { 269,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	136,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #269 = G_SBFX
    { 268,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #268 = G_VECREDUCE_UMIN
    { 267,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #267 = G_VECREDUCE_UMAX
    { 266,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #266 = G_VECREDUCE_SMIN
    { 265,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #265 = G_VECREDUCE_SMAX
    { 264,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #264 = G_VECREDUCE_XOR
    { 263,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #263 = G_VECREDUCE_OR
    { 262,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #262 = G_VECREDUCE_AND
    { 261,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #261 = G_VECREDUCE_MUL
    { 260,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #260 = G_VECREDUCE_ADD
    { 259,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #259 = G_VECREDUCE_FMINIMUM
    { 258,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #258 = G_VECREDUCE_FMAXIMUM
    { 257,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #257 = G_VECREDUCE_FMIN
    { 256,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #256 = G_VECREDUCE_FMAX
    { 255,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #255 = G_VECREDUCE_FMUL
    { 254,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #254 = G_VECREDUCE_FADD
    { 253,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	123,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #253 = G_VECREDUCE_SEQ_FMUL
    { 252,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	123,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #252 = G_VECREDUCE_SEQ_FADD
    { 251,	3,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	53,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #251 = G_BZERO
    { 250,	4,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	132,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #250 = G_MEMSET
    { 249,	4,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	132,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #249 = G_MEMMOVE
    { 248,	3,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	123,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #248 = G_MEMCPY_INLINE
    { 247,	4,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	132,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #247 = G_MEMCPY
    { 246,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	130,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq)|(1ULL<<MCID::Convergent), 0x0ULL },  // Inst #246 = G_WRITE_REGISTER
    { 245,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	51,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq)|(1ULL<<MCID::Convergent), 0x0ULL },  // Inst #245 = G_READ_REGISTER
    { 244,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayRaiseFPException)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #244 = G_STRICT_FLDEXP
    { 243,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayRaiseFPException)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #243 = G_STRICT_FSQRT
    { 242,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	46,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayRaiseFPException)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #242 = G_STRICT_FMA
    { 241,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayRaiseFPException)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #241 = G_STRICT_FREM
    { 240,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayRaiseFPException)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #240 = G_STRICT_FDIV
    { 239,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayRaiseFPException)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #239 = G_STRICT_FMUL
    { 238,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayRaiseFPException)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #238 = G_STRICT_FSUB
    { 237,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayRaiseFPException)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #237 = G_STRICT_FADD
    { 236,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #236 = G_STACKRESTORE
    { 235,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #235 = G_STACKSAVE
    { 234,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	64,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #234 = G_DYN_STACKALLOC
    { 233,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	51,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #233 = G_JUMP_TABLE
    { 232,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	51,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #232 = G_BLOCK_ADDR
    { 231,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #231 = G_ADDRSPACE_CAST
    { 230,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #230 = G_FNEARBYINT
    { 229,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #229 = G_FRINT
    { 228,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #228 = G_FFLOOR
    { 227,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #227 = G_FSQRT
    { 226,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #226 = G_FSIN
    { 225,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #225 = G_FCOS
    { 224,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #224 = G_FCEIL
    { 223,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #223 = G_BITREVERSE
    { 222,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #222 = G_BSWAP
    { 221,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #221 = G_CTPOP
    { 220,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #220 = G_CTLZ_ZERO_UNDEF
    { 219,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #219 = G_CTLZ
    { 218,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #218 = G_CTTZ_ZERO_UNDEF
    { 217,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #217 = G_CTTZ
    { 216,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	126,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #216 = G_SHUFFLE_VECTOR
    { 215,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	123,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #215 = G_EXTRACT_VECTOR_ELT
    { 214,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	119,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #214 = G_INSERT_VECTOR_ELT
    { 213,	3,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	116,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Branch)|(1ULL<<MCID::IndirectBranch)|(1ULL<<MCID::Barrier)|(1ULL<<MCID::Terminator)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #213 = G_BRJT
    { 212,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	0,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Branch)|(1ULL<<MCID::Barrier)|(1ULL<<MCID::Terminator)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #212 = G_BR
    { 211,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #211 = G_LLROUND
    { 210,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #210 = G_LROUND
    { 209,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #209 = G_ABS
    { 208,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #208 = G_UMAX
    { 207,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #207 = G_UMIN
    { 206,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #206 = G_SMAX
    { 205,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #205 = G_SMIN
    { 204,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #204 = G_PTRMASK
    { 203,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #203 = G_PTR_ADD
    { 202,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #202 = G_RESET_FPMODE
    { 201,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #201 = G_SET_FPMODE
    { 200,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #200 = G_GET_FPMODE
    { 199,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #199 = G_RESET_FPENV
    { 198,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #198 = G_SET_FPENV
    { 197,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #197 = G_GET_FPENV
    { 196,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #196 = G_FMAXIMUM
    { 195,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #195 = G_FMINIMUM
    { 194,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #194 = G_FMAXNUM_IEEE
    { 193,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #193 = G_FMINNUM_IEEE
    { 192,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #192 = G_FMAXNUM
    { 191,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #191 = G_FMINNUM
    { 190,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #190 = G_FCANONICALIZE
    { 189,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	93,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #189 = G_IS_FPCLASS
    { 188,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #188 = G_FCOPYSIGN
    { 187,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #187 = G_FABS
    { 186,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #186 = G_UITOFP
    { 185,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #185 = G_SITOFP
    { 184,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #184 = G_FPTOUI
    { 183,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #183 = G_FPTOSI
    { 182,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #182 = G_FPTRUNC
    { 181,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #181 = G_FPEXT
    { 180,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #180 = G_FNEG
    { 179,	3,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #179 = G_FFREXP
    { 178,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #178 = G_FLDEXP
    { 177,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #177 = G_FLOG10
    { 176,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #176 = G_FLOG2
    { 175,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #175 = G_FLOG
    { 174,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #174 = G_FEXP10
    { 173,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #173 = G_FEXP2
    { 172,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #172 = G_FEXP
    { 171,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #171 = G_FPOWI
    { 170,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #170 = G_FPOW
    { 169,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #169 = G_FREM
    { 168,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #168 = G_FDIV
    { 167,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	46,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #167 = G_FMAD
    { 166,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	46,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #166 = G_FMA
    { 165,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #165 = G_FMUL
    { 164,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #164 = G_FSUB
    { 163,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #163 = G_FADD
    { 162,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	112,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #162 = G_UDIVFIXSAT
    { 161,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	112,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #161 = G_SDIVFIXSAT
    { 160,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	112,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #160 = G_UDIVFIX
    { 159,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	112,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #159 = G_SDIVFIX
    { 158,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	112,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #158 = G_UMULFIXSAT
    { 157,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	112,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #157 = G_SMULFIXSAT
    { 156,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	112,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #156 = G_UMULFIX
    { 155,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	112,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #155 = G_SMULFIX
    { 154,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #154 = G_SSHLSAT
    { 153,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #153 = G_USHLSAT
    { 152,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #152 = G_SSUBSAT
    { 151,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #151 = G_USUBSAT
    { 150,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #150 = G_SADDSAT
    { 149,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #149 = G_UADDSAT
    { 148,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #148 = G_SMULH
    { 147,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #147 = G_UMULH
    { 146,	4,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	82,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #146 = G_SMULO
    { 145,	4,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	82,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #145 = G_UMULO
    { 144,	5,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	107,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #144 = G_SSUBE
    { 143,	4,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	82,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #143 = G_SSUBO
    { 142,	5,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	107,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #142 = G_SADDE
    { 141,	4,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	82,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #141 = G_SADDO
    { 140,	5,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	107,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #140 = G_USUBE
    { 139,	4,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	82,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #139 = G_USUBO
    { 138,	5,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	107,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #138 = G_UADDE
    { 137,	4,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	82,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #137 = G_UADDO
    { 136,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	82,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #136 = G_SELECT
    { 135,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	103,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #135 = G_FCMP
    { 134,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	103,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #134 = G_ICMP
    { 133,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #133 = G_ROTL
    { 132,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #132 = G_ROTR
    { 131,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	99,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #131 = G_FSHR
    { 130,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	99,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #130 = G_FSHL
    { 129,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #129 = G_ASHR
    { 128,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #128 = G_LSHR
    { 127,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	96,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #127 = G_SHL
    { 126,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #126 = G_ZEXT
    { 125,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	40,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #125 = G_SEXT_INREG
    { 124,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #124 = G_SEXT
    { 123,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	93,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #123 = G_VAARG
    { 122,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #122 = G_VASTART
    { 121,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	51,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #121 = G_FCONSTANT
    { 120,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	51,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #120 = G_CONSTANT
    { 119,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #119 = G_TRUNC
    { 118,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #118 = G_ANYEXT
    { 117,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	0,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq)|(1ULL<<MCID::Convergent), 0x0ULL },  // Inst #117 = G_INTRINSIC_CONVERGENT_W_SIDE_EFFECTS
    { 116,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	0,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq)|(1ULL<<MCID::Convergent), 0x0ULL },  // Inst #116 = G_INTRINSIC_CONVERGENT
    { 115,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	0,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #115 = G_INTRINSIC_W_SIDE_EFFECTS
    { 114,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	0,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #114 = G_INTRINSIC
    { 113,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Terminator)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #113 = G_INVOKE_REGION_START
    { 112,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Branch)|(1ULL<<MCID::IndirectBranch)|(1ULL<<MCID::Barrier)|(1ULL<<MCID::Terminator)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #112 = G_BRINDIRECT
    { 111,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	51,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Branch)|(1ULL<<MCID::Terminator)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #111 = G_BRCOND
    { 110,	4,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	89,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #110 = G_PREFETCH
    { 109,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	21,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #109 = G_FENCE
    { 108,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #108 = G_ATOMICRMW_UDEC_WRAP
    { 107,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #107 = G_ATOMICRMW_UINC_WRAP
    { 106,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #106 = G_ATOMICRMW_FMIN
    { 105,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #105 = G_ATOMICRMW_FMAX
    { 104,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #104 = G_ATOMICRMW_FSUB
    { 103,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #103 = G_ATOMICRMW_FADD
    { 102,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #102 = G_ATOMICRMW_UMIN
    { 101,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #101 = G_ATOMICRMW_UMAX
    { 100,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #100 = G_ATOMICRMW_MIN
    { 99,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #99 = G_ATOMICRMW_MAX
    { 98,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #98 = G_ATOMICRMW_XOR
    { 97,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #97 = G_ATOMICRMW_OR
    { 96,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #96 = G_ATOMICRMW_NAND
    { 95,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #95 = G_ATOMICRMW_AND
    { 94,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #94 = G_ATOMICRMW_SUB
    { 93,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #93 = G_ATOMICRMW_ADD
    { 92,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	86,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #92 = G_ATOMICRMW_XCHG
    { 91,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	82,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #91 = G_ATOMIC_CMPXCHG
    { 90,	5,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	77,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #90 = G_ATOMIC_CMPXCHG_WITH_SUCCESS
    { 89,	5,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	72,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #89 = G_INDEXED_STORE
    { 88,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #88 = G_STORE
    { 87,	5,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	67,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #87 = G_INDEXED_ZEXTLOAD
    { 86,	5,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	67,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #86 = G_INDEXED_SEXTLOAD
    { 85,	5,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	67,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #85 = G_INDEXED_LOAD
    { 84,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #84 = G_ZEXTLOAD
    { 83,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #83 = G_SEXTLOAD
    { 82,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #82 = G_LOAD
    { 81,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #81 = G_READCYCLECOUNTER
    { 80,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #80 = G_INTRINSIC_ROUNDEVEN
    { 79,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #79 = G_INTRINSIC_LRINT
    { 78,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #78 = G_INTRINSIC_ROUND
    { 77,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #77 = G_INTRINSIC_TRUNC
    { 76,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	64,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #76 = G_INTRINSIC_FPTRUNC_ROUND
    { 75,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #75 = G_CONSTANT_FOLD_BARRIER
    { 74,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	62,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #74 = G_FREEZE
    { 73,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #73 = G_BITCAST
    { 72,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #72 = G_INTTOPTR
    { 71,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #71 = G_PTRTOINT
    { 70,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #70 = G_CONCAT_VECTORS
    { 69,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #69 = G_BUILD_VECTOR_TRUNC
    { 68,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #68 = G_BUILD_VECTOR
    { 67,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #67 = G_MERGE_VALUES
    { 66,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	58,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #66 = G_INSERT
    { 65,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	56,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #65 = G_UNMERGE_VALUES
    { 64,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	53,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #64 = G_EXTRACT
    { 63,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	51,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #63 = G_CONSTANT_POOL
    { 62,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	51,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #62 = G_GLOBAL_VALUE
    { 61,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	51,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #61 = G_FRAME_INDEX
    { 60,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #60 = G_PHI
    { 59,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	50,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #59 = G_IMPLICIT_DEF
    { 58,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #58 = G_XOR
    { 57,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #57 = G_OR
    { 56,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #56 = G_AND
    { 55,	4,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	46,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #55 = G_UDIVREM
    { 54,	4,	2,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	46,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #54 = G_SDIVREM
    { 53,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #53 = G_UREM
    { 52,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #52 = G_SREM
    { 51,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #51 = G_UDIV
    { 50,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #50 = G_SDIV
    { 49,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #49 = G_MUL
    { 48,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #48 = G_SUB
    { 47,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	43,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Commutable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #47 = G_ADD
    { 46,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	40,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #46 = G_ASSERT_ALIGN
    { 45,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	40,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #45 = G_ASSERT_ZEXT
    { 44,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	40,	0|(1ULL<<MCID::PreISelOpcode)|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #44 = G_ASSERT_SEXT
    { 43,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #43 = JUMP_TABLE_DEBUG_INFO
    { 42,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #42 = MEMBARRIER
    { 41,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #41 = ICALL_BRANCH_FUNNEL
    { 40,	3,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	37,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Call)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #40 = PATCHABLE_TYPED_EVENT_CALL
    { 39,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	35,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Call)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #39 = PATCHABLE_EVENT_CALL
    { 38,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Return)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #38 = PATCHABLE_TAIL_CALL
    { 37,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #37 = PATCHABLE_FUNCTION_EXIT
    { 36,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Return)|(1ULL<<MCID::Terminator)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #36 = PATCHABLE_RET
    { 35,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #35 = PATCHABLE_FUNCTION_ENTER
    { 34,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #34 = PATCHABLE_OP
    { 33,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	0,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Branch)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::Terminator)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #33 = FAULTING_OP
    { 32,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	33,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #32 = LOCAL_ESCAPE
    { 31,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Call)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #31 = STATEPOINT
    { 30,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	30,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #30 = PREALLOCATED_ARG
    { 29,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #29 = PREALLOCATED_SETUP
    { 28,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	29,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::Rematerializable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #28 = LOAD_STACK_GUARD
    { 27,	6,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	23,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Call)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #27 = PATCHPOINT
    { 26,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Call)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::MayStore)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #26 = FENTRY_CALL
    { 25,	2,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	21,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Call)|(1ULL<<MCID::MayLoad)|(1ULL<<MCID::UsesCustomInserter)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #25 = STACKMAP
    { 24,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	19,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #24 = ARITH_FENCE
    { 23,	4,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	15,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #23 = PSEUDO_PROBE
    { 22,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #22 = LIFETIME_END
    { 21,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #21 = LIFETIME_START
    { 20,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #20 = BUNDLE
    { 19,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	13,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::CheapAsAMove)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #19 = COPY
    { 18,	2,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	13,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::CheapAsAMove)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #18 = REG_SEQUENCE
    { 17,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	0,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #17 = DBG_LABEL
    { 16,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #16 = DBG_PHI
    { 15,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #15 = DBG_INSTR_REF
    { 14,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #14 = DBG_VALUE_LIST
    { 13,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #13 = DBG_VALUE
    { 12,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	2,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::CheapAsAMove)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #12 = COPY_TO_REGCLASS
    { 11,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	9,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #11 = SUBREG_TO_REG
    { 10,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	0,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::Rematerializable)|(1ULL<<MCID::CheapAsAMove)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #10 = IMPLICIT_DEF
    { 9,	4,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	5,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #9 = INSERT_SUBREG
    { 8,	3,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	2,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #8 = EXTRACT_SUBREG
    { 7,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #7 = KILL
    { 6,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::NotDuplicable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #6 = ANNOTATION_LABEL
    { 5,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::NotDuplicable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #5 = GC_LABEL
    { 4,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::NotDuplicable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #4 = EH_LABEL
    { 3,	1,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Meta)|(1ULL<<MCID::NotDuplicable)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #3 = CFI_INSTRUCTION
    { 2,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::UnmodeledSideEffects)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #2 = INLINEASM_BR
    { 1,	0,	0,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	1,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #1 = INLINEASM
    { 0,	1,	1,	0,	0,	0,	0,	NeoVMImpOpBase + 0,	0,	0|(1ULL<<MCID::Pseudo)|(1ULL<<MCID::Variadic)|(1ULL<<MCID::ExtraSrcRegAllocReq)|(1ULL<<MCID::ExtraDefRegAllocReq), 0x0ULL },  // Inst #0 = PHI
  }, {
    /* 0 */ { -1, 0, MCOI::OPERAND_UNKNOWN, 0 },
    /* 1 */
    /* 1 */ { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 2 */ { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 5 */ { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, MCOI_TIED_TO(0) }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 9 */ { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 13 */ { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 },
    /* 15 */ { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 19 */ { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, MCOI_TIED_TO(0) },
    /* 21 */ { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 23 */ { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 29 */ { 0, 0|(1<<MCOI::LookupPtrRegClass), MCOI::OPERAND_UNKNOWN, 0 },
    /* 30 */ { 0, 0|(1<<MCOI::LookupPtrRegClass), MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 33 */ { 0, 0|(1<<MCOI::LookupPtrRegClass), MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 35 */ { 0, 0|(1<<MCOI::LookupPtrRegClass), MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 },
    /* 37 */ { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { 0, 0|(1<<MCOI::LookupPtrRegClass), MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 },
    /* 40 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_IMM_0, 0 },
    /* 43 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 },
    /* 46 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 },
    /* 50 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 },
    /* 51 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 },
    /* 53 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_IMM_0, 0 },
    /* 56 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 },
    /* 58 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_IMM_0, 0 },
    /* 62 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 },
    /* 64 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 67 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_2, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 },
    /* 72 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_2, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 },
    /* 77 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_2, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 },
    /* 82 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 },
    /* 86 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 },
    /* 89 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 93 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 },
    /* 96 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 },
    /* 99 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 },
    /* 103 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 },
    /* 107 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 },
    /* 112 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_IMM_0, 0 },
    /* 116 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 },
    /* 119 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_2, 0 },
    /* 123 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_2, 0 },
    /* 126 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_UNKNOWN, 0 },
    /* 130 */ { -1, 0, MCOI::OPERAND_UNKNOWN, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 },
    /* 132 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_2, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_IMM_0, 0 },
    /* 136 */ { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_0, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 }, { -1, 0, MCOI::OPERAND_GENERIC_1, 0 },
    /* 140 */ { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 },
    /* 143 */ { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 },
    /* 144 */ { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 },
    /* 146 */ { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { -1, 0, MCOI::OPERAND_IMMEDIATE, 0 },
    /* 148 */ { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 },
    /* 152 */ { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 }, { NeoVM::GR32RegClassID, 0, MCOI::OPERAND_REGISTER, 0 },
  }, {
    /* 0 */
  }
};


#ifdef __GNUC__
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Woverlength-strings"
#endif
extern const char NeoVMInstrNameData[] = {
  /* 0 */ "G_FLOG10\0"
  /* 9 */ "PUSH10\0"
  /* 16 */ "G_FEXP10\0"
  /* 25 */ "LDLOC0\0"
  /* 32 */ "STLOC0\0"
  /* 39 */ "LDSFLD0\0"
  /* 47 */ "STSFLD0\0"
  /* 55 */ "LDARG0\0"
  /* 62 */ "STARG0\0"
  /* 69 */ "PUSH0\0"
  /* 75 */ "NEWSTRUCT0\0"
  /* 86 */ "NEWARRAY0\0"
  /* 96 */ "PUSH11\0"
  /* 103 */ "PUSHDATA1\0"
  /* 113 */ "LDLOC1\0"
  /* 120 */ "STLOC1\0"
  /* 127 */ "LDSFLD1\0"
  /* 135 */ "STSFLD1\0"
  /* 143 */ "LDARG1\0"
  /* 150 */ "STARG1\0"
  /* 157 */ "PUSH1\0"
  /* 163 */ "PUSHM1\0"
  /* 170 */ "PUSH12\0"
  /* 177 */ "PUSHINT32\0"
  /* 187 */ "PUSHDATA2\0"
  /* 197 */ "LDLOC2\0"
  /* 204 */ "STLOC2\0"
  /* 211 */ "LDSFLD2\0"
  /* 219 */ "STSFLD2\0"
  /* 227 */ "G_FLOG2\0"
  /* 235 */ "LDARG2\0"
  /* 242 */ "STARG2\0"
  /* 249 */ "PUSH2\0"
  /* 255 */ "G_FEXP2\0"
  /* 263 */ "PUSH13\0"
  /* 270 */ "LDLOC3\0"
  /* 277 */ "STLOC3\0"
  /* 284 */ "LDSFLD3\0"
  /* 292 */ "STSFLD3\0"
  /* 300 */ "REVERSE3\0"
  /* 309 */ "LDARG3\0"
  /* 316 */ "STARG3\0"
  /* 323 */ "PUSH3\0"
  /* 329 */ "PUSH14\0"
  /* 336 */ "PUSHINT64\0"
  /* 346 */ "PUSHDATA4\0"
  /* 356 */ "LDLOC4\0"
  /* 363 */ "STLOC4\0"
  /* 370 */ "LDSFLD4\0"
  /* 378 */ "STSFLD4\0"
  /* 386 */ "REVERSE4\0"
  /* 395 */ "LDARG4\0"
  /* 402 */ "STARG4\0"
  /* 409 */ "PUSH4\0"
  /* 415 */ "PUSH15\0"
  /* 422 */ "LDLOC5\0"
  /* 429 */ "STLOC5\0"
  /* 436 */ "LDSFLD5\0"
  /* 444 */ "STSFLD5\0"
  /* 452 */ "LDARG5\0"
  /* 459 */ "STARG5\0"
  /* 466 */ "PUSH5\0"
  /* 472 */ "PUSH16\0"
  /* 479 */ "PUSHINT16\0"
  /* 489 */ "PUSHINT256\0"
  /* 500 */ "LDLOC6\0"
  /* 507 */ "STLOC6\0"
  /* 514 */ "LDSFLD6\0"
  /* 522 */ "STSFLD6\0"
  /* 530 */ "LDARG6\0"
  /* 537 */ "STARG6\0"
  /* 544 */ "PUSH6\0"
  /* 550 */ "PUSH7\0"
  /* 556 */ "PUSHINT128\0"
  /* 567 */ "PUSH8\0"
  /* 573 */ "PUSHINT8\0"
  /* 582 */ "PUSH9\0"
  /* 588 */ "PUSHA\0"
  /* 594 */ "CALLA\0"
  /* 600 */ "G_FMA\0"
  /* 606 */ "G_STRICT_FMA\0"
  /* 619 */ "G_FSUB\0"
  /* 626 */ "G_STRICT_FSUB\0"
  /* 640 */ "G_ATOMICRMW_FSUB\0"
  /* 657 */ "G_SUB\0"
  /* 663 */ "G_ATOMICRMW_SUB\0"
  /* 679 */ "DEC\0"
  /* 683 */ "G_INTRINSIC\0"
  /* 695 */ "INC\0"
  /* 699 */ "G_FPTRUNC\0"
  /* 709 */ "G_INTRINSIC_TRUNC\0"
  /* 727 */ "G_TRUNC\0"
  /* 735 */ "G_BUILD_VECTOR_TRUNC\0"
  /* 756 */ "LDLOC\0"
  /* 762 */ "G_DYN_STACKALLOC\0"
  /* 779 */ "STLOC\0"
  /* 785 */ "G_FMAD\0"
  /* 792 */ "G_INDEXED_SEXTLOAD\0"
  /* 811 */ "G_SEXTLOAD\0"
  /* 822 */ "G_INDEXED_ZEXTLOAD\0"
  /* 841 */ "G_ZEXTLOAD\0"
  /* 852 */ "G_INDEXED_LOAD\0"
  /* 867 */ "G_LOAD\0"
  /* 874 */ "G_VECREDUCE_FADD\0"
  /* 891 */ "G_FADD\0"
  /* 898 */ "G_VECREDUCE_SEQ_FADD\0"
  /* 919 */ "G_STRICT_FADD\0"
  /* 933 */ "G_ATOMICRMW_FADD\0"
  /* 950 */ "G_VECREDUCE_ADD\0"
  /* 966 */ "G_ADD\0"
  /* 972 */ "G_PTR_ADD\0"
  /* 982 */ "G_ATOMICRMW_ADD\0"
  /* 998 */ "LDSFLD\0"
  /* 1005 */ "STSFLD\0"
  /* 1012 */ "BOOLAND\0"
  /* 1020 */ "G_ATOMICRMW_NAND\0"
  /* 1037 */ "G_VECREDUCE_AND\0"
  /* 1053 */ "G_AND\0"
  /* 1059 */ "G_ATOMICRMW_AND\0"
  /* 1075 */ "APPEND\0"
  /* 1082 */ "LIFETIME_END\0"
  /* 1095 */ "G_BRCOND\0"
  /* 1104 */ "G_LLROUND\0"
  /* 1114 */ "G_LROUND\0"
  /* 1123 */ "G_INTRINSIC_ROUND\0"
  /* 1141 */ "G_INTRINSIC_FPTRUNC_ROUND\0"
  /* 1167 */ "MOD\0"
  /* 1171 */ "LOAD_STACK_GUARD\0"
  /* 1188 */ "PSEUDO_PROBE\0"
  /* 1201 */ "G_SSUBE\0"
  /* 1209 */ "G_USUBE\0"
  /* 1217 */ "G_FENCE\0"
  /* 1225 */ "ARITH_FENCE\0"
  /* 1237 */ "REG_SEQUENCE\0"
  /* 1250 */ "G_SADDE\0"
  /* 1258 */ "G_UADDE\0"
  /* 1266 */ "G_GET_FPMODE\0"
  /* 1279 */ "G_RESET_FPMODE\0"
  /* 1294 */ "G_SET_FPMODE\0"
  /* 1307 */ "G_FMINNUM_IEEE\0"
  /* 1322 */ "G_FMAXNUM_IEEE\0"
  /* 1337 */ "JMPGE\0"
  /* 1343 */ "G_JUMP_TABLE\0"
  /* 1356 */ "BUNDLE\0"
  /* 1363 */ "JMPLE\0"
  /* 1369 */ "G_MEMCPY_INLINE\0"
  /* 1385 */ "JMPNE\0"
  /* 1391 */ "LOCAL_ESCAPE\0"
  /* 1404 */ "ISTYPE\0"
  /* 1411 */ "G_STACKRESTORE\0"
  /* 1426 */ "G_INDEXED_STORE\0"
  /* 1442 */ "G_STORE\0"
  /* 1450 */ "G_BITREVERSE\0"
  /* 1463 */ "NEGATE\0"
  /* 1470 */ "DBG_VALUE\0"
  /* 1480 */ "G_GLOBAL_VALUE\0"
  /* 1495 */ "G_STACKSAVE\0"
  /* 1507 */ "REMOVE\0"
  /* 1514 */ "G_MEMMOVE\0"
  /* 1524 */ "G_FREEZE\0"
  /* 1533 */ "G_FCANONICALIZE\0"
  /* 1549 */ "SIZE\0"
  /* 1554 */ "G_CTLZ_ZERO_UNDEF\0"
  /* 1572 */ "G_CTTZ_ZERO_UNDEF\0"
  /* 1590 */ "G_IMPLICIT_DEF\0"
  /* 1605 */ "DBG_INSTR_REF\0"
  /* 1619 */ "PUSHF\0"
  /* 1625 */ "JMPIF\0"
  /* 1631 */ "G_FNEG\0"
  /* 1638 */ "EXTRACT_SUBREG\0"
  /* 1653 */ "INSERT_SUBREG\0"
  /* 1667 */ "G_SEXT_INREG\0"
  /* 1680 */ "SUBREG_TO_REG\0"
  /* 1694 */ "G_ATOMIC_CMPXCHG\0"
  /* 1711 */ "G_ATOMICRMW_XCHG\0"
  /* 1728 */ "G_FLOG\0"
  /* 1735 */ "G_VAARG\0"
  /* 1743 */ "LDARG\0"
  /* 1749 */ "STARG\0"
  /* 1755 */ "PREALLOCATED_ARG\0"
  /* 1772 */ "ASSERTMSG\0"
  /* 1782 */ "ABORTMSG\0"
  /* 1791 */ "G_PREFETCH\0"
  /* 1802 */ "G_SMULH\0"
  /* 1810 */ "G_UMULH\0"
  /* 1818 */ "PUSH\0"
  /* 1823 */ "DEPTH\0"
  /* 1829 */ "DBG_PHI\0"
  /* 1837 */ "G_FPTOSI\0"
  /* 1846 */ "G_FPTOUI\0"
  /* 1855 */ "G_FPOWI\0"
  /* 1863 */ "UNPACK\0"
  /* 1870 */ "DUPFROMALTSTACK\0"
  /* 1886 */ "TOALTSTACK\0"
  /* 1897 */ "PICK\0"
  /* 1902 */ "TUCK\0"
  /* 1907 */ "G_PTRMASK\0"
  /* 1917 */ "NUMEQUAL\0"
  /* 1926 */ "NUMNOTEQUAL\0"
  /* 1938 */ "GC_LABEL\0"
  /* 1947 */ "DBG_LABEL\0"
  /* 1957 */ "EH_LABEL\0"
  /* 1966 */ "ANNOTATION_LABEL\0"
  /* 1983 */ "ICALL_BRANCH_FUNNEL\0"
  /* 2003 */ "G_FSHL\0"
  /* 2010 */ "G_SHL\0"
  /* 2016 */ "G_FCEIL\0"
  /* 2024 */ "SYSCALL\0"
  /* 2032 */ "PATCHABLE_TAIL_CALL\0"
  /* 2052 */ "PATCHABLE_TYPED_EVENT_CALL\0"
  /* 2079 */ "PATCHABLE_EVENT_CALL\0"
  /* 2100 */ "FENTRY_CALL\0"
  /* 2112 */ "KILL\0"
  /* 2117 */ "ROLL\0"
  /* 2122 */ "PUSHNULL\0"
  /* 2131 */ "ISNULL\0"
  /* 2138 */ "G_CONSTANT_POOL\0"
  /* 2154 */ "G_ROTL\0"
  /* 2161 */ "MODMUL\0"
  /* 2168 */ "G_VECREDUCE_FMUL\0"
  /* 2185 */ "G_FMUL\0"
  /* 2192 */ "G_VECREDUCE_SEQ_FMUL\0"
  /* 2213 */ "G_STRICT_FMUL\0"
  /* 2227 */ "G_VECREDUCE_MUL\0"
  /* 2243 */ "G_MUL\0"
  /* 2249 */ "JMPGE_L\0"
  /* 2257 */ "JMPLE_L\0"
  /* 2265 */ "JMPNE_L\0"
  /* 2273 */ "JMPIF_L\0"
  /* 2281 */ "CALL_L\0"
  /* 2288 */ "JMP_L\0"
  /* 2294 */ "JMPEQ_L\0"
  /* 2302 */ "JMPGT_L\0"
  /* 2310 */ "JMPLT_L\0"
  /* 2318 */ "JMPIFNOT_L\0"
  /* 2329 */ "ENDTRY_L\0"
  /* 2338 */ "G_FREM\0"
  /* 2345 */ "G_STRICT_FREM\0"
  /* 2359 */ "G_SREM\0"
  /* 2366 */ "G_UREM\0"
  /* 2373 */ "G_SDIVREM\0"
  /* 2383 */ "G_UDIVREM\0"
  /* 2393 */ "PICKITEM\0"
  /* 2402 */ "POPITEM\0"
  /* 2410 */ "SETITEM\0"
  /* 2418 */ "INLINEASM\0"
  /* 2428 */ "G_VECREDUCE_FMINIMUM\0"
  /* 2449 */ "G_FMINIMUM\0"
  /* 2460 */ "G_VECREDUCE_FMAXIMUM\0"
  /* 2481 */ "G_FMAXIMUM\0"
  /* 2492 */ "G_FMINNUM\0"
  /* 2502 */ "G_FMAXNUM\0"
  /* 2512 */ "REVERSEN\0"
  /* 2521 */ "G_INTRINSIC_ROUNDEVEN\0"
  /* 2543 */ "G_ASSERT_ALIGN\0"
  /* 2558 */ "G_FCOPYSIGN\0"
  /* 2570 */ "WITHIN\0"
  /* 2577 */ "G_VECREDUCE_FMIN\0"
  /* 2594 */ "G_ATOMICRMW_FMIN\0"
  /* 2611 */ "G_VECREDUCE_SMIN\0"
  /* 2628 */ "G_SMIN\0"
  /* 2635 */ "G_VECREDUCE_UMIN\0"
  /* 2652 */ "G_UMIN\0"
  /* 2659 */ "G_ATOMICRMW_UMIN\0"
  /* 2676 */ "G_ATOMICRMW_MIN\0"
  /* 2692 */ "G_FSIN\0"
  /* 2699 */ "CFI_INSTRUCTION\0"
  /* 2715 */ "DROPN\0"
  /* 2721 */ "G_SSUBO\0"
  /* 2729 */ "G_USUBO\0"
  /* 2737 */ "G_SADDO\0"
  /* 2745 */ "G_UADDO\0"
  /* 2753 */ "NEOVM_LOAD_PSEUDO\0"
  /* 2771 */ "NEOVM_STORE_PSEUDO\0"
  /* 2790 */ "JUMP_TABLE_DEBUG_INFO\0"
  /* 2812 */ "G_SMULO\0"
  /* 2820 */ "G_UMULO\0"
  /* 2828 */ "G_BZERO\0"
  /* 2836 */ "PACKMAP\0"
  /* 2844 */ "STACKMAP\0"
  /* 2853 */ "NEWMAP\0"
  /* 2860 */ "G_ATOMICRMW_UDEC_WRAP\0"
  /* 2882 */ "G_ATOMICRMW_UINC_WRAP\0"
  /* 2904 */ "G_BSWAP\0"
  /* 2912 */ "G_SITOFP\0"
  /* 2921 */ "G_UITOFP\0"
  /* 2930 */ "NIP\0"
  /* 2934 */ "G_FCMP\0"
  /* 2941 */ "G_ICMP\0"
  /* 2948 */ "JMP\0"
  /* 2952 */ "NOP\0"
  /* 2956 */ "G_CTPOP\0"
  /* 2964 */ "XDROP\0"
  /* 2970 */ "PATCHABLE_OP\0"
  /* 2983 */ "FAULTING_OP\0"
  /* 2995 */ "DUP\0"
  /* 2999 */ "PREALLOCATED_SETUP\0"
  /* 3018 */ "G_FLDEXP\0"
  /* 3027 */ "G_STRICT_FLDEXP\0"
  /* 3043 */ "G_FEXP\0"
  /* 3050 */ "G_FFREXP\0"
  /* 3059 */ "JMPEQ\0"
  /* 3065 */ "CLEAR\0"
  /* 3071 */ "G_BR\0"
  /* 3076 */ "INLINEASM_BR\0"
  /* 3089 */ "G_BLOCK_ADDR\0"
  /* 3102 */ "NEWBUFFER\0"
  /* 3112 */ "MEMBARRIER\0"
  /* 3123 */ "G_CONSTANT_FOLD_BARRIER\0"
  /* 3147 */ "PATCHABLE_FUNCTION_ENTER\0"
  /* 3172 */ "G_READCYCLECOUNTER\0"
  /* 3191 */ "G_READ_REGISTER\0"
  /* 3207 */ "G_WRITE_REGISTER\0"
  /* 3224 */ "OVER\0"
  /* 3229 */ "G_ASHR\0"
  /* 3236 */ "G_FSHR\0"
  /* 3243 */ "G_LSHR\0"
  /* 3250 */ "BOOLOR\0"
  /* 3257 */ "G_FFLOOR\0"
  /* 3266 */ "G_BUILD_VECTOR\0"
  /* 3281 */ "G_SHUFFLE_VECTOR\0"
  /* 3298 */ "G_VECREDUCE_XOR\0"
  /* 3314 */ "G_XOR\0"
  /* 3320 */ "G_ATOMICRMW_XOR\0"
  /* 3336 */ "G_VECREDUCE_OR\0"
  /* 3351 */ "G_OR\0"
  /* 3356 */ "G_ATOMICRMW_OR\0"
  /* 3371 */ "G_ROTR\0"
  /* 3378 */ "G_INTTOPTR\0"
  /* 3389 */ "SUBSTR\0"
  /* 3396 */ "G_FABS\0"
  /* 3403 */ "G_ABS\0"
  /* 3409 */ "G_UNMERGE_VALUES\0"
  /* 3426 */ "G_MERGE_VALUES\0"
  /* 3441 */ "REVERSEITEMS\0"
  /* 3454 */ "CLEARITEMS\0"
  /* 3465 */ "G_FCOS\0"
  /* 3472 */ "G_CONCAT_VECTORS\0"
  /* 3489 */ "COPY_TO_REGCLASS\0"
  /* 3506 */ "G_IS_FPCLASS\0"
  /* 3519 */ "G_ATOMIC_CMPXCHG_WITH_SUCCESS\0"
  /* 3549 */ "G_INTRINSIC_W_SIDE_EFFECTS\0"
  /* 3576 */ "G_INTRINSIC_CONVERGENT_W_SIDE_EFFECTS\0"
  /* 3614 */ "KEYS\0"
  /* 3619 */ "CAT\0"
  /* 3623 */ "G_SSUBSAT\0"
  /* 3633 */ "G_USUBSAT\0"
  /* 3643 */ "G_SADDSAT\0"
  /* 3653 */ "G_UADDSAT\0"
  /* 3663 */ "G_SSHLSAT\0"
  /* 3673 */ "G_USHLSAT\0"
  /* 3683 */ "G_SMULFIXSAT\0"
  /* 3696 */ "G_UMULFIXSAT\0"
  /* 3709 */ "G_SDIVFIXSAT\0"
  /* 3722 */ "G_UDIVFIXSAT\0"
  /* 3735 */ "G_EXTRACT\0"
  /* 3745 */ "G_SELECT\0"
  /* 3754 */ "G_BRINDIRECT\0"
  /* 3767 */ "PACKSTRUCT\0"
  /* 3778 */ "NEWSTRUCT\0"
  /* 3788 */ "PATCHABLE_RET\0"
  /* 3802 */ "G_MEMSET\0"
  /* 3811 */ "LEFT\0"
  /* 3816 */ "JMPGT\0"
  /* 3822 */ "RIGHT\0"
  /* 3828 */ "PUSHT\0"
  /* 3834 */ "PATCHABLE_FUNCTION_EXIT\0"
  /* 3858 */ "G_BRJT\0"
  /* 3865 */ "G_EXTRACT_VECTOR_ELT\0"
  /* 3886 */ "G_INSERT_VECTOR_ELT\0"
  /* 3906 */ "CALLT\0"
  /* 3912 */ "JMPLT\0"
  /* 3918 */ "G_FCONSTANT\0"
  /* 3930 */ "G_CONSTANT\0"
  /* 3941 */ "G_INTRINSIC_CONVERGENT\0"
  /* 3964 */ "STATEPOINT\0"
  /* 3975 */ "PATCHPOINT\0"
  /* 3986 */ "G_PTRTOINT\0"
  /* 3997 */ "G_FRINT\0"
  /* 4005 */ "G_INTRINSIC_LRINT\0"
  /* 4023 */ "G_FNEARBYINT\0"
  /* 4036 */ "INITSSLOT\0"
  /* 4046 */ "INITSLOT\0"
  /* 4055 */ "JMPIFNOT\0"
  /* 4064 */ "ROT\0"
  /* 4068 */ "G_VASTART\0"
  /* 4078 */ "LIFETIME_START\0"
  /* 4093 */ "G_INVOKE_REGION_START\0"
  /* 4115 */ "G_INSERT\0"
  /* 4124 */ "ASSERT\0"
  /* 4131 */ "INVERT\0"
  /* 4138 */ "CONVERT\0"
  /* 4146 */ "ABORT\0"
  /* 4152 */ "G_FSQRT\0"
  /* 4160 */ "G_STRICT_FSQRT\0"
  /* 4175 */ "G_BITCAST\0"
  /* 4185 */ "G_ADDRSPACE_CAST\0"
  /* 4202 */ "DBG_VALUE_LIST\0"
  /* 4217 */ "G_FPEXT\0"
  /* 4225 */ "G_SEXT\0"
  /* 4232 */ "G_ASSERT_SEXT\0"
  /* 4246 */ "G_ANYEXT\0"
  /* 4255 */ "G_ZEXT\0"
  /* 4262 */ "G_ASSERT_ZEXT\0"
  /* 4276 */ "NEWARRAY_T\0"
  /* 4287 */ "G_FDIV\0"
  /* 4294 */ "G_STRICT_FDIV\0"
  /* 4308 */ "G_SDIV\0"
  /* 4315 */ "G_UDIV\0"
  /* 4322 */ "G_GET_FPENV\0"
  /* 4334 */ "G_RESET_FPENV\0"
  /* 4348 */ "G_SET_FPENV\0"
  /* 4360 */ "MODPOW\0"
  /* 4367 */ "G_FPOW\0"
  /* 4374 */ "THROW\0"
  /* 4380 */ "G_VECREDUCE_FMAX\0"
  /* 4397 */ "G_ATOMICRMW_FMAX\0"
  /* 4414 */ "G_VECREDUCE_SMAX\0"
  /* 4431 */ "G_SMAX\0"
  /* 4438 */ "G_VECREDUCE_UMAX\0"
  /* 4455 */ "G_UMAX\0"
  /* 4462 */ "G_ATOMICRMW_UMAX\0"
  /* 4479 */ "G_ATOMICRMW_MAX\0"
  /* 4495 */ "G_FRAME_INDEX\0"
  /* 4509 */ "G_SBFX\0"
  /* 4516 */ "G_UBFX\0"
  /* 4523 */ "G_SMULFIX\0"
  /* 4533 */ "G_UMULFIX\0"
  /* 4543 */ "G_SDIVFIX\0"
  /* 4553 */ "G_UDIVFIX\0"
  /* 4563 */ "NEWARRAY\0"
  /* 4572 */ "HASKEY\0"
  /* 4579 */ "ENDFINALLY\0"
  /* 4590 */ "G_MEMCPY\0"
  /* 4599 */ "COPY\0"
  /* 4604 */ "ENDTRY\0"
  /* 4611 */ "G_CTLZ\0"
  /* 4618 */ "NZ\0"
  /* 4621 */ "G_CTTZ\0"
};
#ifdef __GNUC__
#pragma GCC diagnostic pop
#endif

extern const unsigned NeoVMInstrNameIndices[] = {
    1833U, 2418U, 3076U, 2699U, 1957U, 1938U, 1966U, 2112U, 
    1638U, 1653U, 1592U, 1680U, 3489U, 1470U, 4202U, 1605U, 
    1829U, 1947U, 1237U, 4599U, 1356U, 4078U, 1082U, 1188U, 
    1225U, 2844U, 2100U, 3975U, 1171U, 2999U, 1755U, 3964U, 
    1391U, 2983U, 2970U, 3147U, 3788U, 3834U, 2032U, 2079U, 
    2052U, 1983U, 3112U, 2790U, 4232U, 4262U, 2543U, 966U, 
    657U, 2243U, 4308U, 4315U, 2359U, 2366U, 2373U, 2383U, 
    1053U, 3351U, 3314U, 1590U, 1831U, 4495U, 1480U, 2138U, 
    3735U, 3409U, 4115U, 3426U, 3266U, 735U, 3472U, 3986U, 
    3378U, 4175U, 1524U, 3123U, 1141U, 709U, 1123U, 4005U, 
    2521U, 3172U, 867U, 811U, 841U, 852U, 792U, 822U, 
    1442U, 1426U, 3519U, 1694U, 1711U, 982U, 663U, 1059U, 
    1020U, 3356U, 3320U, 4479U, 2676U, 4462U, 2659U, 933U, 
    640U, 4397U, 2594U, 2882U, 2860U, 1217U, 1791U, 1095U, 
    3754U, 4093U, 683U, 3549U, 3941U, 3576U, 4246U, 727U, 
    3930U, 3918U, 4068U, 1735U, 4225U, 1667U, 4255U, 2010U, 
    3243U, 3229U, 2003U, 3236U, 3371U, 2154U, 2941U, 2934U, 
    3745U, 2745U, 1258U, 2729U, 1209U, 2737U, 1250U, 2721U, 
    1201U, 2820U, 2812U, 1810U, 1802U, 3653U, 3643U, 3633U, 
    3623U, 3673U, 3663U, 4523U, 4533U, 3683U, 3696U, 4543U, 
    4553U, 3709U, 3722U, 891U, 619U, 2185U, 600U, 785U, 
    4287U, 2338U, 4367U, 1855U, 3043U, 255U, 16U, 1728U, 
    227U, 0U, 3018U, 3050U, 1631U, 4217U, 699U, 1837U, 
    1846U, 2912U, 2921U, 3396U, 2558U, 3506U, 1533U, 2492U, 
    2502U, 1307U, 1322U, 2449U, 2481U, 4322U, 4348U, 4334U, 
    1266U, 1294U, 1279U, 972U, 1907U, 2628U, 4431U, 2652U, 
    4455U, 3403U, 1114U, 1104U, 3071U, 3858U, 3886U, 3865U, 
    3281U, 4621U, 1572U, 4611U, 1554U, 2956U, 2904U, 1450U, 
    2016U, 3465U, 2692U, 4152U, 3257U, 3997U, 4023U, 4185U, 
    3089U, 1343U, 762U, 1495U, 1411U, 919U, 626U, 2213U, 
    4294U, 2345U, 606U, 4160U, 3027U, 3191U, 3207U, 4590U, 
    1369U, 1514U, 3802U, 2828U, 898U, 2192U, 874U, 2168U, 
    4380U, 2577U, 2460U, 2428U, 950U, 2227U, 1037U, 3336U, 
    3298U, 4414U, 2611U, 4438U, 2635U, 4509U, 4516U, 4146U, 
    1782U, 3399U, 887U, 1016U, 1075U, 4124U, 1772U, 1012U, 
    3250U, 2027U, 594U, 3906U, 2281U, 3619U, 3065U, 3454U, 
    4138U, 679U, 1823U, 4290U, 2965U, 2715U, 2995U, 1870U, 
    4579U, 4604U, 2329U, 3062U, 1920U, 1873U, 1340U, 3819U, 
    4572U, 695U, 4046U, 4036U, 4131U, 2131U, 1404U, 2948U, 
    3059U, 2294U, 1337U, 2249U, 3816U, 2302U, 1625U, 4055U, 
    2318U, 2273U, 1363U, 2257U, 3912U, 2310U, 1385U, 2265U, 
    2288U, 3614U, 1743U, 55U, 143U, 235U, 309U, 395U, 
    452U, 530U, 756U, 25U, 113U, 197U, 270U, 356U, 
    422U, 500U, 998U, 39U, 127U, 211U, 284U, 370U, 
    436U, 514U, 1353U, 3811U, 3883U, 4393U, 4592U, 2590U, 
    1167U, 2161U, 4360U, 2164U, 1382U, 1463U, 2753U, 2771U, 
    4563U, 86U, 4276U, 3102U, 2853U, 3778U, 75U, 2930U, 
    2952U, 4060U, 1929U, 1917U, 1926U, 4618U, 3254U, 3224U, 
    1865U, 2836U, 3767U, 1897U, 2393U, 2960U, 2402U, 4363U, 
    1818U, 69U, 157U, 9U, 96U, 170U, 263U, 329U, 
    415U, 472U, 249U, 323U, 409U, 466U, 544U, 550U, 
    567U, 582U, 588U, 103U, 187U, 346U, 1619U, 556U, 
    479U, 489U, 177U, 336U, 573U, 163U, 2122U, 3828U, 
    1507U, 3798U, 300U, 386U, 3441U, 2512U, 3822U, 2117U, 
    4064U, 2410U, 2006U, 3232U, 2565U, 1549U, 4155U, 1749U, 
    62U, 150U, 242U, 316U, 402U, 459U, 537U, 779U, 
    32U, 120U, 204U, 277U, 363U, 429U, 507U, 1005U, 
    47U, 135U, 219U, 292U, 378U, 444U, 522U, 622U, 
    3389U, 2907U, 2024U, 4374U, 1886U, 4607U, 2332U, 1902U, 
    1863U, 3419U, 2570U, 2964U, 3310U, 
};

static inline void InitNeoVMMCInstrInfo(MCInstrInfo *II) {
  II->InitMCInstrInfo(NeoVMDescs.Insts, NeoVMInstrNameIndices, NeoVMInstrNameData, nullptr, nullptr, 477);
}

} // end namespace llvm
#endif // GET_INSTRINFO_MC_DESC

#ifdef GET_INSTRINFO_HEADER
#undef GET_INSTRINFO_HEADER
namespace llvm {
struct NeoVMGenInstrInfo : public TargetInstrInfo {
  explicit NeoVMGenInstrInfo(unsigned CFSetupOpcode = ~0u, unsigned CFDestroyOpcode = ~0u, unsigned CatchRetOpcode = ~0u, unsigned ReturnOpcode = ~0u);
  ~NeoVMGenInstrInfo() override = default;

};
} // end namespace llvm
#endif // GET_INSTRINFO_HEADER

#ifdef GET_INSTRINFO_HELPER_DECLS
#undef GET_INSTRINFO_HELPER_DECLS


#endif // GET_INSTRINFO_HELPER_DECLS

#ifdef GET_INSTRINFO_HELPERS
#undef GET_INSTRINFO_HELPERS

#endif // GET_INSTRINFO_HELPERS

#ifdef GET_INSTRINFO_CTOR_DTOR
#undef GET_INSTRINFO_CTOR_DTOR
namespace llvm {
extern const NeoVMInstrTable NeoVMDescs;
extern const unsigned NeoVMInstrNameIndices[];
extern const char NeoVMInstrNameData[];
NeoVMGenInstrInfo::NeoVMGenInstrInfo(unsigned CFSetupOpcode, unsigned CFDestroyOpcode, unsigned CatchRetOpcode, unsigned ReturnOpcode)
  : TargetInstrInfo(CFSetupOpcode, CFDestroyOpcode, CatchRetOpcode, ReturnOpcode) {
  InitMCInstrInfo(NeoVMDescs.Insts, NeoVMInstrNameIndices, NeoVMInstrNameData, nullptr, nullptr, 477);
}
} // end namespace llvm
#endif // GET_INSTRINFO_CTOR_DTOR

#ifdef GET_INSTRINFO_OPERAND_ENUM
#undef GET_INSTRINFO_OPERAND_ENUM
namespace llvm {
namespace NeoVM {
namespace OpName {
enum {
  OPERAND_LAST
};
} // end namespace OpName
} // end namespace NeoVM
} // end namespace llvm
#endif //GET_INSTRINFO_OPERAND_ENUM

#ifdef GET_INSTRINFO_NAMED_OPS
#undef GET_INSTRINFO_NAMED_OPS
namespace llvm {
namespace NeoVM {
LLVM_READONLY
int16_t getNamedOperandIdx(uint16_t Opcode, uint16_t NamedIdx) {
  return -1;
}
} // end namespace NeoVM
} // end namespace llvm
#endif //GET_INSTRINFO_NAMED_OPS

#ifdef GET_INSTRINFO_OPERAND_TYPES_ENUM
#undef GET_INSTRINFO_OPERAND_TYPES_ENUM
namespace llvm {
namespace NeoVM {
namespace OpTypes {
enum OperandType {
  f32imm = 0,
  f64imm = 1,
  i1imm = 2,
  i8imm = 3,
  i16imm = 4,
  i32imm = 5,
  i64imm = 6,
  ptype0 = 7,
  ptype1 = 8,
  ptype2 = 9,
  ptype3 = 10,
  ptype4 = 11,
  ptype5 = 12,
  type0 = 13,
  type1 = 14,
  type2 = 15,
  type3 = 16,
  type4 = 17,
  type5 = 18,
  untyped_imm_0 = 19,
  GR32 = 20,
  OPERAND_TYPE_LIST_END
};
} // end namespace OpTypes
} // end namespace NeoVM
} // end namespace llvm
#endif // GET_INSTRINFO_OPERAND_TYPES_ENUM

#ifdef GET_INSTRINFO_OPERAND_TYPE
#undef GET_INSTRINFO_OPERAND_TYPE
namespace llvm {
namespace NeoVM {
LLVM_READONLY
static int getOperandType(uint16_t Opcode, uint16_t OpIdx) {
  static const uint16_t Offsets[] = {
    /* PHI */
    0,
    /* INLINEASM */
    1,
    /* INLINEASM_BR */
    1,
    /* CFI_INSTRUCTION */
    1,
    /* EH_LABEL */
    2,
    /* GC_LABEL */
    3,
    /* ANNOTATION_LABEL */
    4,
    /* KILL */
    5,
    /* EXTRACT_SUBREG */
    5,
    /* INSERT_SUBREG */
    8,
    /* IMPLICIT_DEF */
    12,
    /* SUBREG_TO_REG */
    13,
    /* COPY_TO_REGCLASS */
    17,
    /* DBG_VALUE */
    20,
    /* DBG_VALUE_LIST */
    20,
    /* DBG_INSTR_REF */
    20,
    /* DBG_PHI */
    20,
    /* DBG_LABEL */
    20,
    /* REG_SEQUENCE */
    21,
    /* COPY */
    23,
    /* BUNDLE */
    25,
    /* LIFETIME_START */
    25,
    /* LIFETIME_END */
    26,
    /* PSEUDO_PROBE */
    27,
    /* ARITH_FENCE */
    31,
    /* STACKMAP */
    33,
    /* FENTRY_CALL */
    35,
    /* PATCHPOINT */
    35,
    /* LOAD_STACK_GUARD */
    41,
    /* PREALLOCATED_SETUP */
    42,
    /* PREALLOCATED_ARG */
    43,
    /* STATEPOINT */
    46,
    /* LOCAL_ESCAPE */
    46,
    /* FAULTING_OP */
    48,
    /* PATCHABLE_OP */
    49,
    /* PATCHABLE_FUNCTION_ENTER */
    49,
    /* PATCHABLE_RET */
    49,
    /* PATCHABLE_FUNCTION_EXIT */
    49,
    /* PATCHABLE_TAIL_CALL */
    49,
    /* PATCHABLE_EVENT_CALL */
    49,
    /* PATCHABLE_TYPED_EVENT_CALL */
    51,
    /* ICALL_BRANCH_FUNNEL */
    54,
    /* MEMBARRIER */
    54,
    /* JUMP_TABLE_DEBUG_INFO */
    54,
    /* G_ASSERT_SEXT */
    55,
    /* G_ASSERT_ZEXT */
    58,
    /* G_ASSERT_ALIGN */
    61,
    /* G_ADD */
    64,
    /* G_SUB */
    67,
    /* G_MUL */
    70,
    /* G_SDIV */
    73,
    /* G_UDIV */
    76,
    /* G_SREM */
    79,
    /* G_UREM */
    82,
    /* G_SDIVREM */
    85,
    /* G_UDIVREM */
    89,
    /* G_AND */
    93,
    /* G_OR */
    96,
    /* G_XOR */
    99,
    /* G_IMPLICIT_DEF */
    102,
    /* G_PHI */
    103,
    /* G_FRAME_INDEX */
    104,
    /* G_GLOBAL_VALUE */
    106,
    /* G_CONSTANT_POOL */
    108,
    /* G_EXTRACT */
    110,
    /* G_UNMERGE_VALUES */
    113,
    /* G_INSERT */
    115,
    /* G_MERGE_VALUES */
    119,
    /* G_BUILD_VECTOR */
    121,
    /* G_BUILD_VECTOR_TRUNC */
    123,
    /* G_CONCAT_VECTORS */
    125,
    /* G_PTRTOINT */
    127,
    /* G_INTTOPTR */
    129,
    /* G_BITCAST */
    131,
    /* G_FREEZE */
    133,
    /* G_CONSTANT_FOLD_BARRIER */
    135,
    /* G_INTRINSIC_FPTRUNC_ROUND */
    137,
    /* G_INTRINSIC_TRUNC */
    140,
    /* G_INTRINSIC_ROUND */
    142,
    /* G_INTRINSIC_LRINT */
    144,
    /* G_INTRINSIC_ROUNDEVEN */
    146,
    /* G_READCYCLECOUNTER */
    148,
    /* G_LOAD */
    149,
    /* G_SEXTLOAD */
    151,
    /* G_ZEXTLOAD */
    153,
    /* G_INDEXED_LOAD */
    155,
    /* G_INDEXED_SEXTLOAD */
    160,
    /* G_INDEXED_ZEXTLOAD */
    165,
    /* G_STORE */
    170,
    /* G_INDEXED_STORE */
    172,
    /* G_ATOMIC_CMPXCHG_WITH_SUCCESS */
    177,
    /* G_ATOMIC_CMPXCHG */
    182,
    /* G_ATOMICRMW_XCHG */
    186,
    /* G_ATOMICRMW_ADD */
    189,
    /* G_ATOMICRMW_SUB */
    192,
    /* G_ATOMICRMW_AND */
    195,
    /* G_ATOMICRMW_NAND */
    198,
    /* G_ATOMICRMW_OR */
    201,
    /* G_ATOMICRMW_XOR */
    204,
    /* G_ATOMICRMW_MAX */
    207,
    /* G_ATOMICRMW_MIN */
    210,
    /* G_ATOMICRMW_UMAX */
    213,
    /* G_ATOMICRMW_UMIN */
    216,
    /* G_ATOMICRMW_FADD */
    219,
    /* G_ATOMICRMW_FSUB */
    222,
    /* G_ATOMICRMW_FMAX */
    225,
    /* G_ATOMICRMW_FMIN */
    228,
    /* G_ATOMICRMW_UINC_WRAP */
    231,
    /* G_ATOMICRMW_UDEC_WRAP */
    234,
    /* G_FENCE */
    237,
    /* G_PREFETCH */
    239,
    /* G_BRCOND */
    243,
    /* G_BRINDIRECT */
    245,
    /* G_INVOKE_REGION_START */
    246,
    /* G_INTRINSIC */
    246,
    /* G_INTRINSIC_W_SIDE_EFFECTS */
    247,
    /* G_INTRINSIC_CONVERGENT */
    248,
    /* G_INTRINSIC_CONVERGENT_W_SIDE_EFFECTS */
    249,
    /* G_ANYEXT */
    250,
    /* G_TRUNC */
    252,
    /* G_CONSTANT */
    254,
    /* G_FCONSTANT */
    256,
    /* G_VASTART */
    258,
    /* G_VAARG */
    259,
    /* G_SEXT */
    262,
    /* G_SEXT_INREG */
    264,
    /* G_ZEXT */
    267,
    /* G_SHL */
    269,
    /* G_LSHR */
    272,
    /* G_ASHR */
    275,
    /* G_FSHL */
    278,
    /* G_FSHR */
    282,
    /* G_ROTR */
    286,
    /* G_ROTL */
    289,
    /* G_ICMP */
    292,
    /* G_FCMP */
    296,
    /* G_SELECT */
    300,
    /* G_UADDO */
    304,
    /* G_UADDE */
    308,
    /* G_USUBO */
    313,
    /* G_USUBE */
    317,
    /* G_SADDO */
    322,
    /* G_SADDE */
    326,
    /* G_SSUBO */
    331,
    /* G_SSUBE */
    335,
    /* G_UMULO */
    340,
    /* G_SMULO */
    344,
    /* G_UMULH */
    348,
    /* G_SMULH */
    351,
    /* G_UADDSAT */
    354,
    /* G_SADDSAT */
    357,
    /* G_USUBSAT */
    360,
    /* G_SSUBSAT */
    363,
    /* G_USHLSAT */
    366,
    /* G_SSHLSAT */
    369,
    /* G_SMULFIX */
    372,
    /* G_UMULFIX */
    376,
    /* G_SMULFIXSAT */
    380,
    /* G_UMULFIXSAT */
    384,
    /* G_SDIVFIX */
    388,
    /* G_UDIVFIX */
    392,
    /* G_SDIVFIXSAT */
    396,
    /* G_UDIVFIXSAT */
    400,
    /* G_FADD */
    404,
    /* G_FSUB */
    407,
    /* G_FMUL */
    410,
    /* G_FMA */
    413,
    /* G_FMAD */
    417,
    /* G_FDIV */
    421,
    /* G_FREM */
    424,
    /* G_FPOW */
    427,
    /* G_FPOWI */
    430,
    /* G_FEXP */
    433,
    /* G_FEXP2 */
    435,
    /* G_FEXP10 */
    437,
    /* G_FLOG */
    439,
    /* G_FLOG2 */
    441,
    /* G_FLOG10 */
    443,
    /* G_FLDEXP */
    445,
    /* G_FFREXP */
    448,
    /* G_FNEG */
    451,
    /* G_FPEXT */
    453,
    /* G_FPTRUNC */
    455,
    /* G_FPTOSI */
    457,
    /* G_FPTOUI */
    459,
    /* G_SITOFP */
    461,
    /* G_UITOFP */
    463,
    /* G_FABS */
    465,
    /* G_FCOPYSIGN */
    467,
    /* G_IS_FPCLASS */
    470,
    /* G_FCANONICALIZE */
    473,
    /* G_FMINNUM */
    475,
    /* G_FMAXNUM */
    478,
    /* G_FMINNUM_IEEE */
    481,
    /* G_FMAXNUM_IEEE */
    484,
    /* G_FMINIMUM */
    487,
    /* G_FMAXIMUM */
    490,
    /* G_GET_FPENV */
    493,
    /* G_SET_FPENV */
    494,
    /* G_RESET_FPENV */
    495,
    /* G_GET_FPMODE */
    495,
    /* G_SET_FPMODE */
    496,
    /* G_RESET_FPMODE */
    497,
    /* G_PTR_ADD */
    497,
    /* G_PTRMASK */
    500,
    /* G_SMIN */
    503,
    /* G_SMAX */
    506,
    /* G_UMIN */
    509,
    /* G_UMAX */
    512,
    /* G_ABS */
    515,
    /* G_LROUND */
    517,
    /* G_LLROUND */
    519,
    /* G_BR */
    521,
    /* G_BRJT */
    522,
    /* G_INSERT_VECTOR_ELT */
    525,
    /* G_EXTRACT_VECTOR_ELT */
    529,
    /* G_SHUFFLE_VECTOR */
    532,
    /* G_CTTZ */
    536,
    /* G_CTTZ_ZERO_UNDEF */
    538,
    /* G_CTLZ */
    540,
    /* G_CTLZ_ZERO_UNDEF */
    542,
    /* G_CTPOP */
    544,
    /* G_BSWAP */
    546,
    /* G_BITREVERSE */
    548,
    /* G_FCEIL */
    550,
    /* G_FCOS */
    552,
    /* G_FSIN */
    554,
    /* G_FSQRT */
    556,
    /* G_FFLOOR */
    558,
    /* G_FRINT */
    560,
    /* G_FNEARBYINT */
    562,
    /* G_ADDRSPACE_CAST */
    564,
    /* G_BLOCK_ADDR */
    566,
    /* G_JUMP_TABLE */
    568,
    /* G_DYN_STACKALLOC */
    570,
    /* G_STACKSAVE */
    573,
    /* G_STACKRESTORE */
    574,
    /* G_STRICT_FADD */
    575,
    /* G_STRICT_FSUB */
    578,
    /* G_STRICT_FMUL */
    581,
    /* G_STRICT_FDIV */
    584,
    /* G_STRICT_FREM */
    587,
    /* G_STRICT_FMA */
    590,
    /* G_STRICT_FSQRT */
    594,
    /* G_STRICT_FLDEXP */
    596,
    /* G_READ_REGISTER */
    599,
    /* G_WRITE_REGISTER */
    601,
    /* G_MEMCPY */
    603,
    /* G_MEMCPY_INLINE */
    607,
    /* G_MEMMOVE */
    610,
    /* G_MEMSET */
    614,
    /* G_BZERO */
    618,
    /* G_VECREDUCE_SEQ_FADD */
    621,
    /* G_VECREDUCE_SEQ_FMUL */
    624,
    /* G_VECREDUCE_FADD */
    627,
    /* G_VECREDUCE_FMUL */
    629,
    /* G_VECREDUCE_FMAX */
    631,
    /* G_VECREDUCE_FMIN */
    633,
    /* G_VECREDUCE_FMAXIMUM */
    635,
    /* G_VECREDUCE_FMINIMUM */
    637,
    /* G_VECREDUCE_ADD */
    639,
    /* G_VECREDUCE_MUL */
    641,
    /* G_VECREDUCE_AND */
    643,
    /* G_VECREDUCE_OR */
    645,
    /* G_VECREDUCE_XOR */
    647,
    /* G_VECREDUCE_SMAX */
    649,
    /* G_VECREDUCE_SMIN */
    651,
    /* G_VECREDUCE_UMAX */
    653,
    /* G_VECREDUCE_UMIN */
    655,
    /* G_SBFX */
    657,
    /* G_UBFX */
    661,
    /* ABORT */
    665,
    /* ABORTMSG */
    665,
    /* ABS */
    665,
    /* ADD */
    665,
    /* AND */
    668,
    /* APPEND */
    668,
    /* ASSERT */
    668,
    /* ASSERTMSG */
    669,
    /* BOOLAND */
    669,
    /* BOOLOR */
    669,
    /* CALL */
    669,
    /* CALLA */
    670,
    /* CALLT */
    670,
    /* CALL_L */
    670,
    /* CAT */
    670,
    /* CLEAR */
    670,
    /* CLEARITEMS */
    670,
    /* CONVERT */
    670,
    /* DEC */
    670,
    /* DEPTH */
    670,
    /* DIV */
    670,
    /* DROP */
    673,
    /* DROPN */
    674,
    /* DUP */
    675,
    /* DUPFROMALTSTACK */
    677,
    /* ENDFINALLY */
    678,
    /* ENDTRY */
    678,
    /* ENDTRY_L */
    678,
    /* EQ */
    678,
    /* EQUAL */
    681,
    /* FROMALTSTACK */
    681,
    /* GE */
    682,
    /* GT */
    685,
    /* HASKEY */
    688,
    /* INC */
    688,
    /* INITSLOT */
    688,
    /* INITSSLOT */
    688,
    /* INVERT */
    688,
    /* ISNULL */
    688,
    /* ISTYPE */
    688,
    /* JMP */
    688,
    /* JMPEQ */
    689,
    /* JMPEQ_L */
    689,
    /* JMPGE */
    689,
    /* JMPGE_L */
    689,
    /* JMPGT */
    689,
    /* JMPGT_L */
    689,
    /* JMPIF */
    689,
    /* JMPIFNOT */
    691,
    /* JMPIFNOT_L */
    693,
    /* JMPIF_L */
    693,
    /* JMPLE */
    693,
    /* JMPLE_L */
    693,
    /* JMPLT */
    693,
    /* JMPLT_L */
    693,
    /* JMPNE */
    693,
    /* JMPNE_L */
    693,
    /* JMP_L */
    693,
    /* KEYS */
    693,
    /* LDARG */
    693,
    /* LDARG0 */
    693,
    /* LDARG1 */
    693,
    /* LDARG2 */
    693,
    /* LDARG3 */
    693,
    /* LDARG4 */
    693,
    /* LDARG5 */
    693,
    /* LDARG6 */
    693,
    /* LDLOC */
    693,
    /* LDLOC0 */
    693,
    /* LDLOC1 */
    693,
    /* LDLOC2 */
    693,
    /* LDLOC3 */
    693,
    /* LDLOC4 */
    693,
    /* LDLOC5 */
    693,
    /* LDLOC6 */
    693,
    /* LDSFLD */
    693,
    /* LDSFLD0 */
    693,
    /* LDSFLD1 */
    693,
    /* LDSFLD2 */
    693,
    /* LDSFLD3 */
    693,
    /* LDSFLD4 */
    693,
    /* LDSFLD5 */
    693,
    /* LDSFLD6 */
    693,
    /* LE */
    693,
    /* LEFT */
    696,
    /* LT */
    696,
    /* MAX */
    699,
    /* MEMCPY */
    699,
    /* MIN */
    699,
    /* MOD */
    699,
    /* MODMUL */
    699,
    /* MODPOW */
    699,
    /* MUL */
    699,
    /* NE */
    702,
    /* NEGATE */
    705,
    /* NEOVM_LOAD_PSEUDO */
    705,
    /* NEOVM_STORE_PSEUDO */
    707,
    /* NEWARRAY */
    709,
    /* NEWARRAY0 */
    709,
    /* NEWARRAY_T */
    709,
    /* NEWBUFFER */
    709,
    /* NEWMAP */
    709,
    /* NEWSTRUCT */
    709,
    /* NEWSTRUCT0 */
    709,
    /* NIP */
    709,
    /* NOP */
    709,
    /* NOT */
    709,
    /* NOTEQUAL */
    709,
    /* NUMEQUAL */
    709,
    /* NUMNOTEQUAL */
    709,
    /* NZ */
    709,
    /* OR */
    709,
    /* OVER */
    709,
    /* PACK */
    711,
    /* PACKMAP */
    711,
    /* PACKSTRUCT */
    711,
    /* PICK */
    711,
    /* PICKITEM */
    711,
    /* POP */
    711,
    /* POPITEM */
    713,
    /* POW */
    713,
    /* PUSH */
    713,
    /* PUSH0 */
    715,
    /* PUSH1 */
    716,
    /* PUSH10 */
    717,
    /* PUSH11 */
    718,
    /* PUSH12 */
    719,
    /* PUSH13 */
    720,
    /* PUSH14 */
    721,
    /* PUSH15 */
    722,
    /* PUSH16 */
    723,
    /* PUSH2 */
    724,
    /* PUSH3 */
    725,
    /* PUSH4 */
    726,
    /* PUSH5 */
    727,
    /* PUSH6 */
    728,
    /* PUSH7 */
    729,
    /* PUSH8 */
    730,
    /* PUSH9 */
    731,
    /* PUSHA */
    732,
    /* PUSHDATA1 */
    732,
    /* PUSHDATA2 */
    732,
    /* PUSHDATA4 */
    732,
    /* PUSHF */
    732,
    /* PUSHINT128 */
    732,
    /* PUSHINT16 */
    732,
    /* PUSHINT256 */
    734,
    /* PUSHINT32 */
    734,
    /* PUSHINT64 */
    736,
    /* PUSHINT8 */
    738,
    /* PUSHM1 */
    740,
    /* PUSHNULL */
    741,
    /* PUSHT */
    741,
    /* REMOVE */
    741,
    /* RET */
    741,
    /* REVERSE3 */
    741,
    /* REVERSE4 */
    745,
    /* REVERSEITEMS */
    750,
    /* REVERSEN */
    750,
    /* RIGHT */
    752,
    /* ROLL */
    752,
    /* ROT */
    755,
    /* SETITEM */
    759,
    /* SHL */
    759,
    /* SHR */
    759,
    /* SIGN */
    759,
    /* SIZE */
    759,
    /* SQRT */
    759,
    /* STARG */
    759,
    /* STARG0 */
    759,
    /* STARG1 */
    759,
    /* STARG2 */
    759,
    /* STARG3 */
    759,
    /* STARG4 */
    759,
    /* STARG5 */
    759,
    /* STARG6 */
    759,
    /* STLOC */
    759,
    /* STLOC0 */
    759,
    /* STLOC1 */
    759,
    /* STLOC2 */
    759,
    /* STLOC3 */
    759,
    /* STLOC4 */
    759,
    /* STLOC5 */
    759,
    /* STLOC6 */
    759,
    /* STSFLD */
    759,
    /* STSFLD0 */
    759,
    /* STSFLD1 */
    759,
    /* STSFLD2 */
    759,
    /* STSFLD3 */
    759,
    /* STSFLD4 */
    759,
    /* STSFLD5 */
    759,
    /* STSFLD6 */
    759,
    /* SUB */
    759,
    /* SUBSTR */
    762,
    /* SWAP */
    762,
    /* SYSCALL */
    765,
    /* THROW */
    766,
    /* TOALTSTACK */
    766,
    /* TRY */
    767,
    /* TRY_L */
    767,
    /* TUCK */
    767,
    /* UNPACK */
    767,
    /* VALUES */
    767,
    /* WITHIN */
    767,
    /* XDROP */
    767,
    /* XOR */
    767,
  };

  using namespace OpTypes;
  static const int8_t OpcodeOperandTypes[] = {
    
    /* PHI */
    -1, 
    /* INLINEASM */
    /* INLINEASM_BR */
    /* CFI_INSTRUCTION */
    i32imm, 
    /* EH_LABEL */
    i32imm, 
    /* GC_LABEL */
    i32imm, 
    /* ANNOTATION_LABEL */
    i32imm, 
    /* KILL */
    /* EXTRACT_SUBREG */
    -1, -1, i32imm, 
    /* INSERT_SUBREG */
    -1, -1, -1, i32imm, 
    /* IMPLICIT_DEF */
    -1, 
    /* SUBREG_TO_REG */
    -1, -1, -1, i32imm, 
    /* COPY_TO_REGCLASS */
    -1, -1, i32imm, 
    /* DBG_VALUE */
    /* DBG_VALUE_LIST */
    /* DBG_INSTR_REF */
    /* DBG_PHI */
    /* DBG_LABEL */
    -1, 
    /* REG_SEQUENCE */
    -1, -1, 
    /* COPY */
    -1, -1, 
    /* BUNDLE */
    /* LIFETIME_START */
    i32imm, 
    /* LIFETIME_END */
    i32imm, 
    /* PSEUDO_PROBE */
    i64imm, i64imm, i8imm, i32imm, 
    /* ARITH_FENCE */
    -1, -1, 
    /* STACKMAP */
    i64imm, i32imm, 
    /* FENTRY_CALL */
    /* PATCHPOINT */
    -1, i64imm, i32imm, -1, i32imm, i32imm, 
    /* LOAD_STACK_GUARD */
    -1, 
    /* PREALLOCATED_SETUP */
    i32imm, 
    /* PREALLOCATED_ARG */
    -1, i32imm, i32imm, 
    /* STATEPOINT */
    /* LOCAL_ESCAPE */
    -1, i32imm, 
    /* FAULTING_OP */
    -1, 
    /* PATCHABLE_OP */
    /* PATCHABLE_FUNCTION_ENTER */
    /* PATCHABLE_RET */
    /* PATCHABLE_FUNCTION_EXIT */
    /* PATCHABLE_TAIL_CALL */
    /* PATCHABLE_EVENT_CALL */
    -1, -1, 
    /* PATCHABLE_TYPED_EVENT_CALL */
    -1, -1, -1, 
    /* ICALL_BRANCH_FUNNEL */
    /* MEMBARRIER */
    /* JUMP_TABLE_DEBUG_INFO */
    i64imm, 
    /* G_ASSERT_SEXT */
    type0, type0, untyped_imm_0, 
    /* G_ASSERT_ZEXT */
    type0, type0, untyped_imm_0, 
    /* G_ASSERT_ALIGN */
    type0, type0, untyped_imm_0, 
    /* G_ADD */
    type0, type0, type0, 
    /* G_SUB */
    type0, type0, type0, 
    /* G_MUL */
    type0, type0, type0, 
    /* G_SDIV */
    type0, type0, type0, 
    /* G_UDIV */
    type0, type0, type0, 
    /* G_SREM */
    type0, type0, type0, 
    /* G_UREM */
    type0, type0, type0, 
    /* G_SDIVREM */
    type0, type0, type0, type0, 
    /* G_UDIVREM */
    type0, type0, type0, type0, 
    /* G_AND */
    type0, type0, type0, 
    /* G_OR */
    type0, type0, type0, 
    /* G_XOR */
    type0, type0, type0, 
    /* G_IMPLICIT_DEF */
    type0, 
    /* G_PHI */
    type0, 
    /* G_FRAME_INDEX */
    type0, -1, 
    /* G_GLOBAL_VALUE */
    type0, -1, 
    /* G_CONSTANT_POOL */
    type0, -1, 
    /* G_EXTRACT */
    type0, type1, untyped_imm_0, 
    /* G_UNMERGE_VALUES */
    type0, type1, 
    /* G_INSERT */
    type0, type0, type1, untyped_imm_0, 
    /* G_MERGE_VALUES */
    type0, type1, 
    /* G_BUILD_VECTOR */
    type0, type1, 
    /* G_BUILD_VECTOR_TRUNC */
    type0, type1, 
    /* G_CONCAT_VECTORS */
    type0, type1, 
    /* G_PTRTOINT */
    type0, type1, 
    /* G_INTTOPTR */
    type0, type1, 
    /* G_BITCAST */
    type0, type1, 
    /* G_FREEZE */
    type0, type0, 
    /* G_CONSTANT_FOLD_BARRIER */
    type0, type0, 
    /* G_INTRINSIC_FPTRUNC_ROUND */
    type0, type1, i32imm, 
    /* G_INTRINSIC_TRUNC */
    type0, type0, 
    /* G_INTRINSIC_ROUND */
    type0, type0, 
    /* G_INTRINSIC_LRINT */
    type0, type1, 
    /* G_INTRINSIC_ROUNDEVEN */
    type0, type0, 
    /* G_READCYCLECOUNTER */
    type0, 
    /* G_LOAD */
    type0, ptype1, 
    /* G_SEXTLOAD */
    type0, ptype1, 
    /* G_ZEXTLOAD */
    type0, ptype1, 
    /* G_INDEXED_LOAD */
    type0, ptype1, ptype1, type2, -1, 
    /* G_INDEXED_SEXTLOAD */
    type0, ptype1, ptype1, type2, -1, 
    /* G_INDEXED_ZEXTLOAD */
    type0, ptype1, ptype1, type2, -1, 
    /* G_STORE */
    type0, ptype1, 
    /* G_INDEXED_STORE */
    ptype0, type1, ptype0, ptype2, -1, 
    /* G_ATOMIC_CMPXCHG_WITH_SUCCESS */
    type0, type1, type2, type0, type0, 
    /* G_ATOMIC_CMPXCHG */
    type0, ptype1, type0, type0, 
    /* G_ATOMICRMW_XCHG */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_ADD */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_SUB */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_AND */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_NAND */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_OR */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_XOR */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_MAX */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_MIN */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_UMAX */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_UMIN */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_FADD */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_FSUB */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_FMAX */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_FMIN */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_UINC_WRAP */
    type0, ptype1, type0, 
    /* G_ATOMICRMW_UDEC_WRAP */
    type0, ptype1, type0, 
    /* G_FENCE */
    i32imm, i32imm, 
    /* G_PREFETCH */
    ptype0, i32imm, i32imm, i32imm, 
    /* G_BRCOND */
    type0, -1, 
    /* G_BRINDIRECT */
    type0, 
    /* G_INVOKE_REGION_START */
    /* G_INTRINSIC */
    -1, 
    /* G_INTRINSIC_W_SIDE_EFFECTS */
    -1, 
    /* G_INTRINSIC_CONVERGENT */
    -1, 
    /* G_INTRINSIC_CONVERGENT_W_SIDE_EFFECTS */
    -1, 
    /* G_ANYEXT */
    type0, type1, 
    /* G_TRUNC */
    type0, type1, 
    /* G_CONSTANT */
    type0, -1, 
    /* G_FCONSTANT */
    type0, -1, 
    /* G_VASTART */
    type0, 
    /* G_VAARG */
    type0, type1, -1, 
    /* G_SEXT */
    type0, type1, 
    /* G_SEXT_INREG */
    type0, type0, untyped_imm_0, 
    /* G_ZEXT */
    type0, type1, 
    /* G_SHL */
    type0, type0, type1, 
    /* G_LSHR */
    type0, type0, type1, 
    /* G_ASHR */
    type0, type0, type1, 
    /* G_FSHL */
    type0, type0, type0, type1, 
    /* G_FSHR */
    type0, type0, type0, type1, 
    /* G_ROTR */
    type0, type0, type1, 
    /* G_ROTL */
    type0, type0, type1, 
    /* G_ICMP */
    type0, -1, type1, type1, 
    /* G_FCMP */
    type0, -1, type1, type1, 
    /* G_SELECT */
    type0, type1, type0, type0, 
    /* G_UADDO */
    type0, type1, type0, type0, 
    /* G_UADDE */
    type0, type1, type0, type0, type1, 
    /* G_USUBO */
    type0, type1, type0, type0, 
    /* G_USUBE */
    type0, type1, type0, type0, type1, 
    /* G_SADDO */
    type0, type1, type0, type0, 
    /* G_SADDE */
    type0, type1, type0, type0, type1, 
    /* G_SSUBO */
    type0, type1, type0, type0, 
    /* G_SSUBE */
    type0, type1, type0, type0, type1, 
    /* G_UMULO */
    type0, type1, type0, type0, 
    /* G_SMULO */
    type0, type1, type0, type0, 
    /* G_UMULH */
    type0, type0, type0, 
    /* G_SMULH */
    type0, type0, type0, 
    /* G_UADDSAT */
    type0, type0, type0, 
    /* G_SADDSAT */
    type0, type0, type0, 
    /* G_USUBSAT */
    type0, type0, type0, 
    /* G_SSUBSAT */
    type0, type0, type0, 
    /* G_USHLSAT */
    type0, type0, type1, 
    /* G_SSHLSAT */
    type0, type0, type1, 
    /* G_SMULFIX */
    type0, type0, type0, untyped_imm_0, 
    /* G_UMULFIX */
    type0, type0, type0, untyped_imm_0, 
    /* G_SMULFIXSAT */
    type0, type0, type0, untyped_imm_0, 
    /* G_UMULFIXSAT */
    type0, type0, type0, untyped_imm_0, 
    /* G_SDIVFIX */
    type0, type0, type0, untyped_imm_0, 
    /* G_UDIVFIX */
    type0, type0, type0, untyped_imm_0, 
    /* G_SDIVFIXSAT */
    type0, type0, type0, untyped_imm_0, 
    /* G_UDIVFIXSAT */
    type0, type0, type0, untyped_imm_0, 
    /* G_FADD */
    type0, type0, type0, 
    /* G_FSUB */
    type0, type0, type0, 
    /* G_FMUL */
    type0, type0, type0, 
    /* G_FMA */
    type0, type0, type0, type0, 
    /* G_FMAD */
    type0, type0, type0, type0, 
    /* G_FDIV */
    type0, type0, type0, 
    /* G_FREM */
    type0, type0, type0, 
    /* G_FPOW */
    type0, type0, type0, 
    /* G_FPOWI */
    type0, type0, type1, 
    /* G_FEXP */
    type0, type0, 
    /* G_FEXP2 */
    type0, type0, 
    /* G_FEXP10 */
    type0, type0, 
    /* G_FLOG */
    type0, type0, 
    /* G_FLOG2 */
    type0, type0, 
    /* G_FLOG10 */
    type0, type0, 
    /* G_FLDEXP */
    type0, type0, type1, 
    /* G_FFREXP */
    type0, type1, type0, 
    /* G_FNEG */
    type0, type0, 
    /* G_FPEXT */
    type0, type1, 
    /* G_FPTRUNC */
    type0, type1, 
    /* G_FPTOSI */
    type0, type1, 
    /* G_FPTOUI */
    type0, type1, 
    /* G_SITOFP */
    type0, type1, 
    /* G_UITOFP */
    type0, type1, 
    /* G_FABS */
    type0, type0, 
    /* G_FCOPYSIGN */
    type0, type0, type1, 
    /* G_IS_FPCLASS */
    type0, type1, -1, 
    /* G_FCANONICALIZE */
    type0, type0, 
    /* G_FMINNUM */
    type0, type0, type0, 
    /* G_FMAXNUM */
    type0, type0, type0, 
    /* G_FMINNUM_IEEE */
    type0, type0, type0, 
    /* G_FMAXNUM_IEEE */
    type0, type0, type0, 
    /* G_FMINIMUM */
    type0, type0, type0, 
    /* G_FMAXIMUM */
    type0, type0, type0, 
    /* G_GET_FPENV */
    type0, 
    /* G_SET_FPENV */
    type0, 
    /* G_RESET_FPENV */
    /* G_GET_FPMODE */
    type0, 
    /* G_SET_FPMODE */
    type0, 
    /* G_RESET_FPMODE */
    /* G_PTR_ADD */
    ptype0, ptype0, type1, 
    /* G_PTRMASK */
    ptype0, ptype0, type1, 
    /* G_SMIN */
    type0, type0, type0, 
    /* G_SMAX */
    type0, type0, type0, 
    /* G_UMIN */
    type0, type0, type0, 
    /* G_UMAX */
    type0, type0, type0, 
    /* G_ABS */
    type0, type0, 
    /* G_LROUND */
    type0, type1, 
    /* G_LLROUND */
    type0, type1, 
    /* G_BR */
    -1, 
    /* G_BRJT */
    ptype0, -1, type1, 
    /* G_INSERT_VECTOR_ELT */
    type0, type0, type1, type2, 
    /* G_EXTRACT_VECTOR_ELT */
    type0, type1, type2, 
    /* G_SHUFFLE_VECTOR */
    type0, type1, type1, -1, 
    /* G_CTTZ */
    type0, type1, 
    /* G_CTTZ_ZERO_UNDEF */
    type0, type1, 
    /* G_CTLZ */
    type0, type1, 
    /* G_CTLZ_ZERO_UNDEF */
    type0, type1, 
    /* G_CTPOP */
    type0, type1, 
    /* G_BSWAP */
    type0, type0, 
    /* G_BITREVERSE */
    type0, type0, 
    /* G_FCEIL */
    type0, type0, 
    /* G_FCOS */
    type0, type0, 
    /* G_FSIN */
    type0, type0, 
    /* G_FSQRT */
    type0, type0, 
    /* G_FFLOOR */
    type0, type0, 
    /* G_FRINT */
    type0, type0, 
    /* G_FNEARBYINT */
    type0, type0, 
    /* G_ADDRSPACE_CAST */
    type0, type1, 
    /* G_BLOCK_ADDR */
    type0, -1, 
    /* G_JUMP_TABLE */
    type0, -1, 
    /* G_DYN_STACKALLOC */
    ptype0, type1, i32imm, 
    /* G_STACKSAVE */
    ptype0, 
    /* G_STACKRESTORE */
    ptype0, 
    /* G_STRICT_FADD */
    type0, type0, type0, 
    /* G_STRICT_FSUB */
    type0, type0, type0, 
    /* G_STRICT_FMUL */
    type0, type0, type0, 
    /* G_STRICT_FDIV */
    type0, type0, type0, 
    /* G_STRICT_FREM */
    type0, type0, type0, 
    /* G_STRICT_FMA */
    type0, type0, type0, type0, 
    /* G_STRICT_FSQRT */
    type0, type0, 
    /* G_STRICT_FLDEXP */
    type0, type0, type1, 
    /* G_READ_REGISTER */
    type0, -1, 
    /* G_WRITE_REGISTER */
    -1, type0, 
    /* G_MEMCPY */
    ptype0, ptype1, type2, untyped_imm_0, 
    /* G_MEMCPY_INLINE */
    ptype0, ptype1, type2, 
    /* G_MEMMOVE */
    ptype0, ptype1, type2, untyped_imm_0, 
    /* G_MEMSET */
    ptype0, type1, type2, untyped_imm_0, 
    /* G_BZERO */
    ptype0, type1, untyped_imm_0, 
    /* G_VECREDUCE_SEQ_FADD */
    type0, type1, type2, 
    /* G_VECREDUCE_SEQ_FMUL */
    type0, type1, type2, 
    /* G_VECREDUCE_FADD */
    type0, type1, 
    /* G_VECREDUCE_FMUL */
    type0, type1, 
    /* G_VECREDUCE_FMAX */
    type0, type1, 
    /* G_VECREDUCE_FMIN */
    type0, type1, 
    /* G_VECREDUCE_FMAXIMUM */
    type0, type1, 
    /* G_VECREDUCE_FMINIMUM */
    type0, type1, 
    /* G_VECREDUCE_ADD */
    type0, type1, 
    /* G_VECREDUCE_MUL */
    type0, type1, 
    /* G_VECREDUCE_AND */
    type0, type1, 
    /* G_VECREDUCE_OR */
    type0, type1, 
    /* G_VECREDUCE_XOR */
    type0, type1, 
    /* G_VECREDUCE_SMAX */
    type0, type1, 
    /* G_VECREDUCE_SMIN */
    type0, type1, 
    /* G_VECREDUCE_UMAX */
    type0, type1, 
    /* G_VECREDUCE_UMIN */
    type0, type1, 
    /* G_SBFX */
    type0, type0, type1, type1, 
    /* G_UBFX */
    type0, type0, type1, type1, 
    /* ABORT */
    /* ABORTMSG */
    /* ABS */
    /* ADD */
    GR32, GR32, GR32, 
    /* AND */
    /* APPEND */
    /* ASSERT */
    GR32, 
    /* ASSERTMSG */
    /* BOOLAND */
    /* BOOLOR */
    /* CALL */
    i32imm, 
    /* CALLA */
    /* CALLT */
    /* CALL_L */
    /* CAT */
    /* CLEAR */
    /* CLEARITEMS */
    /* CONVERT */
    /* DEC */
    /* DEPTH */
    /* DIV */
    GR32, GR32, GR32, 
    /* DROP */
    GR32, 
    /* DROPN */
    GR32, 
    /* DUP */
    GR32, GR32, 
    /* DUPFROMALTSTACK */
    GR32, 
    /* ENDFINALLY */
    /* ENDTRY */
    /* ENDTRY_L */
    /* EQ */
    GR32, GR32, GR32, 
    /* EQUAL */
    /* FROMALTSTACK */
    GR32, 
    /* GE */
    GR32, GR32, GR32, 
    /* GT */
    GR32, GR32, GR32, 
    /* HASKEY */
    /* INC */
    /* INITSLOT */
    /* INITSSLOT */
    /* INVERT */
    /* ISNULL */
    /* ISTYPE */
    /* JMP */
    i32imm, 
    /* JMPEQ */
    /* JMPEQ_L */
    /* JMPGE */
    /* JMPGE_L */
    /* JMPGT */
    /* JMPGT_L */
    /* JMPIF */
    GR32, i32imm, 
    /* JMPIFNOT */
    GR32, i32imm, 
    /* JMPIFNOT_L */
    /* JMPIF_L */
    /* JMPLE */
    /* JMPLE_L */
    /* JMPLT */
    /* JMPLT_L */
    /* JMPNE */
    /* JMPNE_L */
    /* JMP_L */
    /* KEYS */
    /* LDARG */
    /* LDARG0 */
    /* LDARG1 */
    /* LDARG2 */
    /* LDARG3 */
    /* LDARG4 */
    /* LDARG5 */
    /* LDARG6 */
    /* LDLOC */
    /* LDLOC0 */
    /* LDLOC1 */
    /* LDLOC2 */
    /* LDLOC3 */
    /* LDLOC4 */
    /* LDLOC5 */
    /* LDLOC6 */
    /* LDSFLD */
    /* LDSFLD0 */
    /* LDSFLD1 */
    /* LDSFLD2 */
    /* LDSFLD3 */
    /* LDSFLD4 */
    /* LDSFLD5 */
    /* LDSFLD6 */
    /* LE */
    GR32, GR32, GR32, 
    /* LEFT */
    /* LT */
    GR32, GR32, GR32, 
    /* MAX */
    /* MEMCPY */
    /* MIN */
    /* MOD */
    /* MODMUL */
    /* MODPOW */
    /* MUL */
    GR32, GR32, GR32, 
    /* NE */
    GR32, GR32, GR32, 
    /* NEGATE */
    /* NEOVM_LOAD_PSEUDO */
    GR32, GR32, 
    /* NEOVM_STORE_PSEUDO */
    GR32, GR32, 
    /* NEWARRAY */
    /* NEWARRAY0 */
    /* NEWARRAY_T */
    /* NEWBUFFER */
    /* NEWMAP */
    /* NEWSTRUCT */
    /* NEWSTRUCT0 */
    /* NIP */
    /* NOP */
    /* NOT */
    /* NOTEQUAL */
    /* NUMEQUAL */
    /* NUMNOTEQUAL */
    /* NZ */
    /* OR */
    /* OVER */
    GR32, GR32, 
    /* PACK */
    /* PACKMAP */
    /* PACKSTRUCT */
    /* PICK */
    /* PICKITEM */
    /* POP */
    GR32, GR32, 
    /* POPITEM */
    /* POW */
    /* PUSH */
    GR32, GR32, 
    /* PUSH0 */
    GR32, 
    /* PUSH1 */
    GR32, 
    /* PUSH10 */
    GR32, 
    /* PUSH11 */
    GR32, 
    /* PUSH12 */
    GR32, 
    /* PUSH13 */
    GR32, 
    /* PUSH14 */
    GR32, 
    /* PUSH15 */
    GR32, 
    /* PUSH16 */
    GR32, 
    /* PUSH2 */
    GR32, 
    /* PUSH3 */
    GR32, 
    /* PUSH4 */
    GR32, 
    /* PUSH5 */
    GR32, 
    /* PUSH6 */
    GR32, 
    /* PUSH7 */
    GR32, 
    /* PUSH8 */
    GR32, 
    /* PUSH9 */
    GR32, 
    /* PUSHA */
    /* PUSHDATA1 */
    /* PUSHDATA2 */
    /* PUSHDATA4 */
    /* PUSHF */
    /* PUSHINT128 */
    /* PUSHINT16 */
    GR32, i32imm, 
    /* PUSHINT256 */
    /* PUSHINT32 */
    GR32, i32imm, 
    /* PUSHINT64 */
    GR32, i64imm, 
    /* PUSHINT8 */
    GR32, i32imm, 
    /* PUSHM1 */
    GR32, 
    /* PUSHNULL */
    /* PUSHT */
    /* REMOVE */
    /* RET */
    /* REVERSE3 */
    GR32, GR32, GR32, GR32, 
    /* REVERSE4 */
    GR32, GR32, GR32, GR32, GR32, 
    /* REVERSEITEMS */
    /* REVERSEN */
    GR32, GR32, 
    /* RIGHT */
    /* ROLL */
    GR32, GR32, GR32, 
    /* ROT */
    GR32, GR32, GR32, GR32, 
    /* SETITEM */
    /* SHL */
    /* SHR */
    /* SIGN */
    /* SIZE */
    /* SQRT */
    /* STARG */
    /* STARG0 */
    /* STARG1 */
    /* STARG2 */
    /* STARG3 */
    /* STARG4 */
    /* STARG5 */
    /* STARG6 */
    /* STLOC */
    /* STLOC0 */
    /* STLOC1 */
    /* STLOC2 */
    /* STLOC3 */
    /* STLOC4 */
    /* STLOC5 */
    /* STLOC6 */
    /* STSFLD */
    /* STSFLD0 */
    /* STSFLD1 */
    /* STSFLD2 */
    /* STSFLD3 */
    /* STSFLD4 */
    /* STSFLD5 */
    /* STSFLD6 */
    /* SUB */
    GR32, GR32, GR32, 
    /* SUBSTR */
    /* SWAP */
    GR32, GR32, GR32, 
    /* SYSCALL */
    i32imm, 
    /* THROW */
    /* TOALTSTACK */
    GR32, 
  };
  return OpcodeOperandTypes[Offsets[Opcode] + OpIdx];
}
} // end namespace NeoVM
} // end namespace llvm
#endif // GET_INSTRINFO_OPERAND_TYPE

#ifdef GET_INSTRINFO_MEM_OPERAND_SIZE
#undef GET_INSTRINFO_MEM_OPERAND_SIZE
namespace llvm {
namespace NeoVM {
LLVM_READONLY
static int getMemOperandSize(int OpType) {
  switch (OpType) {
  default: return 0;
  }
}
} // end namespace NeoVM
} // end namespace llvm
#endif // GET_INSTRINFO_MEM_OPERAND_SIZE

#ifdef GET_INSTRINFO_LOGICAL_OPERAND_SIZE_MAP
#undef GET_INSTRINFO_LOGICAL_OPERAND_SIZE_MAP
namespace llvm {
namespace NeoVM {
LLVM_READONLY static unsigned
getLogicalOperandSize(uint16_t Opcode, uint16_t LogicalOpIdx) {
  return LogicalOpIdx;
}
LLVM_READONLY static inline unsigned
getLogicalOperandIdx(uint16_t Opcode, uint16_t LogicalOpIdx) {
  auto S = 0U;
  for (auto i = 0U; i < LogicalOpIdx; ++i)
    S += getLogicalOperandSize(Opcode, i);
  return S;
}
} // end namespace NeoVM
} // end namespace llvm
#endif // GET_INSTRINFO_LOGICAL_OPERAND_SIZE_MAP

#ifdef GET_INSTRINFO_LOGICAL_OPERAND_TYPE_MAP
#undef GET_INSTRINFO_LOGICAL_OPERAND_TYPE_MAP
namespace llvm {
namespace NeoVM {
LLVM_READONLY static int
getLogicalOperandType(uint16_t Opcode, uint16_t LogicalOpIdx) {
  return -1;
}
} // end namespace NeoVM
} // end namespace llvm
#endif // GET_INSTRINFO_LOGICAL_OPERAND_TYPE_MAP

#ifdef GET_INSTRINFO_MC_HELPER_DECLS
#undef GET_INSTRINFO_MC_HELPER_DECLS

namespace llvm {
class MCInst;
class FeatureBitset;

namespace NeoVM_MC {

void verifyInstructionPredicates(unsigned Opcode, const FeatureBitset &Features);

} // end namespace NeoVM_MC
} // end namespace llvm

#endif // GET_INSTRINFO_MC_HELPER_DECLS

#ifdef GET_INSTRINFO_MC_HELPERS
#undef GET_INSTRINFO_MC_HELPERS

namespace llvm {
namespace NeoVM_MC {

} // end namespace NeoVM_MC
} // end namespace llvm

#endif // GET_GENISTRINFO_MC_HELPERS

#if (defined(ENABLE_INSTR_PREDICATE_VERIFIER) && !defined(NDEBUG)) ||\
    defined(GET_AVAILABLE_OPCODE_CHECKER)
#define GET_COMPUTE_FEATURES
#endif
#ifdef GET_COMPUTE_FEATURES
#undef GET_COMPUTE_FEATURES
namespace llvm {
namespace NeoVM_MC {

// Bits for subtarget features that participate in instruction matching.
enum SubtargetFeatureBits : uint8_t {
};

inline FeatureBitset computeAvailableFeatures(const FeatureBitset &FB) {
  FeatureBitset Features;
  return Features;
}

inline FeatureBitset computeRequiredFeatures(unsigned Opcode) {
  enum : uint8_t {
    CEFBS_None,
  };

  static constexpr FeatureBitset FeatureBitsets[] = {
    {}, // CEFBS_None
  };
  static constexpr uint8_t RequiredFeaturesRefs[] = {
    CEFBS_None, // PHI = 0
    CEFBS_None, // INLINEASM = 1
    CEFBS_None, // INLINEASM_BR = 2
    CEFBS_None, // CFI_INSTRUCTION = 3
    CEFBS_None, // EH_LABEL = 4
    CEFBS_None, // GC_LABEL = 5
    CEFBS_None, // ANNOTATION_LABEL = 6
    CEFBS_None, // KILL = 7
    CEFBS_None, // EXTRACT_SUBREG = 8
    CEFBS_None, // INSERT_SUBREG = 9
    CEFBS_None, // IMPLICIT_DEF = 10
    CEFBS_None, // SUBREG_TO_REG = 11
    CEFBS_None, // COPY_TO_REGCLASS = 12
    CEFBS_None, // DBG_VALUE = 13
    CEFBS_None, // DBG_VALUE_LIST = 14
    CEFBS_None, // DBG_INSTR_REF = 15
    CEFBS_None, // DBG_PHI = 16
    CEFBS_None, // DBG_LABEL = 17
    CEFBS_None, // REG_SEQUENCE = 18
    CEFBS_None, // COPY = 19
    CEFBS_None, // BUNDLE = 20
    CEFBS_None, // LIFETIME_START = 21
    CEFBS_None, // LIFETIME_END = 22
    CEFBS_None, // PSEUDO_PROBE = 23
    CEFBS_None, // ARITH_FENCE = 24
    CEFBS_None, // STACKMAP = 25
    CEFBS_None, // FENTRY_CALL = 26
    CEFBS_None, // PATCHPOINT = 27
    CEFBS_None, // LOAD_STACK_GUARD = 28
    CEFBS_None, // PREALLOCATED_SETUP = 29
    CEFBS_None, // PREALLOCATED_ARG = 30
    CEFBS_None, // STATEPOINT = 31
    CEFBS_None, // LOCAL_ESCAPE = 32
    CEFBS_None, // FAULTING_OP = 33
    CEFBS_None, // PATCHABLE_OP = 34
    CEFBS_None, // PATCHABLE_FUNCTION_ENTER = 35
    CEFBS_None, // PATCHABLE_RET = 36
    CEFBS_None, // PATCHABLE_FUNCTION_EXIT = 37
    CEFBS_None, // PATCHABLE_TAIL_CALL = 38
    CEFBS_None, // PATCHABLE_EVENT_CALL = 39
    CEFBS_None, // PATCHABLE_TYPED_EVENT_CALL = 40
    CEFBS_None, // ICALL_BRANCH_FUNNEL = 41
    CEFBS_None, // MEMBARRIER = 42
    CEFBS_None, // JUMP_TABLE_DEBUG_INFO = 43
    CEFBS_None, // G_ASSERT_SEXT = 44
    CEFBS_None, // G_ASSERT_ZEXT = 45
    CEFBS_None, // G_ASSERT_ALIGN = 46
    CEFBS_None, // G_ADD = 47
    CEFBS_None, // G_SUB = 48
    CEFBS_None, // G_MUL = 49
    CEFBS_None, // G_SDIV = 50
    CEFBS_None, // G_UDIV = 51
    CEFBS_None, // G_SREM = 52
    CEFBS_None, // G_UREM = 53
    CEFBS_None, // G_SDIVREM = 54
    CEFBS_None, // G_UDIVREM = 55
    CEFBS_None, // G_AND = 56
    CEFBS_None, // G_OR = 57
    CEFBS_None, // G_XOR = 58
    CEFBS_None, // G_IMPLICIT_DEF = 59
    CEFBS_None, // G_PHI = 60
    CEFBS_None, // G_FRAME_INDEX = 61
    CEFBS_None, // G_GLOBAL_VALUE = 62
    CEFBS_None, // G_CONSTANT_POOL = 63
    CEFBS_None, // G_EXTRACT = 64
    CEFBS_None, // G_UNMERGE_VALUES = 65
    CEFBS_None, // G_INSERT = 66
    CEFBS_None, // G_MERGE_VALUES = 67
    CEFBS_None, // G_BUILD_VECTOR = 68
    CEFBS_None, // G_BUILD_VECTOR_TRUNC = 69
    CEFBS_None, // G_CONCAT_VECTORS = 70
    CEFBS_None, // G_PTRTOINT = 71
    CEFBS_None, // G_INTTOPTR = 72
    CEFBS_None, // G_BITCAST = 73
    CEFBS_None, // G_FREEZE = 74
    CEFBS_None, // G_CONSTANT_FOLD_BARRIER = 75
    CEFBS_None, // G_INTRINSIC_FPTRUNC_ROUND = 76
    CEFBS_None, // G_INTRINSIC_TRUNC = 77
    CEFBS_None, // G_INTRINSIC_ROUND = 78
    CEFBS_None, // G_INTRINSIC_LRINT = 79
    CEFBS_None, // G_INTRINSIC_ROUNDEVEN = 80
    CEFBS_None, // G_READCYCLECOUNTER = 81
    CEFBS_None, // G_LOAD = 82
    CEFBS_None, // G_SEXTLOAD = 83
    CEFBS_None, // G_ZEXTLOAD = 84
    CEFBS_None, // G_INDEXED_LOAD = 85
    CEFBS_None, // G_INDEXED_SEXTLOAD = 86
    CEFBS_None, // G_INDEXED_ZEXTLOAD = 87
    CEFBS_None, // G_STORE = 88
    CEFBS_None, // G_INDEXED_STORE = 89
    CEFBS_None, // G_ATOMIC_CMPXCHG_WITH_SUCCESS = 90
    CEFBS_None, // G_ATOMIC_CMPXCHG = 91
    CEFBS_None, // G_ATOMICRMW_XCHG = 92
    CEFBS_None, // G_ATOMICRMW_ADD = 93
    CEFBS_None, // G_ATOMICRMW_SUB = 94
    CEFBS_None, // G_ATOMICRMW_AND = 95
    CEFBS_None, // G_ATOMICRMW_NAND = 96
    CEFBS_None, // G_ATOMICRMW_OR = 97
    CEFBS_None, // G_ATOMICRMW_XOR = 98
    CEFBS_None, // G_ATOMICRMW_MAX = 99
    CEFBS_None, // G_ATOMICRMW_MIN = 100
    CEFBS_None, // G_ATOMICRMW_UMAX = 101
    CEFBS_None, // G_ATOMICRMW_UMIN = 102
    CEFBS_None, // G_ATOMICRMW_FADD = 103
    CEFBS_None, // G_ATOMICRMW_FSUB = 104
    CEFBS_None, // G_ATOMICRMW_FMAX = 105
    CEFBS_None, // G_ATOMICRMW_FMIN = 106
    CEFBS_None, // G_ATOMICRMW_UINC_WRAP = 107
    CEFBS_None, // G_ATOMICRMW_UDEC_WRAP = 108
    CEFBS_None, // G_FENCE = 109
    CEFBS_None, // G_PREFETCH = 110
    CEFBS_None, // G_BRCOND = 111
    CEFBS_None, // G_BRINDIRECT = 112
    CEFBS_None, // G_INVOKE_REGION_START = 113
    CEFBS_None, // G_INTRINSIC = 114
    CEFBS_None, // G_INTRINSIC_W_SIDE_EFFECTS = 115
    CEFBS_None, // G_INTRINSIC_CONVERGENT = 116
    CEFBS_None, // G_INTRINSIC_CONVERGENT_W_SIDE_EFFECTS = 117
    CEFBS_None, // G_ANYEXT = 118
    CEFBS_None, // G_TRUNC = 119
    CEFBS_None, // G_CONSTANT = 120
    CEFBS_None, // G_FCONSTANT = 121
    CEFBS_None, // G_VASTART = 122
    CEFBS_None, // G_VAARG = 123
    CEFBS_None, // G_SEXT = 124
    CEFBS_None, // G_SEXT_INREG = 125
    CEFBS_None, // G_ZEXT = 126
    CEFBS_None, // G_SHL = 127
    CEFBS_None, // G_LSHR = 128
    CEFBS_None, // G_ASHR = 129
    CEFBS_None, // G_FSHL = 130
    CEFBS_None, // G_FSHR = 131
    CEFBS_None, // G_ROTR = 132
    CEFBS_None, // G_ROTL = 133
    CEFBS_None, // G_ICMP = 134
    CEFBS_None, // G_FCMP = 135
    CEFBS_None, // G_SELECT = 136
    CEFBS_None, // G_UADDO = 137
    CEFBS_None, // G_UADDE = 138
    CEFBS_None, // G_USUBO = 139
    CEFBS_None, // G_USUBE = 140
    CEFBS_None, // G_SADDO = 141
    CEFBS_None, // G_SADDE = 142
    CEFBS_None, // G_SSUBO = 143
    CEFBS_None, // G_SSUBE = 144
    CEFBS_None, // G_UMULO = 145
    CEFBS_None, // G_SMULO = 146
    CEFBS_None, // G_UMULH = 147
    CEFBS_None, // G_SMULH = 148
    CEFBS_None, // G_UADDSAT = 149
    CEFBS_None, // G_SADDSAT = 150
    CEFBS_None, // G_USUBSAT = 151
    CEFBS_None, // G_SSUBSAT = 152
    CEFBS_None, // G_USHLSAT = 153
    CEFBS_None, // G_SSHLSAT = 154
    CEFBS_None, // G_SMULFIX = 155
    CEFBS_None, // G_UMULFIX = 156
    CEFBS_None, // G_SMULFIXSAT = 157
    CEFBS_None, // G_UMULFIXSAT = 158
    CEFBS_None, // G_SDIVFIX = 159
    CEFBS_None, // G_UDIVFIX = 160
    CEFBS_None, // G_SDIVFIXSAT = 161
    CEFBS_None, // G_UDIVFIXSAT = 162
    CEFBS_None, // G_FADD = 163
    CEFBS_None, // G_FSUB = 164
    CEFBS_None, // G_FMUL = 165
    CEFBS_None, // G_FMA = 166
    CEFBS_None, // G_FMAD = 167
    CEFBS_None, // G_FDIV = 168
    CEFBS_None, // G_FREM = 169
    CEFBS_None, // G_FPOW = 170
    CEFBS_None, // G_FPOWI = 171
    CEFBS_None, // G_FEXP = 172
    CEFBS_None, // G_FEXP2 = 173
    CEFBS_None, // G_FEXP10 = 174
    CEFBS_None, // G_FLOG = 175
    CEFBS_None, // G_FLOG2 = 176
    CEFBS_None, // G_FLOG10 = 177
    CEFBS_None, // G_FLDEXP = 178
    CEFBS_None, // G_FFREXP = 179
    CEFBS_None, // G_FNEG = 180
    CEFBS_None, // G_FPEXT = 181
    CEFBS_None, // G_FPTRUNC = 182
    CEFBS_None, // G_FPTOSI = 183
    CEFBS_None, // G_FPTOUI = 184
    CEFBS_None, // G_SITOFP = 185
    CEFBS_None, // G_UITOFP = 186
    CEFBS_None, // G_FABS = 187
    CEFBS_None, // G_FCOPYSIGN = 188
    CEFBS_None, // G_IS_FPCLASS = 189
    CEFBS_None, // G_FCANONICALIZE = 190
    CEFBS_None, // G_FMINNUM = 191
    CEFBS_None, // G_FMAXNUM = 192
    CEFBS_None, // G_FMINNUM_IEEE = 193
    CEFBS_None, // G_FMAXNUM_IEEE = 194
    CEFBS_None, // G_FMINIMUM = 195
    CEFBS_None, // G_FMAXIMUM = 196
    CEFBS_None, // G_GET_FPENV = 197
    CEFBS_None, // G_SET_FPENV = 198
    CEFBS_None, // G_RESET_FPENV = 199
    CEFBS_None, // G_GET_FPMODE = 200
    CEFBS_None, // G_SET_FPMODE = 201
    CEFBS_None, // G_RESET_FPMODE = 202
    CEFBS_None, // G_PTR_ADD = 203
    CEFBS_None, // G_PTRMASK = 204
    CEFBS_None, // G_SMIN = 205
    CEFBS_None, // G_SMAX = 206
    CEFBS_None, // G_UMIN = 207
    CEFBS_None, // G_UMAX = 208
    CEFBS_None, // G_ABS = 209
    CEFBS_None, // G_LROUND = 210
    CEFBS_None, // G_LLROUND = 211
    CEFBS_None, // G_BR = 212
    CEFBS_None, // G_BRJT = 213
    CEFBS_None, // G_INSERT_VECTOR_ELT = 214
    CEFBS_None, // G_EXTRACT_VECTOR_ELT = 215
    CEFBS_None, // G_SHUFFLE_VECTOR = 216
    CEFBS_None, // G_CTTZ = 217
    CEFBS_None, // G_CTTZ_ZERO_UNDEF = 218
    CEFBS_None, // G_CTLZ = 219
    CEFBS_None, // G_CTLZ_ZERO_UNDEF = 220
    CEFBS_None, // G_CTPOP = 221
    CEFBS_None, // G_BSWAP = 222
    CEFBS_None, // G_BITREVERSE = 223
    CEFBS_None, // G_FCEIL = 224
    CEFBS_None, // G_FCOS = 225
    CEFBS_None, // G_FSIN = 226
    CEFBS_None, // G_FSQRT = 227
    CEFBS_None, // G_FFLOOR = 228
    CEFBS_None, // G_FRINT = 229
    CEFBS_None, // G_FNEARBYINT = 230
    CEFBS_None, // G_ADDRSPACE_CAST = 231
    CEFBS_None, // G_BLOCK_ADDR = 232
    CEFBS_None, // G_JUMP_TABLE = 233
    CEFBS_None, // G_DYN_STACKALLOC = 234
    CEFBS_None, // G_STACKSAVE = 235
    CEFBS_None, // G_STACKRESTORE = 236
    CEFBS_None, // G_STRICT_FADD = 237
    CEFBS_None, // G_STRICT_FSUB = 238
    CEFBS_None, // G_STRICT_FMUL = 239
    CEFBS_None, // G_STRICT_FDIV = 240
    CEFBS_None, // G_STRICT_FREM = 241
    CEFBS_None, // G_STRICT_FMA = 242
    CEFBS_None, // G_STRICT_FSQRT = 243
    CEFBS_None, // G_STRICT_FLDEXP = 244
    CEFBS_None, // G_READ_REGISTER = 245
    CEFBS_None, // G_WRITE_REGISTER = 246
    CEFBS_None, // G_MEMCPY = 247
    CEFBS_None, // G_MEMCPY_INLINE = 248
    CEFBS_None, // G_MEMMOVE = 249
    CEFBS_None, // G_MEMSET = 250
    CEFBS_None, // G_BZERO = 251
    CEFBS_None, // G_VECREDUCE_SEQ_FADD = 252
    CEFBS_None, // G_VECREDUCE_SEQ_FMUL = 253
    CEFBS_None, // G_VECREDUCE_FADD = 254
    CEFBS_None, // G_VECREDUCE_FMUL = 255
    CEFBS_None, // G_VECREDUCE_FMAX = 256
    CEFBS_None, // G_VECREDUCE_FMIN = 257
    CEFBS_None, // G_VECREDUCE_FMAXIMUM = 258
    CEFBS_None, // G_VECREDUCE_FMINIMUM = 259
    CEFBS_None, // G_VECREDUCE_ADD = 260
    CEFBS_None, // G_VECREDUCE_MUL = 261
    CEFBS_None, // G_VECREDUCE_AND = 262
    CEFBS_None, // G_VECREDUCE_OR = 263
    CEFBS_None, // G_VECREDUCE_XOR = 264
    CEFBS_None, // G_VECREDUCE_SMAX = 265
    CEFBS_None, // G_VECREDUCE_SMIN = 266
    CEFBS_None, // G_VECREDUCE_UMAX = 267
    CEFBS_None, // G_VECREDUCE_UMIN = 268
    CEFBS_None, // G_SBFX = 269
    CEFBS_None, // G_UBFX = 270
    CEFBS_None, // ABORT = 271
    CEFBS_None, // ABORTMSG = 272
    CEFBS_None, // ABS = 273
    CEFBS_None, // ADD = 274
    CEFBS_None, // AND = 275
    CEFBS_None, // APPEND = 276
    CEFBS_None, // ASSERT = 277
    CEFBS_None, // ASSERTMSG = 278
    CEFBS_None, // BOOLAND = 279
    CEFBS_None, // BOOLOR = 280
    CEFBS_None, // CALL = 281
    CEFBS_None, // CALLA = 282
    CEFBS_None, // CALLT = 283
    CEFBS_None, // CALL_L = 284
    CEFBS_None, // CAT = 285
    CEFBS_None, // CLEAR = 286
    CEFBS_None, // CLEARITEMS = 287
    CEFBS_None, // CONVERT = 288
    CEFBS_None, // DEC = 289
    CEFBS_None, // DEPTH = 290
    CEFBS_None, // DIV = 291
    CEFBS_None, // DROP = 292
    CEFBS_None, // DROPN = 293
    CEFBS_None, // DUP = 294
    CEFBS_None, // DUPFROMALTSTACK = 295
    CEFBS_None, // ENDFINALLY = 296
    CEFBS_None, // ENDTRY = 297
    CEFBS_None, // ENDTRY_L = 298
    CEFBS_None, // EQ = 299
    CEFBS_None, // EQUAL = 300
    CEFBS_None, // FROMALTSTACK = 301
    CEFBS_None, // GE = 302
    CEFBS_None, // GT = 303
    CEFBS_None, // HASKEY = 304
    CEFBS_None, // INC = 305
    CEFBS_None, // INITSLOT = 306
    CEFBS_None, // INITSSLOT = 307
    CEFBS_None, // INVERT = 308
    CEFBS_None, // ISNULL = 309
    CEFBS_None, // ISTYPE = 310
    CEFBS_None, // JMP = 311
    CEFBS_None, // JMPEQ = 312
    CEFBS_None, // JMPEQ_L = 313
    CEFBS_None, // JMPGE = 314
    CEFBS_None, // JMPGE_L = 315
    CEFBS_None, // JMPGT = 316
    CEFBS_None, // JMPGT_L = 317
    CEFBS_None, // JMPIF = 318
    CEFBS_None, // JMPIFNOT = 319
    CEFBS_None, // JMPIFNOT_L = 320
    CEFBS_None, // JMPIF_L = 321
    CEFBS_None, // JMPLE = 322
    CEFBS_None, // JMPLE_L = 323
    CEFBS_None, // JMPLT = 324
    CEFBS_None, // JMPLT_L = 325
    CEFBS_None, // JMPNE = 326
    CEFBS_None, // JMPNE_L = 327
    CEFBS_None, // JMP_L = 328
    CEFBS_None, // KEYS = 329
    CEFBS_None, // LDARG = 330
    CEFBS_None, // LDARG0 = 331
    CEFBS_None, // LDARG1 = 332
    CEFBS_None, // LDARG2 = 333
    CEFBS_None, // LDARG3 = 334
    CEFBS_None, // LDARG4 = 335
    CEFBS_None, // LDARG5 = 336
    CEFBS_None, // LDARG6 = 337
    CEFBS_None, // LDLOC = 338
    CEFBS_None, // LDLOC0 = 339
    CEFBS_None, // LDLOC1 = 340
    CEFBS_None, // LDLOC2 = 341
    CEFBS_None, // LDLOC3 = 342
    CEFBS_None, // LDLOC4 = 343
    CEFBS_None, // LDLOC5 = 344
    CEFBS_None, // LDLOC6 = 345
    CEFBS_None, // LDSFLD = 346
    CEFBS_None, // LDSFLD0 = 347
    CEFBS_None, // LDSFLD1 = 348
    CEFBS_None, // LDSFLD2 = 349
    CEFBS_None, // LDSFLD3 = 350
    CEFBS_None, // LDSFLD4 = 351
    CEFBS_None, // LDSFLD5 = 352
    CEFBS_None, // LDSFLD6 = 353
    CEFBS_None, // LE = 354
    CEFBS_None, // LEFT = 355
    CEFBS_None, // LT = 356
    CEFBS_None, // MAX = 357
    CEFBS_None, // MEMCPY = 358
    CEFBS_None, // MIN = 359
    CEFBS_None, // MOD = 360
    CEFBS_None, // MODMUL = 361
    CEFBS_None, // MODPOW = 362
    CEFBS_None, // MUL = 363
    CEFBS_None, // NE = 364
    CEFBS_None, // NEGATE = 365
    CEFBS_None, // NEOVM_LOAD_PSEUDO = 366
    CEFBS_None, // NEOVM_STORE_PSEUDO = 367
    CEFBS_None, // NEWARRAY = 368
    CEFBS_None, // NEWARRAY0 = 369
    CEFBS_None, // NEWARRAY_T = 370
    CEFBS_None, // NEWBUFFER = 371
    CEFBS_None, // NEWMAP = 372
    CEFBS_None, // NEWSTRUCT = 373
    CEFBS_None, // NEWSTRUCT0 = 374
    CEFBS_None, // NIP = 375
    CEFBS_None, // NOP = 376
    CEFBS_None, // NOT = 377
    CEFBS_None, // NOTEQUAL = 378
    CEFBS_None, // NUMEQUAL = 379
    CEFBS_None, // NUMNOTEQUAL = 380
    CEFBS_None, // NZ = 381
    CEFBS_None, // OR = 382
    CEFBS_None, // OVER = 383
    CEFBS_None, // PACK = 384
    CEFBS_None, // PACKMAP = 385
    CEFBS_None, // PACKSTRUCT = 386
    CEFBS_None, // PICK = 387
    CEFBS_None, // PICKITEM = 388
    CEFBS_None, // POP = 389
    CEFBS_None, // POPITEM = 390
    CEFBS_None, // POW = 391
    CEFBS_None, // PUSH = 392
    CEFBS_None, // PUSH0 = 393
    CEFBS_None, // PUSH1 = 394
    CEFBS_None, // PUSH10 = 395
    CEFBS_None, // PUSH11 = 396
    CEFBS_None, // PUSH12 = 397
    CEFBS_None, // PUSH13 = 398
    CEFBS_None, // PUSH14 = 399
    CEFBS_None, // PUSH15 = 400
    CEFBS_None, // PUSH16 = 401
    CEFBS_None, // PUSH2 = 402
    CEFBS_None, // PUSH3 = 403
    CEFBS_None, // PUSH4 = 404
    CEFBS_None, // PUSH5 = 405
    CEFBS_None, // PUSH6 = 406
    CEFBS_None, // PUSH7 = 407
    CEFBS_None, // PUSH8 = 408
    CEFBS_None, // PUSH9 = 409
    CEFBS_None, // PUSHA = 410
    CEFBS_None, // PUSHDATA1 = 411
    CEFBS_None, // PUSHDATA2 = 412
    CEFBS_None, // PUSHDATA4 = 413
    CEFBS_None, // PUSHF = 414
    CEFBS_None, // PUSHINT128 = 415
    CEFBS_None, // PUSHINT16 = 416
    CEFBS_None, // PUSHINT256 = 417
    CEFBS_None, // PUSHINT32 = 418
    CEFBS_None, // PUSHINT64 = 419
    CEFBS_None, // PUSHINT8 = 420
    CEFBS_None, // PUSHM1 = 421
    CEFBS_None, // PUSHNULL = 422
    CEFBS_None, // PUSHT = 423
    CEFBS_None, // REMOVE = 424
    CEFBS_None, // RET = 425
    CEFBS_None, // REVERSE3 = 426
    CEFBS_None, // REVERSE4 = 427
    CEFBS_None, // REVERSEITEMS = 428
    CEFBS_None, // REVERSEN = 429
    CEFBS_None, // RIGHT = 430
    CEFBS_None, // ROLL = 431
    CEFBS_None, // ROT = 432
    CEFBS_None, // SETITEM = 433
    CEFBS_None, // SHL = 434
    CEFBS_None, // SHR = 435
    CEFBS_None, // SIGN = 436
    CEFBS_None, // SIZE = 437
    CEFBS_None, // SQRT = 438
    CEFBS_None, // STARG = 439
    CEFBS_None, // STARG0 = 440
    CEFBS_None, // STARG1 = 441
    CEFBS_None, // STARG2 = 442
    CEFBS_None, // STARG3 = 443
    CEFBS_None, // STARG4 = 444
    CEFBS_None, // STARG5 = 445
    CEFBS_None, // STARG6 = 446
    CEFBS_None, // STLOC = 447
    CEFBS_None, // STLOC0 = 448
    CEFBS_None, // STLOC1 = 449
    CEFBS_None, // STLOC2 = 450
    CEFBS_None, // STLOC3 = 451
    CEFBS_None, // STLOC4 = 452
    CEFBS_None, // STLOC5 = 453
    CEFBS_None, // STLOC6 = 454
    CEFBS_None, // STSFLD = 455
    CEFBS_None, // STSFLD0 = 456
    CEFBS_None, // STSFLD1 = 457
    CEFBS_None, // STSFLD2 = 458
    CEFBS_None, // STSFLD3 = 459
    CEFBS_None, // STSFLD4 = 460
    CEFBS_None, // STSFLD5 = 461
    CEFBS_None, // STSFLD6 = 462
    CEFBS_None, // SUB = 463
    CEFBS_None, // SUBSTR = 464
    CEFBS_None, // SWAP = 465
    CEFBS_None, // SYSCALL = 466
    CEFBS_None, // THROW = 467
    CEFBS_None, // TOALTSTACK = 468
    CEFBS_None, // TRY = 469
    CEFBS_None, // TRY_L = 470
    CEFBS_None, // TUCK = 471
    CEFBS_None, // UNPACK = 472
    CEFBS_None, // VALUES = 473
    CEFBS_None, // WITHIN = 474
    CEFBS_None, // XDROP = 475
    CEFBS_None, // XOR = 476
  };

  assert(Opcode < 477);
  return FeatureBitsets[RequiredFeaturesRefs[Opcode]];
}

} // end namespace NeoVM_MC
} // end namespace llvm
#endif // GET_COMPUTE_FEATURES

#ifdef GET_AVAILABLE_OPCODE_CHECKER
#undef GET_AVAILABLE_OPCODE_CHECKER
namespace llvm {
namespace NeoVM_MC {
bool isOpcodeAvailable(unsigned Opcode, const FeatureBitset &Features) {
  FeatureBitset AvailableFeatures = computeAvailableFeatures(Features);
  FeatureBitset RequiredFeatures = computeRequiredFeatures(Opcode);
  FeatureBitset MissingFeatures =
      (AvailableFeatures & RequiredFeatures) ^
      RequiredFeatures;
  return !MissingFeatures.any();
}
} // end namespace NeoVM_MC
} // end namespace llvm
#endif // GET_AVAILABLE_OPCODE_CHECKER

#ifdef ENABLE_INSTR_PREDICATE_VERIFIER
#undef ENABLE_INSTR_PREDICATE_VERIFIER
#include <sstream>

namespace llvm {
namespace NeoVM_MC {

#ifndef NDEBUG
static const char *SubtargetFeatureNames[] = {
  nullptr
};

#endif // NDEBUG

void verifyInstructionPredicates(
    unsigned Opcode, const FeatureBitset &Features) {
#ifndef NDEBUG
  FeatureBitset AvailableFeatures = computeAvailableFeatures(Features);
  FeatureBitset RequiredFeatures = computeRequiredFeatures(Opcode);
  FeatureBitset MissingFeatures =
      (AvailableFeatures & RequiredFeatures) ^
      RequiredFeatures;
  if (MissingFeatures.any()) {
    std::ostringstream Msg;
    Msg << "Attempting to emit " << &NeoVMInstrNameData[NeoVMInstrNameIndices[Opcode]]
        << " instruction but the ";
    for (unsigned i = 0, e = MissingFeatures.size(); i != e; ++i)
      if (MissingFeatures.test(i))
        Msg << SubtargetFeatureNames[i] << " ";
    Msg << "predicate(s) are not met";
    report_fatal_error(Msg.str().c_str());
  }
#endif // NDEBUG
}
} // end namespace NeoVM_MC
} // end namespace llvm
#endif // ENABLE_INSTR_PREDICATE_VERIFIER

