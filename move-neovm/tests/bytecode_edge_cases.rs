// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Edge case tests for Move bytecode parsing and runtime

use move_neovm::bytecode::{AbilitySet, MoveOpcode, TypeTag};
use move_neovm::{
    global_storage_key, is_move_bytecode, parse_move_bytecode, signer_to_checkwitness,
    validate_move_bytecode, ResourceError, ResourceTracker,
};

// ============================================================================
// Bytecode validation
// ============================================================================

#[test]
fn validate_empty_bytes() {
    assert!(!validate_move_bytecode(&[]));
}

#[test]
fn validate_short_bytes() {
    assert!(!validate_move_bytecode(&[0xa1]));
    assert!(!validate_move_bytecode(&[0xa1, 0x1c]));
    assert!(!validate_move_bytecode(&[0xa1, 0x1c, 0xeb]));
}

#[test]
fn validate_valid_magic() {
    assert!(validate_move_bytecode(&[0xa1, 0x1c, 0xeb, 0x0b]));
}

#[test]
fn validate_wrong_magic() {
    assert!(!validate_move_bytecode(&[0x00, 0x61, 0x73, 0x6d])); // WASM
    assert!(!validate_move_bytecode(&[0xFF, 0xFF, 0xFF, 0xFF]));
    assert!(!validate_move_bytecode(&[0x00, 0x00, 0x00, 0x00]));
}

#[test]
fn is_move_bytecode_wrapper() {
    assert!(is_move_bytecode(&[0xa1, 0x1c, 0xeb, 0x0b, 0, 0, 0, 0]));
    assert!(!is_move_bytecode(&[0, 0, 0, 0]));
}

// ============================================================================
// Parse errors
// ============================================================================

#[test]
fn parse_too_short() {
    let result = parse_move_bytecode(&[0xa1, 0x1c, 0xeb, 0x0b]);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("too short"));
}

#[test]
fn parse_wrong_magic() {
    let result = parse_move_bytecode(&[0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("magic"));
}

#[test]
fn parse_minimal_valid_bytecode() {
    // Magic + version + 0 tables (ULEB128 0)
    let bytes = [
        0xa1, 0x1c, 0xeb, 0x0b, // magic
        0x06, 0x00, 0x00, 0x00, // version 6
        0x00, // table count: 0
    ];
    let result = parse_move_bytecode(&bytes);
    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.version.0, 6);
}

// ============================================================================
// TypeTag
// ============================================================================

#[test]
fn type_tag_to_wasm_type() {
    assert_eq!(TypeTag::Bool.to_wasm_type(), "i32");
    assert_eq!(TypeTag::U8.to_wasm_type(), "i32");
    assert_eq!(TypeTag::U64.to_wasm_type(), "i64");
    assert_eq!(TypeTag::U128.to_wasm_type(), "i64");
    assert_eq!(TypeTag::U256.to_wasm_type(), "i64");
    assert_eq!(TypeTag::Address.to_wasm_type(), "i32");
    assert_eq!(TypeTag::Signer.to_wasm_type(), "i32");
    assert_eq!(TypeTag::Vector(Box::new(TypeTag::U8)).to_wasm_type(), "i32");
    assert_eq!(TypeTag::Struct("Coin".to_string()).to_wasm_type(), "i32");
    assert_eq!(
        TypeTag::Reference(Box::new(TypeTag::U64)).to_wasm_type(),
        "i32"
    );
    assert_eq!(
        TypeTag::MutableReference(Box::new(TypeTag::Bool)).to_wasm_type(),
        "i32"
    );
}

// ============================================================================
// AbilitySet
// ============================================================================

#[test]
fn ability_set_default_is_not_resource() {
    let abilities = AbilitySet::default();
    assert!(!abilities.is_resource());
    assert!(!abilities.copy);
    assert!(!abilities.drop);
    assert!(!abilities.store);
    assert!(!abilities.key);
}

#[test]
fn ability_set_key_is_resource() {
    let abilities = AbilitySet {
        key: true,
        ..Default::default()
    };
    assert!(abilities.is_resource());
}

// ============================================================================
// MoveOpcode byte values
// ============================================================================

