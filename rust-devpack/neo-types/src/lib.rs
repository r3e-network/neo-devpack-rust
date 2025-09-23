//! Neo N3 Core Types
//! 
//! This crate provides the core types and data structures for Neo N3 smart contract development.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

use core::fmt;
use core::ops::{Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor, Not, Shl, Shr};
use core::cmp::{PartialEq, Eq, PartialOrd, Ord, Ordering};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec, boxed::Box};

/// Neo N3 Integer type (32-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NeoInteger(pub i32);

impl NeoInteger {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const MIN: Self = Self(i32::MIN);
    pub const MAX: Self = Self(i32::MAX);
    
    pub fn new(value: i32) -> Self {
        Self(value)
    }
    
    pub fn as_i32(self) -> i32 {
        self.0
    }
    
    pub fn as_u32(self) -> u32 {
        self.0 as u32
    }
}

impl Add for NeoInteger {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for NeoInteger {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for NeoInteger {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for NeoInteger {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Rem for NeoInteger {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl BitAnd for NeoInteger {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for NeoInteger {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for NeoInteger {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Not for NeoInteger {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Shl<u32> for NeoInteger {
    type Output = Self;
    fn shl(self, rhs: u32) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl Shr<u32> for NeoInteger {
    type Output = Self;
    fn shr(self, rhs: u32) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

/// Neo N3 Boolean type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct NeoBoolean(pub bool);

impl NeoBoolean {
    pub const TRUE: Self = Self(true);
    pub const FALSE: Self = Self(false);
    
    pub fn new(value: bool) -> Self {
        Self(value)
    }
    
    pub fn as_bool(self) -> bool {
        self.0
    }
}

impl BitAnd for NeoBoolean {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for NeoBoolean {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for NeoBoolean {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Not for NeoBoolean {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// Neo N3 ByteString type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoByteString {
    data: Vec<u8>,
}

impl NeoByteString {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
    
    pub fn from_slice(slice: &[u8]) -> Self {
        Self { data: slice.to_vec() }
    }
    
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn push(&mut self, byte: u8) {
        self.data.push(byte);
    }
    
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.data.extend_from_slice(slice);
    }
}

/// Neo N3 String type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoString {
    data: String,
}

impl NeoString {
    pub fn new(data: String) -> Self {
        Self { data }
    }
    
    pub fn from_str(s: &str) -> Self {
        Self { data: s.to_string() }
    }
    
    pub fn as_str(&self) -> &str {
        &self.data
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Neo N3 Array type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoArray<T> {
    data: Vec<T>,
}

impl<T> NeoArray<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self { data: Vec::with_capacity(capacity) }
    }
    
    pub fn from_vec(data: Vec<T>) -> Self {
        Self { data }
    }
    
    pub fn push(&mut self, item: T) {
        self.data.push(item);
    }
    
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
}

impl<T> Default for NeoArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<Vec<T>> for NeoArray<T> {
    fn from(data: Vec<T>) -> Self {
        Self { data }
    }
}

/// Neo N3 Map type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoMap<K, V> {
    data: Vec<(K, V)>,
}

impl<K, V> NeoMap<K, V> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    pub fn insert(&mut self, key: K, value: V) -> Option<V> 
    where 
        K: PartialEq,
    {
        for (k, v) in &mut self.data {
            if *k == key {
                return Some(core::mem::replace(v, value));
            }
        }
        self.data.push((key, value));
        None
    }
    
    pub fn get(&self, key: &K) -> Option<&V> 
    where 
        K: PartialEq,
    {
        for (k, v) in &self.data {
            if k == key {
                return Some(v);
            }
        }
        None
    }
    
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> 
    where 
        K: PartialEq,
    {
        for (k, v) in &mut self.data {
            if k == key {
                return Some(v);
            }
        }
        None
    }
    
    pub fn remove(&mut self, key: &K) -> Option<V> 
    where 
        K: PartialEq,
    {
        for (i, (k, _)) in self.data.iter().enumerate() {
            if k == key {
                return Some(self.data.remove(i).1);
            }
        }
        None
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<K, V> Default for NeoMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Neo N3 Struct type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoStruct {
    fields: Vec<(String, NeoValue)>,
}

impl NeoStruct {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }
    
    pub fn with_field(mut self, name: &str, value: NeoValue) -> Self {
        self.fields.push((name.to_string(), value));
        self
    }
    
    pub fn get_field(&self, name: &str) -> Option<&NeoValue> {
        for (field_name, value) in &self.fields {
            if field_name == name {
                return Some(value);
            }
        }
        None
    }
    
    pub fn set_field(&mut self, name: &str, value: NeoValue) {
        for (field_name, field_value) in &mut self.fields {
            if field_name == name {
                *field_value = value;
                return;
            }
        }
        self.fields.push((name.to_string(), value));
    }
    
    pub fn insert(&mut self, name: NeoString, value: NeoValue) {
        self.set_field(name.as_str(), value);
    }
    
    pub fn len(&self) -> usize {
        self.fields.len()
    }
}

impl Default for NeoStruct {
    fn default() -> Self {
        Self::new()
    }
}

/// Neo N3 Value type (union of all Neo types)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeoValue {
    Integer(NeoInteger),
    Boolean(NeoBoolean),
    ByteString(NeoByteString),
    String(NeoString),
    Array(NeoArray<NeoValue>),
    Map(NeoMap<NeoValue, NeoValue>),
    Struct(NeoStruct),
    Null,
}

impl NeoValue {
    pub fn is_null(&self) -> bool {
        matches!(self, NeoValue::Null)
    }
    
