use neo_syscalls::NeoVMSyscall;
use neo_types::*;

/// Storage convenience helpers built on top of the syscall layer.
pub struct NeoStorage;

impl NeoStorage {
    pub fn get_context() -> NeoResult<NeoStorageContext> {
        NeoVMSyscall::storage_get_context()
    }

    pub fn get_read_only_context() -> NeoResult<NeoStorageContext> {
        NeoVMSyscall::storage_get_read_only_context()
    }

    pub fn as_read_only(context: &NeoStorageContext) -> NeoResult<NeoStorageContext> {
        NeoVMSyscall::storage_as_read_only(context)
    }

    pub fn get(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<NeoByteString> {
        NeoVMSyscall::storage_get(context, key)
    }

    pub fn put(
        context: &NeoStorageContext,
        key: &NeoByteString,
        value: &NeoByteString,
    ) -> NeoResult<()> {
        NeoVMSyscall::storage_put(context, key, value)
    }

    pub fn delete(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<()> {
        NeoVMSyscall::storage_delete(context, key)
    }

    pub fn find(
        context: &NeoStorageContext,
        prefix: &NeoByteString,
    ) -> NeoResult<NeoIterator<NeoValue>> {
        NeoVMSyscall::storage_find(context, prefix)
    }
}

