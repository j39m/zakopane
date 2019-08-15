// This module defines constants and functions for working with zakocmp
// configuration files.

extern crate yaml_rust;

use std::io::{Error, ErrorKind};
use std::result::Result;
use std::vec::Vec;
use yaml_rust::{Yaml, YamlLoader};

const DEFAULT_POLICY_KEY: &'static str = "default-policy";
const POLICIES_KEY: &'static str = "policies";

// Enumerates the string representations of known policies.
const POLICY_REPR_IGNORE: &'static str = "ignore";
const POLICY_REPR_NOADD: &'static str = "noadd";
const POLICY_REPR_NODELETE: &'static str = "nodelete";
const POLICY_REPR_NOMODIFY: &'static str = "nomodify";
const POLICY_REPR_IMMUTABLE: &'static str = "immutable";

// Represents known policies as integers.
pub const POLICY_IGNORE: i32 = 0;
pub const POLICY_NOADD: i32 = 1 << 0;
pub const POLICY_NODELETE: i32 = 1 << 1;
pub const POLICY_NOMODIFY: i32 = 1 << 2;
pub const POLICY_IMMUTABLE: i32 = POLICY_NOADD | POLICY_NODELETE | POLICY_NOMODIFY;

// Represents a sorted vector of zakopane config rules, each mapping a
// path (prefix) to a policy. This type alias is provided for ease of
// coding.
type ZakopanePolicies = Vec<(String, i32)>;

// Represents a zakopane config. Please consult the documentation.
pub struct ZakopaneConfig {
    default_policy: i32,
    policies: ZakopanePolicies,
}

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
/// // Multiple policies are bitwise OR'd together.
/// let policy: i32 = config::policy_repr_as_int(
///     &"noadd,nomodify").unwrap();
/// assert!(policy == config::POLICY_NOADD | config::POLICY_NOMODIFY);
/// ```
///
/// ```
/// # mod zakocmp;
/// // The biggest bitwise OR equals POLICY_IMMUTABLE.
/// let policy: i32 = config::policy_repr_as_int(
///     &"noadd,nodelete,nomodify").unwrap();
/// assert!(policy == config::POLICY_IMMUTABLE);
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
    let policy_ints: Vec<i32> = policy
        .split(",")
        .map(|tok| policy_token_as_int(tok))
        .collect::<Result<Vec<i32>, Error>>()?;
    return Ok(policy_ints
        .iter()
        .fold(POLICY_IGNORE, |accum, elem| accum | elem));
}

// Borrows yaml representations of one line of zakopane policy and
// returns the corresponding valid tuple suitable for use in building a
// ZakopanePolicies object.
fn extract_policy(ypath: &Yaml, policy_repr: &Yaml) -> Result<(String, i32), Error> {
    let path: String = match ypath.as_str() {
        Some(string) => string.to_owned(),
        None => return Err(Error::new(ErrorKind::InvalidData, "malformed path?")),
    };
    let policy: i32 = match policy_repr.as_str() {
        Some(string) => policy_repr_as_int(string)?,
        None => return Err(Error::new(ErrorKind::InvalidData, "malformed policy?")),
    };
    Ok((path, policy))
}

// Borrows the YAML representation of a zakopane config and returns the
// corresponding ZakopanePolicies. The return value can be benignly
// empty (e.g. if the present config elects not to specify any rules).
fn policies_from_yaml(doc: &Yaml) -> Result<ZakopanePolicies, Error> {
    let policies_map_yaml = &doc[POLICIES_KEY];
    if policies_map_yaml.is_badvalue() {
        // Assumes the config may be benignly devoid of specific
        // policies, returning Ok.
        return Ok(vec![]);
    }
    // Otherwise, iterates over the policies map. Each entry in the
    // policies map correlates a path prefix to a comma-separated list
    // of policies.
    let policies_map: &yaml_rust::yaml::Hash = match policies_map_yaml.as_hash() {
        Some(map) => map,
        None => return Err(Error::new(ErrorKind::InvalidData, "malformed policies")),
    };
    let mut policies: ZakopanePolicies = policies_map
        .into_iter()
        .map(|pair| extract_policy(&pair.0, &pair.1))
        .collect::<Result<ZakopanePolicies, Error>>()?;
    policies.sort_unstable_by_key(|pair| pair.0.to_owned());
    Ok(policies)
}

