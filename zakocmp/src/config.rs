// This module defines constants and functions for working with zakocmp
// configuration files.

extern crate yaml_rust;

use std::result::Result;
use std::string::String;
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
type Policies = Vec<(String, i32)>;

// Represents a zakopane config. Please consult the documentation.
pub struct Config {
    default_policy: i32,
    policies: Policies,
}

// Borrows the string representation of one policy `token` and returns
// the equivalent integral representation.
fn policy_token_as_int(token: &str) -> Result<i32, std::io::Error> {
    match token {
        POLICY_REPR_IGNORE => Ok(POLICY_IGNORE),
        POLICY_REPR_NOADD => Ok(POLICY_NOADD),
        POLICY_REPR_NODELETE => Ok(POLICY_NODELETE),
        POLICY_REPR_NOMODIFY => Ok(POLICY_NOMODIFY),
        POLICY_REPR_IMMUTABLE => Ok(POLICY_IMMUTABLE),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("bad token: ``{}''", token),
        )),
    }
}

// Borrows the string representation of a combined `policy` and returns
// the equivalent integral representation. This function expects
// `policy` to comprise one or more policy tokens separated by commas.
fn policy_tokens_as_int(policy: &str) -> Result<i32, std::io::Error> {
    let policy_ints: Vec<i32> = policy
        .split(",")
        .map(|tok| policy_token_as_int(tok))
        .collect::<Result<Vec<i32>, std::io::Error>>()?;
    return Ok(policy_ints
        .iter()
        .fold(POLICY_IGNORE, |accum, elem| accum | elem));
}

// Borrows yaml representations of one line of zakopane policy and
// returns the corresponding valid tuple suitable for use in building a
// Policies object.
fn extract_policy(ypath: &Yaml, policy_tokens: &Yaml) -> Result<(String, i32), std::io::Error> {
    let path: String = match ypath.as_str() {
        Some(string) => string.to_owned(),
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "malformed path?",
            ))
        }
    };
    let policy: i32 = match policy_tokens.as_str() {
        Some(string) => policy_tokens_as_int(string)?,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "malformed policy?",
            ))
        }
    };
    Ok((path, policy))
}

// Borrows the YAML representation of a zakopane config and returns the
// corresponding Policies. The return value can be benignly
// empty (e.g. if the present config elects not to specify any rules).
fn policies_from_yaml(doc: &Yaml) -> Result<Policies, std::io::Error> {
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
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "malformed policies",
            ))
        }
    };
    let mut policies: Policies = policies_map
        .into_iter()
        .map(|pair| extract_policy(&pair.0, &pair.1))
        .collect::<Result<Policies, std::io::Error>>()?;
    policies.sort_unstable_by_key(|pair| pair.0.to_owned());
    Ok(policies)
}

// Borrows the YAML representation of a zakopane config and returns the
// integral default-policy defined within.
fn default_policy_from_yaml(doc: &Yaml) -> Result<i32, std::io::Error> {
    let default_policy_yaml = &doc[DEFAULT_POLICY_KEY];
    if default_policy_yaml.is_badvalue() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            DEFAULT_POLICY_KEY,
        ));
    }
    let default_policy: i32 = match default_policy_yaml.as_str() {
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                DEFAULT_POLICY_KEY,
            ))
        }
        Some(token) => policy_tokens_as_int(&token),
    }?;
    Ok(default_policy)
}