    pub fn as_integer(&self) -> Option<NeoInteger> {
        match self {
            NeoValue::Integer(i) => Some(*i),
            _ => None,
        }
    }
    
    pub fn as_boolean(&self) -> Option<NeoBoolean> {
        match self {
            NeoValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    
    pub fn as_byte_string(&self) -> Option<&NeoByteString> {
        match self {
            NeoValue::ByteString(bs) => Some(bs),
            _ => None,
        }
    }
    
    pub fn as_string(&self) -> Option<&NeoString> {
        match self {
            NeoValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_array(&self) -> Option<&NeoArray<NeoValue>> {
        match self {
            NeoValue::Array(a) => Some(a),
            _ => None,
        }
    }
    
    pub fn as_map(&self) -> Option<&NeoMap<NeoValue, NeoValue>> {
        match self {
            NeoValue::Map(m) => Some(m),
            _ => None,
        }
    }
    
    pub fn as_struct(&self) -> Option<&NeoStruct> {
        match self {
            NeoValue::Struct(s) => Some(s),
            _ => None,
        }
    }
}

impl From<NeoInteger> for NeoValue {
    fn from(value: NeoInteger) -> Self {
        NeoValue::Integer(value)
    }
}

impl From<NeoBoolean> for NeoValue {
    fn from(value: NeoBoolean) -> Self {
        NeoValue::Boolean(value)
    }
}

impl From<NeoByteString> for NeoValue {
    fn from(value: NeoByteString) -> Self {
        NeoValue::ByteString(value)
    }
}

impl From<NeoString> for NeoValue {
    fn from(value: NeoString) -> Self {
        NeoValue::String(value)
    }
}

impl From<NeoArray<NeoValue>> for NeoValue {
    fn from(value: NeoArray<NeoValue>) -> Self {
        NeoValue::Array(value)
    }
}

impl From<NeoMap<NeoValue, NeoValue>> for NeoValue {
    fn from(value: NeoMap<NeoValue, NeoValue>) -> Self {
        NeoValue::Map(value)
    }
}

impl From<NeoStruct> for NeoValue {
    fn from(value: NeoStruct) -> Self {
        NeoValue::Struct(value)
    }
}

/// Neo N3 Iterator type
#[derive(Debug, Clone)]
pub struct NeoIterator<T> {
    data: Vec<T>,
    index: usize,
}

impl<T> NeoIterator<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data, index: 0 }
    }
    
    pub fn next(&mut self) -> Option<T> {
        if self.index < self.data.len() {
            let item = self.data.remove(self.index);
            Some(item)
        } else {
            None
        }
    }
    
    pub fn has_next(&self) -> bool {
        self.index < self.data.len()
    }
}

/// Neo N3 Storage Context type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoStorageContext {
    id: u32,
}

impl NeoStorageContext {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
    
