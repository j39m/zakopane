// This module defines constants and functions for working with zakocmp
// configuration files.

use std::clone::Clone;
use std::error::Error;

use yaml_rust::{Yaml, YamlLoader};

use crate::structs::CliOptions;
use crate::structs::ZakocmpError;

// Represents a single zakopane config policy.
type Policy = i32;

// Represents a sorted vector of zakopane config rules, each mapping a
// path (prefix) to a policy. This type alias is provided for ease of
// coding.
type Policies = Vec<(String, Policy)>;

const DEFAULT_POLICY_KEY: &'static str = "default-policy";
const POLICIES_KEY: &'static str = "policies";

// Enumerates the string representations of known policies.
const POLICY_REPR_IGNORE: &'static str = "ignore";
const POLICY_REPR_NOADD: &'static str = "noadd";
const POLICY_REPR_NODELETE: &'static str = "nodelete";
const POLICY_REPR_NOMODIFY: &'static str = "nomodify";
const POLICY_REPR_IMMUTABLE: &'static str = "immutable";

// Represents known policies as an integral type.
pub const POLICY_IGNORE: Policy = 0;
pub const POLICY_NOADD: Policy = 1 << 0;
pub const POLICY_NODELETE: Policy = 1 << 1;
pub const POLICY_NOMODIFY: Policy = 1 << 2;
pub const POLICY_IMMUTABLE: Policy = POLICY_NOADD | POLICY_NODELETE | POLICY_NOMODIFY;

// Represents a zakopane config. Please consult the documentation.
pub struct Config {
    default_policy: Policy,
    policies: Policies,
}

// Borrows the string representation of one policy `token` and returns
// the equivalent integral representation.
fn policy_token_as_int(token: &str) -> Result<Policy, ZakocmpError> {
    match token {
        POLICY_REPR_IGNORE => Ok(POLICY_IGNORE),
        POLICY_REPR_NOADD => Ok(POLICY_NOADD),
        POLICY_REPR_NODELETE => Ok(POLICY_NODELETE),
        POLICY_REPR_NOMODIFY => Ok(POLICY_NOMODIFY),
        POLICY_REPR_IMMUTABLE => Ok(POLICY_IMMUTABLE),
        _ => Err(ZakocmpError::Config(format!("bad token: ``{}''", token))),
    }
}

// Borrows the string representation of a combined `policy` and returns
// the equivalent integral representation. This function expects
// `policy` to comprise one or more policy tokens separated by commas.
fn policy_tokens_as_int(policy: &str) -> Result<Policy, ZakocmpError> {
    let policy_ints: Vec<Policy> = policy
        .split(",")
        .map(|tok| policy_token_as_int(tok))
        .collect::<Result<Vec<Policy>, ZakocmpError>>()?;
    Ok(policy_ints
        .iter()
        .fold(POLICY_IGNORE, |accum, elem| accum | elem))
}

// Borrows yaml representations of one line of zakopane policy and
// returns the corresponding valid tuple suitable for use in building a
// Policies object.
fn policy_tuple_from_yaml(
    ypath: &Yaml,
    policy_tokens: &Yaml,
) -> Result<(String, Policy), ZakocmpError> {
    let path: String = match ypath.as_str() {
        Some(string) => string.to_owned(),
        None => return Err(ZakocmpError::Config("malformed path?".to_string())),
    };
    let policy: Policy = match policy_tokens.as_str() {
        Some(string) => policy_tokens_as_int(string)?,
        None => return Err(ZakocmpError::Config("malformed policy?".to_string())),
    };
    Ok((path, policy))
}

// Borrows the YAML representation of a zakopane config and returns the
// corresponding Policies. The return value can be benignly
// empty (e.g. if the present config elects not to specify any rules).
fn policies_from_yaml(doc: &Yaml) -> Result<Policies, ZakocmpError> {
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
        None => return Err(ZakocmpError::Config("malformed policies".to_string())),
    };
    let mut policies: Policies = policies_map
        .into_iter()
        .map(|pair| policy_tuple_from_yaml(&pair.0, &pair.1))
        .collect::<Result<Policies, ZakocmpError>>()?;
    policies.sort_unstable_by_key(|pair| pair.0.to_owned());
    Ok(policies)
}

