//! Comprehensive unit tests for Solana compatibility layer
//!
//! Tests cover all major components of the neo-solana-compat crate.

use neo_solana_compat::{
    account_info::{AccountInfo, next_account_info},
    program::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::{Pubkey, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
};

// ============================================================================
// Pubkey Tests
// ============================================================================

#[test]
fn test_pubkey_new() {
    let bytes = [1u8; 32];
    let pk = Pubkey::new(bytes);
    assert_eq!(pk.to_bytes(), bytes);
}

#[test]
fn test_pubkey_default() {
    let pk = Pubkey::default();
    assert_eq!(pk.to_bytes(), [0u8; 32]);
}

#[test]
fn test_pubkey_from_slice() {
    let bytes = [42u8; 32];
    let pk = Pubkey::new_from_slice(&bytes);
    assert_eq!(pk.to_bytes(), bytes);
}

#[test]
fn test_pubkey_to_neo_uint160() {
    let mut bytes = [0u8; 32];
    for i in 0..20 {
        bytes[i] = i as u8;
    }
    let pk = Pubkey::new(bytes);
    let uint160 = pk.to_neo_uint160();

    for i in 0..20 {
        assert_eq!(uint160[i], i as u8);
    }
}

#[test]
fn test_pubkey_is_system_program() {
    assert!(SYSTEM_PROGRAM_ID.is_system_program());

    let non_system = Pubkey::new([1u8; 32]);
    assert!(!non_system.is_system_program());
}

#[test]
fn test_pubkey_find_program_address() {
    let program_id = Pubkey::new([1u8; 32]);
    let seeds: &[&[u8]] = &[b"seed1", b"seed2"];

    let (pda, bump) = Pubkey::find_program_address(seeds, &program_id);

    // PDA should be deterministic
    let (pda2, bump2) = Pubkey::find_program_address(seeds, &program_id);
    assert_eq!(pda.to_bytes(), pda2.to_bytes());
    assert_eq!(bump, bump2);
}

#[test]
fn test_pubkey_create_program_address() {
    let program_id = Pubkey::new([1u8; 32]);
    let seeds: &[&[u8]] = &[b"test"];

    let result = Pubkey::create_program_address(seeds, &program_id);
    assert!(result.is_ok());
}

#[test]
fn test_pubkey_equality() {
    let pk1 = Pubkey::new([1u8; 32]);
    let pk2 = Pubkey::new([1u8; 32]);
    let pk3 = Pubkey::new([2u8; 32]);

    assert_eq!(pk1, pk2);
    assert_ne!(pk1, pk3);
}

#[test]
fn test_pubkey_clone() {
    let pk1 = Pubkey::new([42u8; 32]);
    let pk2 = pk1.clone();
    assert_eq!(pk1, pk2);
}

#[test]
fn test_pubkey_as_ref() {
    let bytes = [42u8; 32];
    let pk = Pubkey::new(bytes);
    let slice: &[u8] = pk.as_ref();
    assert_eq!(slice, &bytes[..]);
}

// ============================================================================
// ProgramError Tests
// ============================================================================

#[test]
fn test_program_error_to_u64() {
    assert_eq!(ProgramError::InvalidArgument.to_u64(), 1);
    assert_eq!(ProgramError::InvalidInstructionData.to_u64(), 2);
    assert_eq!(ProgramError::InvalidAccountData.to_u64(), 3);
    assert_eq!(ProgramError::InsufficientFunds.to_u64(), 5);
    assert_eq!(ProgramError::MissingRequiredSignature.to_u64(), 7);
    assert_eq!(ProgramError::NotEnoughAccountKeys.to_u64(), 10);
    assert_eq!(ProgramError::Custom(999).to_u64(), 999);
}

#[test]
fn test_program_error_from_u64() {
    assert_eq!(ProgramError::from(1), ProgramError::InvalidArgument);
    assert_eq!(ProgramError::from(2), ProgramError::InvalidInstructionData);
    assert_eq!(ProgramError::from(7), ProgramError::MissingRequiredSignature);
    assert_eq!(ProgramError::from(12345), ProgramError::Custom(12345));
}

#[test]
fn test_program_error_roundtrip() {
    let errors = [
        ProgramError::InvalidArgument,
        ProgramError::InsufficientFunds,
        ProgramError::AccountAlreadyInitialized,
        ProgramError::Custom(42),
    ];

    for err in errors {
        let code = err.to_u64();
        let recovered = ProgramError::from(code);
        assert_eq!(err, recovered);
    }
}

// ============================================================================
// AccountInfo Tests
// ============================================================================

#[test]
fn test_account_info_creation() {
    let key = Pubkey::new([1u8; 32]);
    let owner = Pubkey::new([2u8; 32]);
    let mut lamports = 1000u64;
    let mut data = vec![0u8; 100];

    let account = AccountInfo::new(
        &key,
        true,  // is_signer
        true,  // is_writable
        &mut lamports,
        &mut data,
        &owner,
        false, // executable
        0,     // rent_epoch
    );

    assert_eq!(account.key(), &key);
    assert_eq!(account.owner(), &owner);
    assert!(account.is_signer());
    assert!(account.is_writable());
    assert_eq!(account.lamports(), 1000);
    assert_eq!(account.data_len(), 100);
}

#[test]
fn test_account_info_borrow_data() {
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let mut lamports = 0u64;
    let mut data = vec![1, 2, 3, 4, 5];

    let account = AccountInfo::new(
        &key, false, false, &mut lamports, &mut data, &owner, false, 0,
    );

    // Immutable borrow
    let borrowed = account.try_borrow_data();
    assert!(borrowed.is_ok());
    assert_eq!(borrowed.unwrap().len(), 5);
}

#[test]
fn test_account_info_borrow_mut_data() {
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let mut lamports = 0u64;
    let mut data = vec![0u8; 10];

    let account = AccountInfo::new(
        &key, false, true, &mut lamports, &mut data, &owner, false, 0,
    );

    // Mutable borrow
    {
        let mut borrowed = account.try_borrow_mut_data().unwrap();
        borrowed[0] = 42;
    }

    // Verify mutation persisted
    let borrowed = account.try_borrow_data().unwrap();
    assert_eq!(borrowed[0], 42);
}

#[test]
fn test_account_info_data_is_empty() {
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let mut lamports = 0u64;
    let mut empty_data: Vec<u8> = vec![];
    let mut non_empty_data = vec![1u8];

    let empty_account = AccountInfo::new(
        &key, false, false, &mut lamports, &mut empty_data, &owner, false, 0,
    );
    assert!(empty_account.data_is_empty());

    let mut lamports2 = 0u64;
    let non_empty_account = AccountInfo::new(
        &key, false, false, &mut lamports2, &mut non_empty_data, &owner, false, 0,
    );
    assert!(!non_empty_account.data_is_empty());
}

#[test]
fn test_next_account_info() {
    let key1 = Pubkey::new([1u8; 32]);
    let key2 = Pubkey::new([2u8; 32]);
    let owner = Pubkey::default();
    let mut lamports1 = 0u64;
    let mut lamports2 = 0u64;
    let mut data1 = vec![0u8; 10];
    let mut data2 = vec![0u8; 10];

    let account1 = AccountInfo::new(
        &key1, false, false, &mut lamports1, &mut data1, &owner, false, 0,
    );
    let account2 = AccountInfo::new(
        &key2, false, false, &mut lamports2, &mut data2, &owner, false, 0,
    );

    let accounts = [account1, account2];
    let mut iter = accounts.iter();

    let first = next_account_info(&mut iter);
    assert!(first.is_ok());
    assert_eq!(first.unwrap().key(), &key1);

    let second = next_account_info(&mut iter);
    assert!(second.is_ok());
    assert_eq!(second.unwrap().key(), &key2);

    // Third should fail
    let third = next_account_info(&mut iter);
    assert!(third.is_err());
    assert_eq!(third.unwrap_err(), ProgramError::NotEnoughAccountKeys);
}

// ============================================================================
// Instruction Tests
// ============================================================================

#[test]
fn test_instruction_creation() {
    let program_id = Pubkey::new([1u8; 32]);
    let data = vec![1, 2, 3, 4];
    let accounts = vec![
        AccountMeta::new(Pubkey::new([2u8; 32]), true),
        AccountMeta::new_readonly(Pubkey::new([3u8; 32]), false),
    ];

    let ix = Instruction::new(program_id, data.clone(), accounts.clone());

    assert_eq!(ix.program_id, program_id);
    assert_eq!(ix.data, data);
    assert_eq!(ix.accounts.len(), 2);
}

#[test]
fn test_instruction_with_method() {
    let program_id = Pubkey::new([1u8; 32]);
    let data = vec![42];
    let accounts = vec![];

    let ix = Instruction::new_with_method(program_id, "transfer", data.clone(), accounts);

    // Method name should be encoded at start of data
    assert_eq!(ix.data[0], 8); // "transfer" length
    assert_eq!(&ix.data[1..9], b"transfer");
    assert_eq!(ix.data[9], 42); // Original data
}

#[test]
fn test_account_meta_writable() {
    let pk = Pubkey::new([1u8; 32]);

    let writable = AccountMeta::new(pk, false);
    assert!(writable.is_writable);
    assert!(!writable.is_signer);

    let writable_signer = AccountMeta::new(pk, true);
    assert!(writable_signer.is_writable);
    assert!(writable_signer.is_signer);
}

#[test]
fn test_account_meta_readonly() {
    let pk = Pubkey::new([1u8; 32]);

    let readonly = AccountMeta::new_readonly(pk, false);
    assert!(!readonly.is_writable);
    assert!(!readonly.is_signer);

    let readonly_signer = AccountMeta::new_readonly(pk, true);
    assert!(!readonly_signer.is_writable);
    assert!(readonly_signer.is_signer);
}

// ============================================================================
// Token Program ID Tests
// ============================================================================

#[test]
fn test_token_program_id_constant() {
    // Verify TOKEN_PROGRAM_ID is not all zeros
    assert_ne!(TOKEN_PROGRAM_ID.to_bytes(), [0u8; 32]);
}

// ============================================================================
// Debug/Display Tests
// ============================================================================

#[test]
fn test_pubkey_debug() {
    let pk = Pubkey::new([0xAB; 32]);
    let debug_str = format!("{:?}", pk);
    assert!(debug_str.contains("Pubkey"));
}

#[test]
fn test_program_error_display() {
    let err = ProgramError::Custom(42);
    let display = format!("{}", err);
    assert!(display.contains("42"));
}

#[test]
fn test_account_info_debug() {
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let mut lamports = 100u64;
    let mut data = vec![0u8; 50];

    let account = AccountInfo::new(
        &key, true, true, &mut lamports, &mut data, &owner, false, 0,
    );

    let debug_str = format!("{:?}", account);
    assert!(debug_str.contains("AccountInfo"));
    assert!(debug_str.contains("is_signer"));
    assert!(debug_str.contains("data_len"));
}
