mod generated {
    include!(concat!(env!("OUT_DIR"), "/syscalls.rs"));
}

pub use generated::SyscallInfo;

pub fn all() -> &'static [SyscallInfo] {
    generated::SYSCALLS
}

pub fn lookup(name: &str) -> Option<&'static SyscallInfo> {
    generated::SYSCALLS
        .iter()
        .find(|info| info.name.eq_ignore_ascii_case(name))
}

pub fn lookup_by_hash(hash: u32) -> Option<&'static SyscallInfo> {
    generated::SYSCALLS.iter().find(|info| info.hash == hash)
}
