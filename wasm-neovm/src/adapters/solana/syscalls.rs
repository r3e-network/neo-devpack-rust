/// Map Solana syscall names to Neo syscall descriptors.
pub(super) fn map_solana_syscall(name: &str) -> Option<&'static str> {
    match name {
        // ===== Logging =====
        "sol_log_" | "sol_log" | "log" => Some("System.Runtime.Log"),
        "sol_log_64" | "log_64" => Some("System.Runtime.Log"),
        "sol_log_pubkey" => Some("System.Runtime.Log"),
        "sol_log_compute_units" => Some("System.Runtime.Log"),
        "sol_log_data" => Some("System.Runtime.Log"),

        // ===== Time/Clock =====
        "sol_get_clock_sysvar" | "get_clock" => Some("System.Runtime.GetTime"),
        "sol_get_epoch_schedule_sysvar" => Some("System.Runtime.GetTime"),
        "sol_get_rent_sysvar" => None, // No direct equivalent

        // ===== Crypto =====
        "sol_sha256" | "sha256" => Some("Neo.Crypto.SHA256"),
        "sol_keccak256" | "keccak256" => Some("Neo.Crypto.Keccak256"),
        "sol_blake3" => Some("Neo.Crypto.SHA256"), // No direct equivalent, fallback
        "sol_secp256k1_recover" => Some("Neo.Crypto.VerifyWithECDsa"),
        "sol_alt_bn128_group_op" => None,  // BN128 not supported
        "sol_poseidon" => None,            // Poseidon not supported
        "sol_curve25519_validate" => None, // Ed25519 validation

        // ===== Program Invocation (CPI) =====
        "sol_invoke_signed" | "sol_invoke" | "invoke" => Some("System.Contract.Call"),
        "sol_invoke_signed_c" => Some("System.Contract.Call"),
        "sol_invoke_signed_rust" => Some("System.Contract.Call"),

        // ===== Memory Operations =====
        // These are handled by wasm-neovm runtime helpers, not syscalls
        "sol_memcpy_" | "sol_memcpy" => None,
        "sol_memmove_" | "sol_memmove" => None,
        "sol_memset_" | "sol_memset" => None,
        "sol_memcmp_" | "sol_memcmp" => None,

        // ===== Return Data =====
        "sol_set_return_data" => None, // Stack-based in NeoVM
        "sol_get_return_data" => None,

        // ===== Account/Program Info =====
        "sol_get_processed_sibling_instruction" => None,
        "sol_get_stack_height" => None,
        "sol_get_last_restart_slot" => None,

        // ===== Signature Verification =====
        "sol_verify_signature" => Some("System.Runtime.CheckWitness"),

        // ===== Address Lookup Tables =====
        "sol_get_epoch_rewards_sysvar" => None,

        // ===== System Program Operations =====
        "sol_create_program_address" => None, // PDA - needs runtime emulation
        "sol_try_find_program_address" => None,

        _ => None,
    }
}