// Borrows the YAML representation of a zakopane config and returns the
// integral default-policy defined within.
fn default_policy_from_yaml(doc: &Yaml) -> Result<Policy, ZakocmpError> {
    let default_policy_yaml = &doc[DEFAULT_POLICY_KEY];
    if default_policy_yaml.is_badvalue() {
        return Err(ZakocmpError::Config(DEFAULT_POLICY_KEY.to_string()));
    }
    let default_policy: Policy = match default_policy_yaml.as_str() {
        None => return Err(ZakocmpError::Config(DEFAULT_POLICY_KEY.to_string())),
        Some(token) => policy_tokens_as_int(&token),
    }?;
    Ok(default_policy)
}

// Interprets |config_contents| as YAML and returns the first document
// within (if present).
fn read_yaml(config_contents: &str) -> Result<Option<Yaml>, ZakocmpError> {
    let docs: Vec<Yaml> = YamlLoader::load_from_str(&config_contents).map_err(
        |scan_error: yaml_rust::ScanError| {
            ZakocmpError::Config(scan_error.description().to_string())
        },
    )?;
    // Explicitly allow empty configs.
    if docs.len() == 0 {
        return Ok(None);
    }
    Ok(Some(docs[0].clone()))
}

// Returns the default policy for this invocation.
fn get_default_policy(
    options: &CliOptions,
    yaml_config: &Option<Yaml>,
) -> Result<Policy, ZakocmpError> {
    if let Some(default_from_cli) = options.default_policy {
        return policy_tokens_as_int(default_from_cli);
    } else if let Some(yaml) = yaml_config {
        return default_policy_from_yaml(yaml);
    }
    Ok(POLICY_IMMUTABLE)
}

// Returns any additional policies for this invocation.
fn get_policies(yaml_config: &Option<Yaml>) -> Result<Policies, ZakocmpError> {
    match yaml_config {
        Some(doc) => policies_from_yaml(doc),
        None => Ok(Policies::new()),
    }
}

impl Config {
    // Borrows the string representation of a zakopane config and
    // returns a corresponding Config.
    pub fn new(options: &CliOptions) -> Result<Config, ZakocmpError> {
        let yaml_config: Option<Yaml> = match options.config_path {
            Some(path) => {
                let config = crate::helpers::ingest_file(path)?;
                read_yaml(&config)?
            }
            None => None,
        };

        let default_policy = get_default_policy(&options, &yaml_config)?;
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

    // Borrows a `path` and returns the best-matched policy that
    // applies. This function returns an owned tuple of the
    // (closest-matched path expression, integral policy).
    //
    // This function represents the default-policy fallback by
    // returning the tuple consisting of an empty &str and the
    // default policy.
    pub fn match_policy(&self, path: &str) -> (&str, Policy) {
        let mut best_match_path: &str = "";
        let mut best_match_policy: Policy = 0;
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

pub mod test_support {
    use crate::structs::CliOptions;
    use std::path::PathBuf;

    // Creates a CliOptions instance for testing.
    pub fn options<'a>(
        config_path: Option<&'a str>,
        default_policy: Option<&'a str>,
    ) -> CliOptions<'a> {
        CliOptions {
            config_path: config_path,
            default_policy: default_policy,
        }
    }

    // Returns |path| with the cargo test data directory prepended.
    pub fn data_path(path: &str) -> PathBuf {
        let mut result = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        result.push("config-test-data/");
        result.push(path);
        result
    }
} // pub mod test_support

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_token_bare() {
        let policy: Policy = policy_tokens_as_int(&"noadd").unwrap();
        assert_eq!(policy, POLICY_NOADD);

        let policy: Policy = policy_tokens_as_int(&"nodelete").unwrap();
        assert_eq!(policy, POLICY_NODELETE);

        let policy: Policy = policy_tokens_as_int(&"nomodify").unwrap();
        assert_eq!(policy, POLICY_NOMODIFY);
    }