#[test]
fn opcode_byte_roundtrip() {
    assert_eq!(MoveOpcode::Nop.opcode_byte(), 0x00);
    assert_eq!(MoveOpcode::Pop.opcode_byte(), 0x01);
    assert_eq!(MoveOpcode::Ret.opcode_byte(), 0x02);
    assert_eq!(MoveOpcode::BrTrue(0).opcode_byte(), 0x03);
    assert_eq!(MoveOpcode::BrFalse(0).opcode_byte(), 0x04);
    assert_eq!(MoveOpcode::Branch(0).opcode_byte(), 0x05);
    assert_eq!(MoveOpcode::LdU8(0).opcode_byte(), 0x06);
    assert_eq!(MoveOpcode::LdU64(0).opcode_byte(), 0x07);
    assert_eq!(MoveOpcode::LdU128(0).opcode_byte(), 0x08);
    assert_eq!(MoveOpcode::LdTrue.opcode_byte(), 0x0D);
    assert_eq!(MoveOpcode::LdFalse.opcode_byte(), 0x0E);
    assert_eq!(MoveOpcode::Add.opcode_byte(), 0x22);
    assert_eq!(MoveOpcode::Sub.opcode_byte(), 0x23);
    assert_eq!(MoveOpcode::Mul.opcode_byte(), 0x24);
    assert_eq!(MoveOpcode::Mod.opcode_byte(), 0x25);
    assert_eq!(MoveOpcode::Div.opcode_byte(), 0x26);
    assert_eq!(MoveOpcode::Lt.opcode_byte(), 0x32);
    assert_eq!(MoveOpcode::Gt.opcode_byte(), 0x33);
    assert_eq!(MoveOpcode::Le.opcode_byte(), 0x34);
    assert_eq!(MoveOpcode::Ge.opcode_byte(), 0x35);
    assert_eq!(MoveOpcode::And.opcode_byte(), 0x40);
    assert_eq!(MoveOpcode::Or.opcode_byte(), 0x41);
    assert_eq!(MoveOpcode::Not.opcode_byte(), 0x42);
    assert_eq!(MoveOpcode::Eq.opcode_byte(), 0x43);
    assert_eq!(MoveOpcode::Neq.opcode_byte(), 0x44);
    assert_eq!(MoveOpcode::Abort.opcode_byte(), 0x45);
}

#[test]
fn opcode_byte_resource_ops() {
    assert_eq!(MoveOpcode::Exists(0).opcode_byte(), 0x50);
    assert_eq!(MoveOpcode::BorrowGlobal(0).opcode_byte(), 0x51);
    assert_eq!(MoveOpcode::MutBorrowGlobal(0).opcode_byte(), 0x52);
    assert_eq!(MoveOpcode::MoveFrom(0).opcode_byte(), 0x53);
    assert_eq!(MoveOpcode::MoveTo(0).opcode_byte(), 0x54);
}

#[test]
fn opcode_byte_vector_ops() {
    assert_eq!(MoveOpcode::VecPack(0, 0).opcode_byte(), 0x60);
    assert_eq!(MoveOpcode::VecLen(0).opcode_byte(), 0x61);
    assert_eq!(MoveOpcode::VecImmBorrow(0).opcode_byte(), 0x62);
    assert_eq!(MoveOpcode::VecMutBorrow(0).opcode_byte(), 0x63);
    assert_eq!(MoveOpcode::VecPushBack(0).opcode_byte(), 0x64);
    assert_eq!(MoveOpcode::VecPopBack(0).opcode_byte(), 0x65);
}

#[test]
fn opcode_byte_cast_ops() {
    assert_eq!(MoveOpcode::CastU8.opcode_byte(), 0x09);
    assert_eq!(MoveOpcode::CastU64.opcode_byte(), 0x0A);
    assert_eq!(MoveOpcode::CastU128.opcode_byte(), 0x0B);
}

#[test]
fn opcode_byte_local_ops() {
    assert_eq!(MoveOpcode::CopyLoc(0).opcode_byte(), 0x0F);
    assert_eq!(MoveOpcode::MoveLoc(0).opcode_byte(), 0x10);
    assert_eq!(MoveOpcode::StLoc(0).opcode_byte(), 0x11);
    assert_eq!(MoveOpcode::MutBorrowLoc(0).opcode_byte(), 0x12);
    assert_eq!(MoveOpcode::ImmBorrowLoc(0).opcode_byte(), 0x13);
}

// ============================================================================
// ResourceTracker
// ============================================================================

#[test]
fn resource_tracker_new_is_empty() {
    let tracker = ResourceTracker::new();
    assert!(!tracker.exists(b"addr", "Type"));
}

