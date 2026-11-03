// SPDX-License-Identifier: GPL-3.0-or-later
//! The settings of the library. Every tuned value is a line of the settings file; a missing line keeps the conservative default.

use crate::error::VerifyError;

/// The settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The most bytes a results file may have.
    pub max_file_bytes: u64,
    /// The most findings a results file may hold.
    pub max_findings: usize,
    /// The areas that must have been run for a verdict to be a pass (a missing one makes the verdict inconclusive).
    pub required_areas: Vec<String>,
    /// A warning that is confirmed also makes the verdict a failure.
    pub fail_on_warning: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_file_bytes: 64 * 1024 * 1024,
            max_findings: 1_000_000,
            required_areas: ["schematic", "board", "rules", "manufacturing"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            fail_on_warning: false,
        }
    }
}

fn flag(text: &str) -> Option<bool> {
    match text {
        "true" | "on" | "yes" => Some(true),
        "false" | "off" | "no" => Some(false),
        _ => None,
    }
}

impl Config {
    /// Parse a settings file (`key = value` lines, `#` comments).
    pub fn parse(text: &str) -> Result<Config, VerifyError> {
        let mut config = Config::default();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let (key, value) = raw
                .split_once('=')
                .ok_or(VerifyError::ConfigSyntax { line })?;
            let (key, value) = (key.trim(), value.trim());
            let bad = || VerifyError::ConfigBadValue {
                line,
                key: key.to_string(),
            };
            match key {
                "max_file_bytes" => config.max_file_bytes = value.parse().map_err(|_| bad())?,
                "max_findings" => {
                    let n: usize = value.parse().map_err(|_| bad())?;
                    if n == 0 {
                        return Err(bad());
                    }
                    config.max_findings = n;
                }
                "required_areas" => {
                    // an empty list is allowed: nothing is then required
                    let list: Vec<String> = value
                        .split(',')
                        .map(|a| a.trim().to_ascii_lowercase())
                        .filter(|a| !a.is_empty())
                        .collect();
                    if list
                        .iter()
                        .any(|a| !a.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
                    {
                        return Err(bad());
                    }
                    config.required_areas = list;
                }
                "fail_on_warning" => config.fail_on_warning = flag(value).ok_or_else(bad)?,
                _ => {
                    return Err(VerifyError::ConfigUnknownKey {
                        line,
                        key: key.to_string(),
                    })
                }
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_conservative() {
        let c = Config::parse("").unwrap();
        assert_eq!(c.required_areas.len(), 4);
        assert!(!c.fail_on_warning);
    }

    #[test]
    fn lines_change_values_and_bad_lines_are_refused() {
        let c =
            Config::parse("# c\nrequired_areas = Board, rules\nfail_on_warning = on\n").unwrap();
        assert_eq!(c.required_areas, vec!["board", "rules"]);
        assert!(c.fail_on_warning);
        assert!(Config::parse("required_areas =\n")
            .unwrap()
            .required_areas
            .is_empty());
        assert!(matches!(
            Config::parse("x\n"),
            Err(VerifyError::ConfigSyntax { line: 1 })
        ));
        assert!(matches!(
            Config::parse("a = 1\n"),
            Err(VerifyError::ConfigUnknownKey { line: 1, .. })
        ));
        for bad in [
            "required_areas = a b\n",
            "max_findings = 0\n",
            "fail_on_warning = maybe\n",
        ] {
            assert!(
                matches!(
                    Config::parse(bad),
                    Err(VerifyError::ConfigBadValue { line: 1, .. })
                ),
                "{bad:?}"
            );
        }
    }
}
