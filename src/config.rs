// This module defines constants and functions for working with zakopane
// configuration files.

use std::clone::Clone;

use yaml_rust::{Yaml, YamlLoader};

use crate::structs::ZakopaneError;

type PolicyBitfield = u8;

#[derive(Debug)]
pub struct Policy {
    bitfield: PolicyBitfield,
}

#[repr(u8)]
enum PolicyAsU8 {
    Ignore = 0b000,
    NoAdd = 0b001,
    NoDelete = 0b010,
    NoModify = 0b100,
    Immutable = 0b111,
}

fn policy_int_from(token: &str) -> Result<PolicyBitfield, ZakopaneError> {
    match token {
        "ignore" => Ok(PolicyAsU8::Ignore as u8),
        "noadd" => Ok(PolicyAsU8::NoAdd as u8),
        "nodelete" => Ok(PolicyAsU8::NoDelete as u8),
        "nomodify" => Ok(PolicyAsU8::NoModify as u8),
        "immutable" => Ok(PolicyAsU8::Immutable as u8),
        _ => Err(ZakopaneError::Config(format!("bad token: ``{}''", token))),
    }
}

impl TryFrom<&str> for Policy {
    type Error = crate::structs::ZakopaneError;
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let policy_u8s: Vec<PolicyBitfield> = input
            .split(",")
            .map(|tok| policy_int_from(tok))
            .collect::<Result<Vec<PolicyBitfield>, ZakopaneError>>(
        )?;
        let folded = policy_u8s
            .iter()
            .fold(PolicyAsU8::Ignore as u8, |accum, elem| accum | elem);
        Ok(Policy { bitfield: folded })
    }
}

impl Policy {
    pub fn is_ignore(&self) -> bool {
        self.bitfield == PolicyAsU8::Ignore as u8
    }
    pub fn is_noadd(&self) -> bool {
        (self.bitfield & PolicyAsU8::NoAdd as u8) != 0
    }
    pub fn is_nodelete(&self) -> bool {
        (self.bitfield & PolicyAsU8::NoDelete as u8) != 0
    }
    pub fn is_nomodify(&self) -> bool {
        (self.bitfield & PolicyAsU8::NoModify as u8) != 0
    }
}

// Represents a sorted vector of zakopane config rules, each mapping a
// path (prefix) to a policy. This type alias is provided for ease of
// coding.
type Policies = Vec<(String, Policy)>;

const DEFAULT_POLICY_KEY: &'static str = "default-policy";
const POLICIES_KEY: &'static str = "policies";

// Represents a zakopane config. Please consult the documentation.
pub struct Config {
    default_policy: Policy,
    policies: Policies,
}

// Borrows yaml representations of one line of zakopane policy and
// returns the corresponding valid tuple suitable for use in building a
// Policies object.
fn policy_tuple_from_yaml(
    ypath: &Yaml,
    policy_tokens: &Yaml,
) -> Result<(String, Policy), ZakopaneError> {
    let path: String = match ypath.as_str() {
        Some(string) => string.to_owned(),
        None => return Err(ZakopaneError::Config("malformed path?".to_string())),
    };
    let policy: Policy = match policy_tokens.as_str() {
        Some(string) => Policy::try_from(string)?,
        None => return Err(ZakopaneError::Config("malformed policy?".to_string())),
    };
    Ok((path, policy))
}

// Borrows the YAML representation of a zakopane config and returns the
// corresponding Policies. The return value can be benignly
// empty (e.g. if the present config elects not to specify any rules).
fn policies_from_yaml(doc: &Yaml) -> Result<Policies, ZakopaneError> {
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
        None => return Err(ZakopaneError::Config("malformed policies".to_string())),
    };
    let mut policies: Policies = policies_map
        .into_iter()
        .map(|pair| policy_tuple_from_yaml(&pair.0, &pair.1))
        .collect::<Result<Policies, ZakopaneError>>()?;
    policies.sort_unstable_by_key(|pair| pair.0.to_owned());
    Ok(policies)
}

