// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Extended Solana compatibility tests covering edge cases

use neo_solana_compat::{
    account_info::{next_account_info, AccountInfo},
    program::{invoke, AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::{Pubkey, PubkeyError, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
};

// ============================================================================
// Pubkey edge cases
// ============================================================================

#[test]
fn pubkey_new_default_is_all_zeros() {
    let pk = Pubkey::new_default();
    assert_eq!(pk.to_bytes(), [0u8; 32]);
    assert_eq!(pk, Pubkey::default());
}

#[test]
fn pubkey_len_constant() {
    assert_eq!(Pubkey::LEN, 32);
}

#[test]
fn pubkey_hash_works() {
    use std::collections::HashSet;
    let pk1 = Pubkey::new([1u8; 32]);
    let pk2 = Pubkey::new([1u8; 32]);
    let pk3 = Pubkey::new([2u8; 32]);
    let mut set = HashSet::new();
    set.insert(pk1);
    set.insert(pk2);
    set.insert(pk3);
    assert_eq!(set.len(), 2);
}

#[test]
fn pubkey_neo_uint160_first_20_bytes() {
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        bytes[i] = i as u8;
    }
    let pk = Pubkey::new(bytes);
    let uint160 = pk.to_neo_uint160();
    assert_eq!(uint160.len(), 20);
    for i in 0..20 {
        assert_eq!(uint160[i], i as u8);
    }
}

#[test]
fn pubkey_system_program_is_zeros() {
    assert_eq!(SYSTEM_PROGRAM_ID.to_bytes(), [0u8; 32]);
    assert!(SYSTEM_PROGRAM_ID.is_system_program());
}

#[test]
fn pubkey_token_program_not_system() {
    assert!(!TOKEN_PROGRAM_ID.is_system_program());
    assert_ne!(TOKEN_PROGRAM_ID.to_bytes(), [0u8; 32]);
}

#[test]
fn pubkey_find_pda_deterministic() {
    let program = Pubkey::new([0xAA; 32]);
    let seeds: &[&[u8]] = &[b"mint", b"authority"];
    let (pda1, bump1) = Pubkey::find_program_address(seeds, &program);
    let (pda2, bump2) = Pubkey::find_program_address(seeds, &program);
    assert_eq!(pda1, pda2);
    assert_eq!(bump1, bump2);
    assert_eq!(bump1, 255);
}

#[test]
fn pubkey_find_pda_different_seeds_different_result() {
    let program = Pubkey::new([0xBB; 32]);
    let (pda1, _) = Pubkey::find_program_address(&[b"a"], &program);
    let (pda2, _) = Pubkey::find_program_address(&[b"b"], &program);
    assert_ne!(pda1, pda2);
}

#[test]
fn pubkey_find_pda_different_programs_different_result() {
    let program1 = Pubkey::new([1u8; 32]);
    let program2 = Pubkey::new([2u8; 32]);
    let seeds: &[&[u8]] = &[b"seed"];
    let (pda1, _) = Pubkey::find_program_address(seeds, &program1);
    let (pda2, _) = Pubkey::find_program_address(seeds, &program2);
    assert_ne!(pda1, pda2);
}

#[test]
fn pubkey_find_pda_empty_seeds() {
    let program = Pubkey::new([0xCC; 32]);
    let (pda, bump) = Pubkey::find_program_address(&[], &program);
    // With no seeds, result is just XOR of program_id
    assert_eq!(pda.to_bytes(), program.to_bytes());
    assert_eq!(bump, 255);
}

#[test]
fn pubkey_create_program_address_succeeds() {
    let program = Pubkey::new([5u8; 32]);
    let result = Pubkey::create_program_address(&[b"hello"], &program);
    assert!(result.is_ok());
}

#[test]
fn pubkey_debug_format() {
    let pk = Pubkey::new([0xFF; 32]);
    let debug = format!("{:?}", pk);
    assert!(debug.starts_with("Pubkey("));
}

#[test]
fn pubkey_as_ref_slice() {
    let bytes = [0x42u8; 32];
    let pk = Pubkey::new(bytes);
    let slice: &[u8] = pk.as_ref();
    assert_eq!(slice.len(), 32);
    assert_eq!(slice[0], 0x42);
}

// ============================================================================
// ProgramError edge cases
// ============================================================================

#[test]
fn program_error_all_variants_to_u64() {
    let variants = [
        (ProgramError::InvalidArgument, 1),
        (ProgramError::InvalidInstructionData, 2),
        (ProgramError::InvalidAccountData, 3),
        (ProgramError::AccountDataTooSmall, 4),
        (ProgramError::InsufficientFunds, 5),
        (ProgramError::IncorrectProgramId, 6),
        (ProgramError::MissingRequiredSignature, 7),
        (ProgramError::AccountAlreadyInitialized, 8),
        (ProgramError::UninitializedAccount, 9),
        (ProgramError::NotEnoughAccountKeys, 10),
        (ProgramError::AccountBorrowFailed, 11),
        (ProgramError::MaxSeedLengthExceeded, 12),
        (ProgramError::InvalidSeeds, 13),
        (ProgramError::BorshIoError, 14),
        (ProgramError::AccountNotRentExempt, 15),
        (ProgramError::UnsupportedSysvar, 16),
        (ProgramError::IllegalOwner, 17),
        (ProgramError::MaxAccountsDataSizeExceeded, 18),
        (ProgramError::InvalidReentrancy, 19),
    ];

    for (err, expected_code) in &variants {
        assert_eq!(err.to_u64(), *expected_code);
    }
}

#[test]
fn program_error_all_variants_from_u64_roundtrip() {
    for code in 1..=19u64 {
        let err = ProgramError::from(code);
        assert_eq!(err.to_u64(), code);
    }
}

#[test]
fn program_error_custom_zero() {
    let err = ProgramError::Custom(0);
    assert_eq!(err.to_u64(), 0);
}

#[test]
fn program_error_custom_u32_max() {
    let err = ProgramError::Custom(u32::MAX);
    assert_eq!(err.to_u64(), u32::MAX as u64);
}

#[test]
fn program_error_from_unknown_code_is_custom() {
    let err = ProgramError::from(100);
    assert_eq!(err, ProgramError::Custom(100));
}

#[test]
fn program_error_from_zero_is_custom() {
    let err = ProgramError::from(0u64);
    assert_eq!(err, ProgramError::Custom(0));
}

#[test]
fn program_error_display_custom() {
    assert!(format!("{}", ProgramError::Custom(42)).contains("42"));
}

#[test]
fn program_error_display_standard() {
    let display = format!("{}", ProgramError::InvalidArgument);
    assert!(display.contains("InvalidArgument"));
}

#[test]
fn program_error_clone_and_copy() {
    let err = ProgramError::InsufficientFunds;
    let cloned = err;
    assert_eq!(err, cloned);
}

// ============================================================================
// AccountInfo edge cases
// ============================================================================

#[test]
fn account_info_zero_lamports() {
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let mut lamports = 0u64;
    let mut data = vec![];
    let account = AccountInfo::new(
        &key, false, false, &mut lamports, &mut data, &owner, false, 0,
    );
    assert_eq!(account.lamports(), 0);
    assert!(account.data_is_empty());
}

#[test]
fn account_info_executable_flag() {
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let mut lamports = 0u64;
    let mut data = vec![];
    let account = AccountInfo::new(
        &key, false, false, &mut lamports, &mut data, &owner, true, 0,
    );
    assert!(account.executable);
}

#[test]
fn account_info_rent_epoch() {
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let mut lamports = 0u64;
    let mut data = vec![];
    let account = AccountInfo::new(
        &key, false, false, &mut lamports, &mut data, &owner, false, 42,
    );
    assert_eq!(account.rent_epoch, 42);
}

#[test]
fn account_info_signer_and_writable_flags() {
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let mut lamports = 0u64;
    let mut data = vec![];

    // Neither signer nor writable
    let account = AccountInfo::new(
        &key, false, false, &mut lamports, &mut data, &owner, false, 0,
    );
    assert!(!account.is_signer());
    assert!(!account.is_writable());
}

#[test]
fn next_account_info_empty_iterator() {
    let accounts: [AccountInfo; 0] = [];
    let mut iter = accounts.iter();
    let result = next_account_info(&mut iter);
    assert_eq!(result.unwrap_err(), ProgramError::NotEnoughAccountKeys);
}

// ============================================================================
// Instruction tests
// ============================================================================

#[test]
fn instruction_empty_data() {
    let program_id = Pubkey::default();
    let ix = Instruction::new(program_id, vec![], vec![]);
    assert!(ix.data.is_empty());
    assert!(ix.accounts.is_empty());
}

#[test]
fn instruction_with_method_empty_name() {
    let program_id = Pubkey::default();
    let ix = Instruction::new_with_method(program_id, "", vec![42], vec![]);
    assert_eq!(ix.data[0], 0); // empty method name length
    assert_eq!(ix.data[1], 42); // original data
}

#[test]
fn account_meta_new_all_flags() {
    let pk = Pubkey::new([1u8; 32]);
    let writable_signer = AccountMeta::new(pk, true);
    assert!(writable_signer.is_writable);
    assert!(writable_signer.is_signer);

    let writable_no_signer = AccountMeta::new(pk, false);
    assert!(writable_no_signer.is_writable);
    assert!(!writable_no_signer.is_signer);

    let readonly_signer = AccountMeta::new_readonly(pk, true);
    assert!(!readonly_signer.is_writable);
    assert!(readonly_signer.is_signer);

    let readonly_no_signer = AccountMeta::new_readonly(pk, false);
    assert!(!readonly_no_signer.is_writable);
    assert!(!readonly_no_signer.is_signer);
}

// ============================================================================
// PubkeyError
// ============================================================================

#[test]
fn pubkey_error_debug() {
    let err = PubkeyError::MaxSeedLengthExceeded;
    let debug = format!("{:?}", err);
    assert!(debug.contains("MaxSeedLengthExceeded"));

    let err2 = PubkeyError::InvalidSeeds;
    assert_ne!(err, err2);
}