impl ZakopaneConfig {
    // Borrows the string representation of a zakopane config and
    // returns a corresponding ZakopaneConfig.
    pub fn new(config: &str) -> Result<ZakopaneConfig, Error> {
        let docs: Vec<Yaml> = match YamlLoader::load_from_str(config) {
            Ok(val) => val,
            Err(scan_error) => return Err(Error::new(ErrorKind::InvalidData, scan_error)),
        };
        if docs.len() == 0 {
            return Err(Error::new(ErrorKind::InvalidData, "empty zakopane config"));
        }
        let doc = &docs[0];

        let default_policy_yaml = &doc[DEFAULT_POLICY_KEY];
        if default_policy_yaml.is_badvalue() {
            return Err(Error::new(ErrorKind::InvalidData, DEFAULT_POLICY_KEY));
        }
        let default_policy: i32 = match default_policy_yaml.as_str() {
            None => return Err(Error::new(ErrorKind::InvalidData, DEFAULT_POLICY_KEY)),
            Some(token) => policy_repr_as_int(&token),
        }?;

        let policies: ZakopanePolicies = policies_from_yaml(&doc)?;

        Ok(ZakopaneConfig {
            default_policy: default_policy,
            policies: policies,
        })
    }

    // Returns how many rules this config contains.
    // This is never less than 1 as a default-policy is always required.
    pub fn rules(&self) -> usize {
        1 + self.policies.len()
    }

    // Returns the default policy.
    pub fn default_policy(&self) -> i32 {
        self.default_policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_bare_noadd() {
        let policy: i32 = match policy_repr_as_int(&"noadd") {
            Ok(value) => value,
            Err(oof) => panic!(oof),
        };
        assert!(policy == POLICY_NOADD);
    }

    #[test]
    fn test_policy_bare_nodelete() {
        let policy: i32 = match policy_repr_as_int(&"nodelete") {
            Ok(value) => value,
            Err(oof) => panic!(oof),
        };
        assert!(policy == POLICY_NODELETE);
    }

    #[test]
    fn test_policy_bare_nomodify() {
        let policy: i32 = match policy_repr_as_int(&"nomodify") {
            Ok(value) => value,
            Err(oof) => panic!(oof),
        };
        assert!(policy == POLICY_NOMODIFY);
    }

    #[test]
    fn test_policy_combo() {
        let policy: i32 = match policy_repr_as_int(&"noadd,nodelete") {
            Ok(value) => value,
            Err(oof) => panic!(oof),
        };
        assert!(policy == POLICY_NOADD | POLICY_NODELETE);
    }

    #[test]
    fn test_policy_repetition() {
        let policy: i32 =
            match policy_repr_as_int(&"noadd,noadd,noadd,noadd,nodelete,nodelete,nodelete,noadd") {
                Ok(value) => value,
                Err(oof) => panic!(oof),
            };
        assert!(policy == POLICY_NOADD | POLICY_NODELETE);
    }

    #[test]
    fn disallow_empty_config() {
        let config = "";
        assert!(!ZakopaneConfig::new(&config).is_ok());
    }

    #[test]
    fn disallow_obviously_malformed_config() {
        let config = r#"
This is not a zakopane config -
rather, it's two lines of text.
        "#;
        assert!(!ZakopaneConfig::new(&config).is_ok());
    }

    #[test]
    fn config_requires_default_policy() {
        let config = r#"
policies:
    hello-there: nomodify
        "#;
        assert!(!ZakopaneConfig::new(&config).is_ok());

        let mut config_with_default_policy: String = r#"
default-policy: immutable
        "#.to_string();
        config_with_default_policy.push_str(&config);
        assert!(ZakopaneConfig::new(&config_with_default_policy).is_ok())
    }

    #[test]
    fn config_with_no_specific_policies() {
        let config = r#"
default-policy: nodelete
one-irrelevant-key: it doesn't matter what we put here
another-irrelevant-key: this doesn't invalidate the YAML
third-irrelevant-key: so long as it contains a default-policy
        "#;
        assert!(ZakopaneConfig::new(&config).is_ok());
    }

    #[test]
    fn config_with_several_policies() {
        let config = r#"
default-policy: immutable
policies:
    hello-there: noadd
    general-kenobi: nodelete
        "#;
        assert!(ZakopaneConfig::new(&config).is_ok());
    }
}
