//! Runtime helper bookkeeping types.
//!
//! These structs/enums track which NeoVM helper routines have been requested
//! during translation so `RuntimeHelpers::finalize` can emit the helper bodies
//! once and patch all call sites.

#[derive(Clone, Default)]
pub(crate) struct MemoryConfig {
    pub(super) initial_pages: u32,
    pub(super) maximum_pages: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MemoryHelperKind {
    Load(u32),
    Store(u32),
    Grow,
    Fill,
    Copy,
    EnvMemcpy,
    EnvMemmove,
    EnvMemset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BitHelperKind {
    Clz(u32),
    Ctz(u32),
    Popcnt(u32),
}

impl BitHelperKind {
    pub(crate) fn bits(self) -> u32 {
        match self {
            BitHelperKind::Clz(bits) | BitHelperKind::Ctz(bits) | BitHelperKind::Popcnt(bits) => {
                bits
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TableHelperKind {
    Get(usize),
    Set(usize),
    Size(usize),
    Grow(usize),
    Fill(usize),
    Copy { dst: usize, src: usize },
    InitFromPassive { table: usize, segment: usize },
    ElemDrop(usize),
}

#[derive(Clone, Copy)]
pub(crate) enum CallTarget {
    Import(u32),
    Defined(usize),
}

#[derive(Default)]
pub(crate) struct HelperRecord {
    pub(super) offset: Option<usize>,
    pub(super) calls: Vec<usize>,
}

pub(crate) struct DataSegmentInfo {
    pub(super) bytes: Vec<u8>,
    pub(super) kind: DataSegmentKind,
    pub(super) defined: bool,
}

pub(crate) enum DataSegmentKind {
    Passive {
        init_record: HelperRecord,
        drop_record: HelperRecord,
        byte_slot: Option<usize>,
        drop_slot: Option<usize>,
    },
    Active {
        offset: u64,
    },
}

pub(crate) struct GlobalDescriptor {
    pub(super) slot: usize,
    pub(super) mutable: bool,
    pub(super) initial_value: i128,
    pub(super) const_value: Option<i128>,
}

pub(crate) struct TableDescriptor {
    pub(super) initial_entries: Vec<i32>,
    pub(super) maximum: Option<usize>,
    pub(super) slot: Option<usize>,
}

pub(crate) enum ElementSegmentKind {
    Passive {
        value_slot: Option<usize>,
        drop_slot: Option<usize>,
    },
    Active {
        _table_index: usize,
        _offset: usize,
    },
}

pub(crate) struct ElementSegmentInfo {
    pub(super) values: Vec<i32>,
    pub(super) kind: ElementSegmentKind,
    pub(super) defined: bool,
}

pub(crate) struct TableInfo;

pub(super) struct PassiveSegmentLayout<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) byte_slot: usize,
    pub(super) drop_slot: usize,
}

pub(super) struct ActiveSegmentLayout<'a> {
    pub(super) offset: u64,
    pub(super) bytes: &'a [u8],
}

pub(super) struct GlobalLayout {
    pub(super) slot: usize,
    pub(super) initial_value: i128,
}

pub(super) struct TableLayout<'a> {
    pub(super) slot: usize,
    pub(super) entries: &'a [i32],
}

pub(super) struct PassiveElementLayout<'a> {
    pub(super) values: &'a [i32],
    pub(super) value_slot: usize,
    pub(super) drop_slot: usize,
}
