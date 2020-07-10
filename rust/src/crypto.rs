use libsodium_sys;

use crate::common::*;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io;
use std::sync::atomic;

// TODO: fix me
#[cfg_attr(feature = "cargo-clippy", allow(replace_consts))]
static mut SODIUM_INIT: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

pub const NETCODE_ENCRYPT_EXTA_BYTES: usize =
    libsodium_sys::crypto_aead_chacha20poly1305_ABYTES as usize;

#[derive(Debug, err_derive::Error)]
pub enum EncryptError {
    #[error(display = "InvalidPublicKeySize")]
    InvalidPublicKeySize,
    #[error(display = "BufferSizeMismatch")]
    BufferSizeMismatch,
    #[error(display = "{}", _0)]
    IO(io::Error),
    #[error(display = "Failed")]
    Failed,
}

impl From<io::Error> for EncryptError {
    fn from(err: io::Error) -> Self {
        EncryptError::IO(err)
    }
}

fn init_sodium() {
    unsafe {
        if SODIUM_INIT.load(atomic::Ordering::Relaxed) == 0 {
            libsodium_sys::sodium_init();
            SODIUM_INIT.store(1, atomic::Ordering::Relaxed);
        }
    }
}

/// Generates a new random private key.
pub fn generate_key() -> [u8; NETCODE_KEY_BYTES] {
    let mut key: [u8; NETCODE_KEY_BYTES] = [0; NETCODE_KEY_BYTES];

    random_bytes(&mut key);

    key
}

pub fn random_bytes(out: &mut [u8]) {
    unsafe {
        init_sodium();
        libsodium_sys::randombytes_buf(out.as_mut_ptr() as _, out.len());
    }
}

// TODO: fix me
#[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
pub fn encode(
    out: &mut [u8],
    data: &[u8],
    additional_data: Option<&[u8]>,
    nonce: u64,
    key: &[u8; NETCODE_KEY_BYTES],
) -> Result<usize, EncryptError> {
    if key.len() != NETCODE_KEY_BYTES {
        return Err(EncryptError::InvalidPublicKeySize);
    }

    if out.len() < data.len() + NETCODE_ENCRYPT_EXTA_BYTES {
        return Err(EncryptError::BufferSizeMismatch);
    }

    let (result, written) = unsafe {
        init_sodium();
        let mut written: u64 = out.len() as u64;

        let mut final_nonce = [0; 12];
        io::Cursor::new(&mut final_nonce[4..]).write_u64::<LittleEndian>(nonce)?;

        let result = libsodium_sys::crypto_aead_chacha20poly1305_ietf_encrypt(
            out.as_mut_ptr(),
            &mut written,
            data.as_ptr(),
            data.len() as u64,
            additional_data.map_or(::std::ptr::null_mut(), |v| v.as_ptr()),
            additional_data.map_or(0, |v| v.len()) as u64,
            ::std::ptr::null(),
            final_nonce.as_ptr(),
            key.as_ptr(),
        );

        (result, written)
    };

    match result {
        -1 => Err(EncryptError::Failed),
        _ => Ok(written as usize),
    }
}

// TODO: fix me
#[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
pub fn decode(
    out: &mut [u8],
    data: &[u8],
    additional_data: Option<&[u8]>,
    nonce: u64,
    key: &[u8; NETCODE_KEY_BYTES],
) -> Result<usize, EncryptError> {
    if key.len() != NETCODE_KEY_BYTES {
        return Err(EncryptError::InvalidPublicKeySize);
    }

    if out.len() < data.len() - NETCODE_ENCRYPT_EXTA_BYTES {
        return Err(EncryptError::BufferSizeMismatch);
    }

    let (result, read) = unsafe {
        init_sodium();
        let mut read: u64 = out.len() as u64;

        let mut final_nonce = [0; 12];
        io::Cursor::new(&mut final_nonce[4..]).write_u64::<LittleEndian>(nonce)?;

        let result = libsodium_sys::crypto_aead_chacha20poly1305_ietf_decrypt(
            out.as_mut_ptr(),
            &mut read,
            ::std::ptr::null_mut(),
            data.as_ptr(),
            data.len() as u64,
            additional_data.map_or(::std::ptr::null_mut(), |v| v.as_ptr()),
            additional_data.map_or(0, |v| v.len()) as u64,
            final_nonce.as_ptr(),
            key.as_ptr(),
        );

        (result, read)
    };

    match result {
        -1 => Err(EncryptError::Failed),
        _ => Ok(read as usize),
    }
}
