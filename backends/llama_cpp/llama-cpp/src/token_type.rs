//! Utilities for working with `llama_token_type` values.

/// A rust flavored equivalent of `llama_token_type`.
#[repr(u32)]
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
#[allow(clippy::module_name_repetitions)]
pub enum LlamaTokenType {
    /// An undefined token type.
    Undefined = llama_cpp_sys::LLAMA_TOKEN_ATTR_UNDEFINED as _,
    /// A normal token type.
    Normal = llama_cpp_sys::LLAMA_TOKEN_ATTR_NORMAL as _,
    /// An unknown token type.
    Unknown = llama_cpp_sys::LLAMA_TOKEN_ATTR_UNKNOWN as _,
    /// A control token type.
    Control = llama_cpp_sys::LLAMA_TOKEN_ATTR_CONTROL as _,
    /// A user defined token type.
    UserDefined = llama_cpp_sys::LLAMA_TOKEN_ATTR_USER_DEFINED as _,
    /// An unused token type.
    Unused = llama_cpp_sys::LLAMA_TOKEN_ATTR_UNUSED as _,
    /// A byte token type.
    Byte = llama_cpp_sys::LLAMA_TOKEN_ATTR_BYTE as _,
}

/// A safe wrapper for converting potentially deceptive `llama_token_type` values into
/// `LlamaVocabType`.
///
/// The error branch returns the original value.
///
/// ```
/// # use std::convert::TryFrom;
/// # use std::ffi::c_int;
/// # use std::num::TryFromIntError;
/// # use std::result::Result;
/// # use llama_cpp_2::token_type::{LlamaTokenTypeFromIntError, LlamaTokenType};
/// # fn main() -> Result<(), LlamaTokenTypeFromIntError> {
/// let llama_token_type = LlamaTokenType::try_from(0 as llama_cpp_sys::llama_token_type)?;
/// assert_eq!(llama_token_type, LlamaTokenType::Undefined);
///
/// let bad_llama_token_type = LlamaTokenType::try_from(100 as llama_cpp_sys::llama_token_type);
/// assert_eq!(Err(LlamaTokenTypeFromIntError::UnknownValue(100)), bad_llama_token_type);
/// # Ok(())
/// # }
impl TryFrom<llama_cpp_sys::llama_token_attr> for LlamaTokenType {
    type Error = LlamaTokenTypeFromIntError;

    fn try_from(value: llama_cpp_sys::llama_vocab_type) -> Result<Self, Self::Error> {
        match value {
            llama_cpp_sys::LLAMA_TOKEN_ATTR_UNDEFINED => Ok(LlamaTokenType::Undefined),
            llama_cpp_sys::LLAMA_TOKEN_ATTR_NORMAL => Ok(LlamaTokenType::Normal),
            llama_cpp_sys::LLAMA_TOKEN_ATTR_UNKNOWN => Ok(LlamaTokenType::Unknown),
            llama_cpp_sys::LLAMA_TOKEN_ATTR_CONTROL => Ok(LlamaTokenType::Control),
            llama_cpp_sys::LLAMA_TOKEN_ATTR_USER_DEFINED => Ok(LlamaTokenType::UserDefined),
            llama_cpp_sys::LLAMA_TOKEN_ATTR_UNUSED => Ok(LlamaTokenType::Unused),
            llama_cpp_sys::LLAMA_TOKEN_ATTR_BYTE => Ok(LlamaTokenType::Byte),
            _ => Err(LlamaTokenTypeFromIntError::UnknownValue(value as _)),
        }
    }
}

/// An error type for `LlamaTokenType::try_from`.
#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum LlamaTokenTypeFromIntError {
    /// The value is not a valid `llama_token_type`.
    #[error("Unknown Value {0}")]
    UnknownValue(std::ffi::c_uint),
}
