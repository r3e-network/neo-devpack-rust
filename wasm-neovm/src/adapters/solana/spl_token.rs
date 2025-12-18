/// Map SPL Token program calls to Neo equivalents.
pub(super) fn map_spl_token_syscall(name: &str) -> Option<&'static str> {
    match name {
        // SPL Token operations map to NEP-17 equivalents via contract calls
        "transfer" | "transfer_checked" => Some("System.Contract.Call"),
        "mint_to" | "mint_to_checked" => Some("System.Contract.Call"),
        "burn" | "burn_checked" => Some("System.Contract.Call"),
        "approve" | "approve_checked" => Some("System.Contract.Call"),
        "revoke" => Some("System.Contract.Call"),
        "initialize_mint" => Some("System.Contract.Call"),
        "initialize_account" => Some("System.Contract.Call"),
        "close_account" => Some("System.Contract.Call"),
        "freeze_account" => Some("System.Contract.Call"),
        "thaw_account" => Some("System.Contract.Call"),
        "get_account_data_size" => Some("System.Storage.Get"),
        _ => None,
    }
}
