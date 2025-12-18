#[derive(Debug, Clone)]
pub(super) enum Literal {
    Integer(i128),
    Bytes(Vec<u8>),
    Array(usize),
    Unknown,
}