// Borrows the YAML representation of a zakopane config and returns the
// integral default-policy defined within.
fn default_policy_from_yaml(doc: &Yaml) -> Result<Option<Policy>, ZakopaneError> {
    let default_policy_yaml = &doc[DEFAULT_POLICY_KEY];
    if default_policy_yaml.is_badvalue() {
        return Ok(None);
    }
    let default_policy: Policy = match default_policy_yaml.as_str() {
        None => return Err(ZakopaneError::Config(DEFAULT_POLICY_KEY.to_string())),
        Some(token) => Policy::try_from(token),
    }?;
    Ok(Some(default_policy))
}

// Interprets |config_contents| as YAML and returns the first document
// within (if present).
fn read_yaml(config_contents: &str) -> Result<Option<Yaml>, ZakopaneError> {
    let docs: Vec<Yaml> = YamlLoader::load_from_str(&config_contents).map_err(
        |scan_error: yaml_rust::ScanError| ZakopaneError::Config(scan_error.to_string()),
    )?;
    // Explicitly allow empty configs.
    if docs.len() == 0 {
        return Ok(None);
    }
    Ok(Some(docs[0].clone()))
}

// Returns the default policy for this invocation.
fn get_default_policy(yaml_config: &Option<Yaml>) -> Result<Policy, ZakopaneError> {
    if let Some(yaml) = yaml_config {
        if let Some(default_policy) = default_policy_from_yaml(&yaml)? {
            return Ok(default_policy);
        }
    }
    Ok(Policy {
        bitfield: PolicyAsU8::Immutable as u8,
    })
}

// Returns any additional policies for this invocation.
fn get_policies(yaml_config: &Option<Yaml>) -> Result<Policies, ZakopaneError> {
    match yaml_config {
        Some(doc) => policies_from_yaml(doc),
        None => Ok(Policies::new()),
    }
}

impl Config {
    // Borrows the string representation of a zakopane config and
    // returns a corresponding Config.
    pub fn new(config_path: Option<std::path::PathBuf>) -> Result<Config, ZakopaneError> {
        let yaml_config: Option<Yaml> = match config_path {
            Some(path) => {
                let config = crate::helpers::ingest_file(&path)?;
                read_yaml(&config)?
            }
            None => None,
        };

        let default_policy = get_default_policy(&yaml_config)?;
        let policies = get_policies(&yaml_config)?;

        Ok(Config {
            default_policy: default_policy,
            policies: policies,
        })
    }

    // Returns how many rules this config contains.
    // This is never less than 1 as a default-policy is always present.
    pub fn rules(&self) -> usize {
        1 + self.policies.len()
    }

    pub fn match_policy(&self, path: &str) -> &Policy {
        let mut best_match_path: &str = "";
        let mut best_match_policy: Option<&Policy> = None;
        for (prefix, policy) in self.policies.iter() {
            if path.starts_with(prefix) && prefix.len() > best_match_path.len() {
                best_match_path = prefix;
                best_match_policy = Some(policy);
            }
        }
        if best_match_policy.is_some() {
            return best_match_policy.unwrap();
        }
        &self.default_policy
    }
}