    #[test]
    fn policy_tokens_can_combo() {
        let policy: Policy = policy_tokens_as_int(&"noadd,nodelete").unwrap();
        assert_eq!(policy, POLICY_NOADD | POLICY_NODELETE);
    }

    #[test]
    fn policy_tokens_can_repeat() {
        let policy: Policy =
            policy_tokens_as_int(&"noadd,noadd,noadd,noadd,nodelete,nodelete,nodelete,noadd")
                .unwrap();
        assert_eq!(policy, POLICY_NOADD | POLICY_NODELETE);
    }

    #[test]
    fn config_must_not_be_obviously_malformed() {
        let config_path = test_support::data_path("flagrantly-invalid-yaml");
        let options = test_support::options(Some(config_path.to_str().unwrap()), None);
        assert!(Config::new(&options).is_err());
    }

    #[test]
    fn config_has_default_policy() {
        // In case no configuration is provided at all, the Config
        // struct is still well-defined.
        let empty_options = test_support::options(None, None);
        let unopinionated_config = Config::new(&empty_options).unwrap();
        assert!(unopinionated_config.rules() == 1);
        assert!(unopinionated_config.match_policy("") == ("", POLICY_IMMUTABLE));

        // Tests that a default policy presented on the command-line
        // takes precedence over a written default policy.
        let config_path = test_support::data_path("config-with-default-and-extra-policy");
        // Simulates an invocation in which "noadd" was given as the
        // default policy. The referenced config file uses "ignore"
        // ATOW, and the command-line "noadd" will win aganist it.
        let default_policy_on_cli_options =
            test_support::options(Some(config_path.to_str().unwrap()), Some("noadd"));
        let noadd_is_default = Config::new(&default_policy_on_cli_options).unwrap();
        assert!(noadd_is_default.rules() == 2);
        assert!(noadd_is_default.match_policy("") == ("", POLICY_NOADD));
        assert!(
            noadd_is_default.match_policy("hello/there/general-kenobi")
                == ("hello/there", POLICY_IMMUTABLE)
        );

        // Tests that a written default policy emerges absent explicit
        // specification on the command-line.
        let default_policy_in_yaml_options =
            test_support::options(Some(config_path.to_str().unwrap()), None);
        let ignore_is_default = Config::new(&default_policy_in_yaml_options).unwrap();
        assert!(ignore_is_default.rules() == 2);
        assert!(ignore_is_default.match_policy("") == ("", POLICY_IGNORE));
        assert!(
            noadd_is_default.match_policy("hello/there/general-kenobi")
                == ("hello/there", POLICY_IMMUTABLE)
        );
    }

    #[test]
    fn config_might_not_have_specific_policies() {
        let config_path = test_support::data_path("config-without-specific-policies");
        let options = test_support::options(Some(config_path.to_str().unwrap()), None);
        let config = Config::new(&options).unwrap();
        assert!(config.rules() == 1);
        assert!(config.match_policy("") == ("", POLICY_NODELETE));
    }

    #[test]
    fn config_policies_must_be_a_map() {
        let config_path = test_support::data_path("config-with-ill-formed-policies");
        let options = test_support::options(Some(config_path.to_str().unwrap()), None);
        assert!(Config::new(&options).is_err());
    }

    #[test]
    fn match_default_policy() {
        let config_path = test_support::data_path("config-without-specific-policies");
        let options = test_support::options(Some(config_path.to_str().unwrap()), None);
        let config = Config::new(&options).unwrap();

        // With only a default policy, this config has just 1 rule.
        assert_eq!(config.rules(), 1);

        // Any path prefix we throw at match_policy() shall come up
        // as the default policy.
        let (_path, policy) = config.match_policy("./Documents/hello/there.txt");
        assert_eq!(policy, POLICY_NODELETE);
        let (_path, policy) = config.match_policy("./Music/general/kenobi.txt");
        assert_eq!(policy, POLICY_NODELETE);
    }

    #[test]
    fn match_nondefault_policies() {
        let config_path = test_support::data_path("config-with-several-policies");
        let options = test_support::options(Some(config_path.to_str().unwrap()), None);
        let config = Config::new(&options).unwrap();

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
