// This module defines constants and functions for working with zakocmp
// configuration files.

use std::io::{Error, ErrorKind};
use std::result::Result;

// Enumerates the string representations of known policies.
const POLICY_REPR_IGNORE: &str = "ignore";
const POLICY_REPR_NOADD: &str = "noadd";
const POLICY_REPR_NODELETE: &str = "nodelete";
const POLICY_REPR_NOMODIFY: &str = "nomodify";
const POLICY_REPR_IMMUTABLE: &str = "immutable";

// Represents known policies as integers.
pub const POLICY_IGNORE: i32 = 0;
pub const POLICY_NOADD: i32 = 1 << 0;
pub const POLICY_NODELETE: i32 = 1 << 1;
pub const POLICY_NOMODIFY: i32 = 1 << 2;
pub const POLICY_IMMUTABLE: i32 = POLICY_NOADD | POLICY_NODELETE | POLICY_NOMODIFY;

// Borrows the string representation of one policy `token` and returns
// the equivalent integral representation.
fn policy_token_as_int(token: &str) -> Result<i32, Error> {
    match token {
        POLICY_REPR_IGNORE => Ok(POLICY_IGNORE),
        POLICY_REPR_NOADD => Ok(POLICY_NOADD),
        POLICY_REPR_NODELETE => Ok(POLICY_NODELETE),
        POLICY_REPR_NOMODIFY => Ok(POLICY_NOMODIFY),
        POLICY_REPR_IMMUTABLE => Ok(POLICY_IMMUTABLE),
        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("bad token: ``{}''", token),
        )),
    }
}

/// Borrows the string representation of a combined `policy` and returns
/// the equivalent integral representation. This function expects
/// `policy` to comprise one or more policy tokens separated by commas.
///
/// # Examples
///
/// ```
/// # mod zakocmp;
/// let policy: i32 = config::policy_repr_as_int(&"immutable").unwrap();
/// assert!(policy == config::POLICY_IMMUTABLE);
/// ```
///
/// ```
/// # mod zakocmp;
/// let policy: i32 = config::policy_repr_as_int(&"noadd").unwrap();
/// assert!(policy == config::POLICY_NOADD);
/// ```
///
/// ```
/// # mod zakocmp;
/// let policy: i32 = config::policy_repr_as_int(&"nomodify").unwrap();
/// assert!(policy == config::POLICY_NOMODIFY);
/// ```
///
/// ```
/// # mod zakocmp;
/// let policy: i32 = config::policy_repr_as_int(&"nodelete").unwrap();
/// assert!(policy == config::POLICY_NODELETE);
/// ```
///
/// ```
/// # mod zakocmp;
/// // While it's pointless to repeat oneself,
/// // it's not invalid to do so.
/// let policy: i32 = config::policy_repr_as_int(
///     &"immutable,immutable,immutable").unwrap();
/// assert!(policy == config::POLICY_IMMUTABLE);
/// ```
///
/// ```
/// # mod zakocmp;
/// let policy: i32 = config::policy_repr_as_int(
///     &"noadd,nodelete,nomodify").unwrap();
/// assert!(policy == config::POLICY_IMMUTABLE);
/// ```
///
/// ```
/// # mod zakocmp;
/// let policy: i32 = config::policy_repr_as_int(
///     &"noadd,nomodify").unwrap();
/// assert!(policy == config::POLICY_NOADD | config::POLICY_NOMODIFY);
/// ```
///
/// # Failures
///
/// ```
/// # mod zakocmp;
/// // Obi-wan was witty, but not good at dictating zakocmp policy.
/// assert!(!config::policy_repr_as_int(&"hello there!").is_ok());
/// ```
///
pub fn policy_repr_as_int(policy: &str) -> Result<i32, Error> {
    let mut policy_int: i32 = POLICY_IGNORE;
    for token in policy.split(",") {
        policy_int |= policy_token_as_int(token)?;
    }
    return Ok(policy_int);
}
