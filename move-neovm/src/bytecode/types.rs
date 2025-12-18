//! Move bytecode type definitions.

/// Move bytecode version
#[derive(Debug, Clone, Copy)]
pub struct BytecodeVersion(pub u32);

/// A parsed Move module
#[derive(Debug, Clone)]
pub struct MoveModule {
    /// Module version
    pub version: BytecodeVersion,
    /// Module name
    pub name: String,
    /// Identifier table offset
    pub identifiers_offset: u32,
    /// Identifier table count
    pub identifiers_count: u32,
    /// Struct table offset
    pub struct_defs_offset: u32,
    /// Struct table count
    pub struct_defs_count: u32,
    /// Function handle offset (unused placeholder)
    pub _function_handles_offset: u32,
    /// Function handle count (unused placeholder)
    pub _function_handles_count: u32,
    /// Function table offset
    pub function_defs_offset: u32,
    /// Function table count
    pub function_defs_count: u32,
    /// Struct definitions
    pub structs: Vec<StructDef>,
    /// Function definitions
    pub functions: Vec<FunctionDef>,
}

/// Ability flags attached to a struct definition
#[derive(Debug, Clone, Copy, Default)]
pub struct AbilitySet {
    pub copy: bool,
    pub drop: bool,
    pub store: bool,
    pub key: bool,
}

impl AbilitySet {
    pub fn is_resource(&self) -> bool {
        self.key
    }
}

/// A struct (or resource) definition
#[derive(Debug, Clone)]
pub struct StructDef {
    /// Struct name (fully-qualified or simple)
    pub name: String,
    /// Ability set declared on the struct
    pub abilities: AbilitySet,
    /// Field definitions
    pub fields: Vec<FieldDef>,
}

/// A field definition
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// Field name
    pub name: String,
    /// Field type
    pub type_tag: TypeTag,
}

/// A function definition
#[derive(Debug, Clone)]
pub struct FunctionDef {
    /// Function name
    pub name: String,
    /// Is this a public function?
    pub is_public: bool,
    /// Is this an entry function?
    pub is_entry: bool,
    /// Parameter types
    pub parameters: Vec<TypeTag>,
    /// Return types
    pub returns: Vec<TypeTag>,
    /// Local slot layout (parameters are always the first locals)
    pub locals: Vec<TypeTag>,
    /// Function body (opcodes)
    pub code: Vec<MoveOpcode>,
}

/// Move type tags
#[derive(Debug, Clone)]
pub enum TypeTag {
    Bool,
    U8,
    U64,
    U128,
    U256,
    Address,
    Signer,
    Vector(Box<TypeTag>),
    Struct(String),
    Reference(Box<TypeTag>),
    MutableReference(Box<TypeTag>),
}

impl TypeTag {
    /// Convert to WASM-compatible representation
    pub fn to_wasm_type(&self) -> &'static str {
        match self {
            TypeTag::Bool => "i32",
            TypeTag::U8 => "i32",
            TypeTag::U64 => "i64",
            TypeTag::U128 => "i64",      // Requires multi-word handling
            TypeTag::U256 => "i64",      // Requires multi-word handling
            TypeTag::Address => "i32",   // Pointer to 32-byte array
            TypeTag::Signer => "i32",    // Pointer to signer data
            TypeTag::Vector(_) => "i32", // Pointer to vector
            TypeTag::Struct(_) => "i32", // Pointer to struct
            TypeTag::Reference(_) => "i32",
            TypeTag::MutableReference(_) => "i32",
        }
    }
}

/// Move VM opcodes (simplified subset)
#[derive(Debug, Clone)]
pub enum MoveOpcode {
    // Constants
    LdU8(u8),
    LdU64(u64),
    LdU128(u128),
    LdTrue,
    LdFalse,
    LdConst(u16), // Constant pool index

    // Local operations
    CopyLoc(u8),
    MoveLoc(u8),
    StLoc(u8),
    MutBorrowLoc(u8),
    ImmBorrowLoc(u8),

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Neq,