pub mod test_support {
    // Returns |path| with the cargo test data directory prepended.
    pub fn data_path(path: &str) -> std::path::PathBuf {
        let mut result = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        result.push("tests");
        result.push("config-test-data");
        result.push(path);
        result
    }
} // pub mod test_support

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_token_bare() {
        let policy = Policy::try_from("noadd").unwrap();
        assert!(policy.is_noadd());

        let policy = Policy::try_from("nodelete").unwrap();
        assert!(policy.is_nodelete());

        let policy = Policy::try_from("nomodify").unwrap();
        assert!(policy.is_nomodify());
    }

    #[test]
    fn policy_tokens_can_combo() {
        let policy = Policy::try_from("noadd,nodelete").unwrap();
        assert!(policy.is_noadd());
        assert!(policy.is_nodelete());
    }

    #[test]
    fn policy_tokens_can_repeat() {
        let policy =
            Policy::try_from("noadd,noadd,noadd,noadd,nodelete,nodelete,nodelete,noadd").unwrap();
        assert!(policy.is_noadd());
        assert!(policy.is_nodelete());
    }

    #[test]
    fn config_can_contain_anything() {
        // This...might not be the best behavior to go for.
        let config_path = test_support::data_path("flagrantly-invalid-yaml");
        let config = Config::new(Some(config_path)).unwrap();
        assert_eq!(config.rules(), 1);
    }

    #[test]
    fn config_can_be_empty() {
        // An empty config file is valid (albeit trivial) YAML and is
        // considered valid.
        let config = Config::new(Some(std::path::PathBuf::from("/dev/null"))).unwrap();

        assert!(config.default_policy.is_noadd());
        assert!(config.default_policy.is_nodelete());
        assert!(config.default_policy.is_nomodify());
    }

    #[test]
    fn config_can_omit_default_policy() {
        // A config file without a default policy is valid.
        let config_path = test_support::data_path("config-without-default-policy");
        let config = Config::new(Some(config_path)).unwrap();
        assert_eq!(config.rules(), 5);

        assert!(config.default_policy.is_noadd());
        assert!(config.default_policy.is_nodelete());
        assert!(config.default_policy.is_nomodify());
    }

    #[test]
    fn config_might_not_have_specific_policies() {
        let config_path = test_support::data_path("config-without-specific-policies");
        let config = Config::new(Some(config_path)).unwrap();
        assert!(config.rules() == 1);
        assert!(config.match_policy("").is_nodelete());
    }

    #[test]
    fn config_policies_must_be_a_map() {
        let config_path = test_support::data_path("config-with-ill-formed-policies");
        assert!(Config::new(Some(config_path)).is_err());
    }

    #[test]
    fn match_default_policy() {
        let config_path = test_support::data_path("config-without-specific-policies");
        let config = Config::new(Some(config_path)).unwrap();

        // With only a default policy, this config has just 1 rule.
        assert_eq!(config.rules(), 1);

        // Any path prefix we throw at match_policy() shall come up
        // as the default policy.
        assert!(config
            .match_policy("./Documents/hello/there.txt")
            .is_nodelete());
        assert!(config
            .match_policy("./Music/general/kenobi.txt")
            .is_nodelete());
    }

    #[test]
    fn match_nondefault_policies() {
        let config_path = test_support::data_path("config-with-several-policies");
        let config = Config::new(Some(config_path)).unwrap();

        assert_eq!(config.rules(), 5);

        // Falls back on the default-policy absent any specific policy
        // defined for this file.
        let policy = config.match_policy("./Documents/catch-me-senpai.txt");
        assert!(policy.is_noadd());
        assert!(policy.is_nodelete());
        assert!(policy.is_nomodify());
        // Matches only ``./Pictures.''
        assert!(config.match_policy("./Pictures/2016/yano.jpg").is_noadd());
        // As above and does _not_ match ``./Pictures/2019/third-party/.''
        assert!(config
            .match_policy("./Pictures/2019/first-party.jpg")
            .is_noadd());
        // Does match ``./Pictures/2019/third-party/.''
        assert!(config
            .match_policy("./Pictures/2019/third-party/yano.jpg")
            .is_nodelete());

        // Path prefix matching is done strictly and exactly;
        // ``food.md'' doesn't match ``food/,'' so there's no risk of
        // zakopane confusing cohabiting entities with similar basenames.
        assert!(config.match_policy("./Pictures/2020/food.md").is_nomodify());
        let policy = config.match_policy("./Pictures/2020/food/tacos.jpg");
        assert!(policy.is_nodelete());
        assert!(policy.is_nomodify());
    }
}