impl Config {
    // Borrows the string representation of a zakopane config and
    // returns a corresponding Config.
    pub fn new(config: &str) -> Result<Config, std::io::Error> {
        let docs: Vec<Yaml> = match YamlLoader::load_from_str(config) {
            Ok(val) => val,
            Err(scan_error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    scan_error,
                ))
            }
        };
        if docs.len() == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "empty zakopane config",
            ));
        }
        let doc = &docs[0];

        let default_policy = default_policy_from_yaml(&doc)?;
        let policies: Policies = policies_from_yaml(&doc)?;

        Ok(Config {
            default_policy: default_policy,
            policies: policies,
        })
    }

    // Returns how many rules this config contains.
    // This is never less than 1 as a default-policy is always required.
    pub fn rules(&self) -> usize {
        1 + self.policies.len()
    }

    // Borrows a `path` and returns the best-matched policy that
    // applies. This function returns an owned tuple of the
    // (closest-matched path expression, integral policy).
    //
    // This function represents the default-policy fallback by
    // returning the tuple consisting of an empty &str and the
    // default policy.
    pub fn match_policy(&self, path: &str) -> (&str, i32) {
        let mut best_match_path: &str = "";
        let mut best_match_policy: i32 = 0;
        for (prefix, policy) in self.policies.iter() {
            if path.starts_with(prefix) && prefix.len() > best_match_path.len() {
                best_match_path = prefix;
                best_match_policy = *policy;
            }
        }
        if best_match_path.len() == 0 {
            return ("", self.default_policy);
        }
        return (best_match_path, best_match_policy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_token_bare() {
        let policy: i32 = policy_tokens_as_int(&"noadd").unwrap();
        assert_eq!(policy, POLICY_NOADD);

        let policy: i32 = policy_tokens_as_int(&"nodelete").unwrap();
        assert_eq!(policy, POLICY_NODELETE);

        let policy: i32 = policy_tokens_as_int(&"nomodify").unwrap();
        assert_eq!(policy, POLICY_NOMODIFY);
    }

    #[test]
    fn policy_tokens_can_combo() {
        let policy: i32 = policy_tokens_as_int(&"noadd,nodelete").unwrap();
        assert_eq!(policy, POLICY_NOADD | POLICY_NODELETE);
    }

    #[test]
    fn policy_tokens_can_repeat() {
        let policy: i32 =
            policy_tokens_as_int(&"noadd,noadd,noadd,noadd,nodelete,nodelete,nodelete,noadd")
                .unwrap();
        assert_eq!(policy, POLICY_NOADD | POLICY_NODELETE);
    }

    #[test]
    fn config_must_not_be_empty() {
        let config = "";
        assert!(Config::new(&config).is_err());
    }

    #[test]
    fn config_must_not_be_obviously_malformed() {
        let config = r#"
This is not a zakopane config -
rather, it's two lines of text.
        "#;
        assert!(Config::new(&config).is_err());
    }

    #[test]
    fn config_requires_default_policy() {
        let config = r#"
policies:
    hello-there: nomodify
        "#;
        assert!(Config::new(&config).is_err());

        let mut config_with_default_policy: String = r#"
default-policy: immutable
        "#
        .to_string();
        config_with_default_policy.push_str(&config);
        assert!(Config::new(&config_with_default_policy).is_ok())
    }

    #[test]
    fn config_might_not_have_specific_policies() {
        let config = r#"
default-policy: nodelete
one-irrelevant-key: it doesn't matter what we put here
another-irrelevant-key: this doesn't invalidate the YAML
third-irrelevant-key: so long as it contains a default-policy
        "#;
        assert!(Config::new(&config).is_ok());
    }

    #[test]
    fn config_policies_must_be_a_map() {
        let config = r#"
default-policy: noadd
policies:
    -   eh?
    -   this ain't a map
        "#;
        assert!(Config::new(&config).is_err());
    }

    #[test]
    fn config_can_have_several_policies() {
        let config = r#"
default-policy: immutable
policies:
    hello-there: noadd
    general-kenobi: nodelete
        "#;
        assert!(Config::new(&config).is_ok());
    }

    #[test]
    fn match_default_policy() {
        let config_yaml = r#"
default-policy: noadd
        "#;
        let config = Config::new(&config_yaml).unwrap();

        // With only a default policy, this config has just 1 rule.
        assert_eq!(config.rules(), 1);

        // Any path prefix we throw at match_policy() shall come up
        // as the default policy.
        let (_path, policy) = config.match_policy("./Documents/hello/there.txt");
        assert_eq!(policy, POLICY_NOADD);
        let (_path, policy) = config.match_policy("./Music/general/kenobi.txt");
        assert_eq!(policy, POLICY_NOADD);
    }

    #[test]
    fn match_nondefault_policies() {
        let config_yaml = r#"
default-policy: immutable
policies:
    ./Pictures/: noadd
    ./Pictures/2019/third-party/: nodelete
    ./Pictures/2020/: nomodify
    ./Pictures/2020/food/: nodelete,nomodify
        "#;
        let config = Config::new(&config_yaml).unwrap();

        assert_eq!(config.rules(), 5);

        // Falls back on the default-policy absent any specific policy
        // defined for this file.
        let (_path, policy) = config.match_policy("./Documents/catch-me-senpai.txt");
        assert_eq!(policy, POLICY_IMMUTABLE);
        // Matches only ``./Pictures.''
        let (_path, policy) = config.match_policy("./Pictures/2016/yano.jpg");
        assert_eq!(policy, POLICY_NOADD);
        // As above and does _not_ match ``./Pictures/2019/third-party/.''
        let (_path, policy) = config.match_policy("./Pictures/2019/first-party.jpg");
        assert_eq!(policy, POLICY_NOADD);
        // Does match ``./Pictures/2019/third-party/.''
        let (_path, policy) = config.match_policy("./Pictures/2019/third-party/yano.jpg");
        assert_eq!(policy, POLICY_NODELETE);

        // Path prefix matching is done strictly and exactly;
        // ``food.md'' doesn't match ``food/,'' so there's no risk of
        // zakopane confusing cohabiting entities with similar basenames.
        let (path, policy) = config.match_policy("./Pictures/2020/food.md");
        assert_eq!(policy, POLICY_NOMODIFY);
        assert_eq!(path, "./Pictures/2020/");
        let (path, policy) = config.match_policy("./Pictures/2020/food/tacos.jpg");
        assert_eq!(policy, POLICY_NODELETE | POLICY_NOMODIFY);
        assert_eq!(path, "./Pictures/2020/food/");
    }
}
