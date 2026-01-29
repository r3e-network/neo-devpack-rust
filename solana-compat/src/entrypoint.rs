//! Entrypoint macros and types for Solana compatibility
//!
//! Provides the `entrypoint!` macro that maps to Neo contract entry points.

use crate::account_info::AccountInfo;
use crate::program_error::ProgramError;
use crate::pubkey::Pubkey;

/// Result type for program execution
pub type ProgramResult = Result<(), ProgramError>;

/// Type of the user-defined entrypoint function
pub type ProcessInstruction =
    fn(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult;

/// Declare the program entrypoint
///
/// This macro creates a WASM export that:
/// 1. Deserializes input from Neo transaction data
/// 2. Calls the user's `process_instruction` function
/// 3. Returns success/failure to `NeoVM`
///
/// # Example
///
/// ```rust,ignore
/// use neo_solana_compat::{entrypoint, entrypoint::ProgramResult, pubkey::Pubkey, account_info::AccountInfo};
///
/// entrypoint!(process_instruction);
///
/// fn process_instruction(
///     program_id: &Pubkey,
///     accounts: &[AccountInfo],
///     instruction_data: &[u8],
/// ) -> ProgramResult {
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! entrypoint {
    ($process_instruction:ident) => {
        /// The program entrypoint
        #[cfg(not(feature = "no-entrypoint"))]
        #[no_mangle]
        pub extern "C" fn main(input: i32, input_len: i32) -> i64 {
            // In NeoVM context:
            // - input: pointer to memory containing serialized transaction data
            // - input_len: length of the input data
            // Returns: 0 for success, non-zero error code for failure

            // SAFETY: This is called by the NeoVM runtime with valid pointers.
            // The input pointer is guaranteed to be valid for input_len bytes.
            let result = unsafe {
                $crate::entrypoint::__neo_process_instruction(
                    input as *const u8,
                    input_len as usize,
                    $process_instruction,
                )
            };

            match result {
                Ok(()) => 0,
                Err(e) => e.to_u64() as i64,
            }
        }
    };
}

/// Internal function to process instruction (called by entrypoint macro)
///
/// # Safety
///
/// The caller must ensure:
/// - `input` is a valid, non-null pointer when `input_len > 0`
/// - `input` points to at least `input_len` valid bytes
/// - The memory remains valid and immutable for the duration of this function
///
/// These invariants are guaranteed when called by the NeoVM runtime through
/// the entrypoint macro.
#[doc(hidden)]
pub unsafe fn __neo_process_instruction(
    input: *const u8,
    input_len: usize,
    process_instruction: ProcessInstruction,
) -> ProgramResult {
    // Create a slice from the input pointer
    let data = if input.is_null() || input_len == 0 {
        &[]
    } else {
        // SAFETY: The caller guarantees the pointer is valid for input_len bytes.
        core::slice::from_raw_parts(input, input_len)
    };

    // Parse the input format:
    // [32 bytes: program_id][4 bytes: num_accounts][account_data...][instruction_data...]
    if data.len() < 36 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Extract program ID
    let mut program_id_bytes = [0u8; 32];
    program_id_bytes.copy_from_slice(&data[0..32]);
    let program_id = Pubkey::new(program_id_bytes);

    // Number of accounts
    let num_accounts = u32::from_le_bytes([data[32], data[33], data[34], data[35]]) as usize;

    // For now, pass empty accounts and remaining data as instruction_data
    // Full implementation would parse account infos from the data
    let accounts: &[AccountInfo] = &[];
    let instruction_data = if data.len() > 36 { &data[36..] } else { &[] };

    // Suppress unused variable warning
    let _ = num_accounts;

    // Call the user's process instruction function
    process_instruction(&program_id, accounts, instruction_data)
}

/// Simplified entrypoint that receives instruction data directly
///
/// This is the Neo-native approach where the contract method
/// receives parameters directly rather than through serialized accounts.
#[macro_export]
macro_rules! neo_entrypoint {
    ($name:ident, $handler:ident) => {
        #[no_mangle]
        pub extern "C" fn $name() {
            $handler()
        }
    };
    ($name:ident, $handler:ident, $($param:ident : $type:ty),*) => {
        #[no_mangle]
        pub extern "C" fn $name($($param: $type),*) {
            $handler($($param),*)
        }
    };
}