#[test]
fn resource_tracker_move_to_and_exists() {
    let mut tracker = ResourceTracker::new();
    tracker.move_to(b"addr1", "Coin").unwrap();
    assert!(tracker.exists(b"addr1", "Coin"));
    assert!(!tracker.exists(b"addr2", "Coin"));
    assert!(!tracker.exists(b"addr1", "Token"));
}

#[test]
fn resource_tracker_double_move_to_fails() {
    let mut tracker = ResourceTracker::new();
    tracker.move_to(b"addr", "Coin").unwrap();
    let err = tracker.move_to(b"addr", "Coin").unwrap_err();
    assert!(matches!(err, ResourceError::AlreadyExists { .. }));
}

#[test]
fn resource_tracker_move_from_nonexistent_fails() {
    let mut tracker = ResourceTracker::new();
    let err = tracker.move_from(b"addr", "Coin").unwrap_err();
    assert!(matches!(err, ResourceError::NotFound { .. }));
}

#[test]
fn resource_tracker_borrow_existing() {
    let mut tracker = ResourceTracker::new();
    tracker.move_to(b"addr", "Coin").unwrap();
    assert!(tracker.borrow(b"addr", "Coin").is_ok());
}

#[test]
fn resource_tracker_borrow_nonexistent_fails() {
    let tracker = ResourceTracker::new();
    let err = tracker.borrow(b"addr", "Coin").unwrap_err();
    assert!(matches!(err, ResourceError::NotFound { .. }));
}

#[test]
fn resource_tracker_full_lifecycle() {
    let mut tracker = ResourceTracker::new();
    let addr = b"test";
    let ty = "0x1::M::R";

    // Create
    tracker.move_to(addr, ty).unwrap();
    assert!(tracker.exists(addr, ty));

    // Borrow
    tracker.borrow(addr, ty).unwrap();

    // Remove
    tracker.move_from(addr, ty).unwrap();
    assert!(!tracker.exists(addr, ty));

    // Re-create
    tracker.move_to(addr, ty).unwrap();
    assert!(tracker.exists(addr, ty));
}

#[test]
fn resource_tracker_multiple_types_at_same_address() {
    let mut tracker = ResourceTracker::new();
    tracker.move_to(b"addr", "Coin").unwrap();
    tracker.move_to(b"addr", "Token").unwrap();
    assert!(tracker.exists(b"addr", "Coin"));
    assert!(tracker.exists(b"addr", "Token"));
    tracker.move_from(b"addr", "Coin").unwrap();
    assert!(!tracker.exists(b"addr", "Coin"));
    assert!(tracker.exists(b"addr", "Token"));
}

// ============================================================================
// Global storage key
// ============================================================================

#[test]
fn global_storage_key_format() {
    let key = global_storage_key(b"\x01\x02", "Coin");
    assert_eq!(key[0], b'R'); // prefix
    assert_eq!(key[1], 0x01);
    assert_eq!(key[2], 0x02);
    assert_eq!(key[3], b':'); // separator
    assert_eq!(&key[4..], b"Coin");
}

#[test]
fn global_storage_key_empty_address() {
    let key = global_storage_key(b"", "Type");
    assert_eq!(key, vec![b'R', b':', b'T', b'y', b'p', b'e']);
}

#[test]
fn global_storage_key_empty_type() {
    let key = global_storage_key(b"\x01", "");
    assert_eq!(key, vec![b'R', 0x01, b':']);
}

// ============================================================================
// signer_to_checkwitness
// ============================================================================

#[test]
fn signer_maps_to_checkwitness() {
    assert_eq!(signer_to_checkwitness(), "System.Runtime.CheckWitness");
}

// ============================================================================
// ResourceError display
// ============================================================================

#[test]
fn resource_error_display_already_exists() {
    let err = ResourceError::AlreadyExists {
        type_name: "Coin".to_string(),
    };
    assert!(err.to_string().contains("already exists"));
    assert!(err.to_string().contains("Coin"));
}

#[test]
fn resource_error_display_not_found() {
    let err = ResourceError::NotFound {
        type_name: "Token".to_string(),
    };
    assert!(err.to_string().contains("not found"));
    assert!(err.to_string().contains("Token"));
}

#[test]
fn resource_error_display_cannot_copy() {
    let err = ResourceError::CannotCopy {
        type_name: "NFT".to_string(),
    };
    assert!(err.to_string().contains("cannot be copied"));
}

#[test]
fn resource_error_display_cannot_drop() {
    let err = ResourceError::CannotDrop {
        type_name: "Vault".to_string(),
    };
    assert!(err.to_string().contains("cannot be dropped"));
}