    // Logical
    And,
    Or,
    Not,

    // Control flow
    Branch(u16),
    BrTrue(u16),
    BrFalse(u16),
    Call(u16), // Function index
    Ret,
    Abort,

    // Resource operations
    Pack(u16), // Struct index
    Unpack(u16),
    BorrowField(u16),
    MutBorrowField(u16),
    MoveFrom(u16), // Struct index
    MoveTo(u16),
    Exists(u16),
    BorrowGlobal(u16),
    MutBorrowGlobal(u16),

    // Stack
    Pop,

    // Vector operations
    VecPack(u16, u64),
    VecLen(u16),
    VecImmBorrow(u16),
    VecMutBorrow(u16),
    VecPushBack(u16),
    VecPopBack(u16),

    // Casting
    CastU8,
    CastU64,
    CastU128,

    // Nop (placeholder for unsupported)
    Nop,
}

impl MoveOpcode {
    /// Get opcode byte value
    pub fn opcode_byte(&self) -> u8 {
        match self {
            MoveOpcode::Pop => 0x01,
            MoveOpcode::Ret => 0x02,
            MoveOpcode::BrTrue(_) => 0x03,
            MoveOpcode::BrFalse(_) => 0x04,
            MoveOpcode::Branch(_) => 0x05,
            MoveOpcode::LdU8(_) => 0x06,
            MoveOpcode::LdU64(_) => 0x07,
            MoveOpcode::LdU128(_) => 0x08,
            MoveOpcode::CastU8 => 0x09,
            MoveOpcode::CastU64 => 0x0A,
            MoveOpcode::CastU128 => 0x0B,
            MoveOpcode::LdConst(_) => 0x0C,
            MoveOpcode::LdTrue => 0x0D,
            MoveOpcode::LdFalse => 0x0E,
            MoveOpcode::CopyLoc(_) => 0x0F,
            MoveOpcode::MoveLoc(_) => 0x10,
            MoveOpcode::StLoc(_) => 0x11,
            MoveOpcode::MutBorrowLoc(_) => 0x12,
            MoveOpcode::ImmBorrowLoc(_) => 0x13,
            MoveOpcode::MutBorrowField(_) => 0x14,
            MoveOpcode::BorrowField(_) => 0x15,
            MoveOpcode::Call(_) => 0x16,
            MoveOpcode::Pack(_) => 0x17,
            MoveOpcode::Unpack(_) => 0x18,
            MoveOpcode::Add => 0x22,
            MoveOpcode::Sub => 0x23,
            MoveOpcode::Mul => 0x24,
            MoveOpcode::Mod => 0x25,
            MoveOpcode::Div => 0x26,
            MoveOpcode::Lt => 0x32,
            MoveOpcode::Gt => 0x33,
            MoveOpcode::Le => 0x34,
            MoveOpcode::Ge => 0x35,
            MoveOpcode::And => 0x40,
            MoveOpcode::Or => 0x41,
            MoveOpcode::Not => 0x42,
            MoveOpcode::Eq => 0x43,
            MoveOpcode::Neq => 0x44,
            MoveOpcode::Abort => 0x45,
            MoveOpcode::Exists(_) => 0x50,
            MoveOpcode::BorrowGlobal(_) => 0x51,
            MoveOpcode::MutBorrowGlobal(_) => 0x52,
            MoveOpcode::MoveFrom(_) => 0x53,
            MoveOpcode::MoveTo(_) => 0x54,
            MoveOpcode::VecPack(_, _) => 0x60,
            MoveOpcode::VecLen(_) => 0x61,
            MoveOpcode::VecImmBorrow(_) => 0x62,
            MoveOpcode::VecMutBorrow(_) => 0x63,
            MoveOpcode::VecPushBack(_) => 0x64,
            MoveOpcode::VecPopBack(_) => 0x65,
            MoveOpcode::Nop => 0x00,
        }
    }
}