    pub fn id(&self) -> u32 {
        self.id
    }
}

/// Neo N3 Contract Manifest
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoContractManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub email: String,
    pub description: String,
    pub abi: NeoContractABI,
    pub permissions: Vec<NeoContractPermission>,
    pub trusts: Vec<String>,
    pub supported_standards: Vec<String>,
}

/// Neo N3 Contract ABI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoContractABI {
    pub hash: String,
    pub methods: Vec<NeoContractMethod>,
    pub events: Vec<NeoContractEvent>,
}

/// Neo N3 Contract Method
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoContractMethod {
    pub name: String,
    pub parameters: Vec<NeoContractParameter>,
    pub return_type: String,
    pub offset: u32,
    pub safe: bool,
}

/// Neo N3 Contract Event
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoContractEvent {
    pub name: String,
    pub parameters: Vec<NeoContractParameter>,
}

/// Neo N3 Contract Parameter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoContractParameter {
    pub name: String,
    pub r#type: String,
}

/// Neo N3 Contract Permission
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoContractPermission {
    pub contract: String,
    pub methods: Vec<String>,
}

/// Neo N3 Error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeoError {
    InvalidOperation,
    InvalidArgument,
    InvalidType,
    OutOfBounds,
    DivisionByZero,
    Overflow,
    Underflow,
    NullReference,
    InvalidState,
    Custom(String),
}

impl fmt::Display for NeoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NeoError::InvalidOperation => write!(f, "Invalid operation"),
            NeoError::InvalidArgument => write!(f, "Invalid argument"),
            NeoError::InvalidType => write!(f, "Invalid type"),
            NeoError::OutOfBounds => write!(f, "Out of bounds"),
            NeoError::DivisionByZero => write!(f, "Division by zero"),
            NeoError::Overflow => write!(f, "Overflow"),
            NeoError::Underflow => write!(f, "Underflow"),
            NeoError::NullReference => write!(f, "Null reference"),
            NeoError::InvalidState => write!(f, "Invalid state"),
            NeoError::Custom(msg) => write!(f, "Custom error: {}", msg),
        }
    }
}

impl NeoError {
    pub fn new(message: &str) -> Self {
        NeoError::Custom(message.to_string())
    }
    
    pub fn message(&self) -> &str {
        match self {
            NeoError::Custom(msg) => msg,
            _ => "Unknown error",
        }
    }
}

/// Neo N3 Result type
pub type NeoResult<T> = Result<T, NeoError>;

/// Neo N3 Contract trait
pub trait NeoContract {
    fn name() -> &'static str;
    fn version() -> &'static str;
    fn author() -> &'static str;
    fn description() -> &'static str;
}

/// Neo N3 Contract Entry Point
pub trait NeoContractEntry {
    fn deploy() -> NeoResult<()>;
    fn update() -> NeoResult<()>;
    fn destroy() -> NeoResult<()>;
}

/// Neo N3 Contract Method trait
pub trait NeoContractMethodTrait {
    fn name() -> &'static str;
    fn parameters() -> &'static [&'static str];
    fn return_type() -> &'static str;
    fn execute(args: &[NeoValue]) -> NeoResult<NeoValue>;
}

// Default implementations for Neo types
impl Default for NeoInteger {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Default for NeoBoolean {
    fn default() -> Self {
        Self::FALSE
    }
}

impl Default for NeoByteString {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl Default for NeoString {
    fn default() -> Self {
        Self::from_str("")
    }
}
