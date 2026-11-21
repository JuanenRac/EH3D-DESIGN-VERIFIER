// SPDX-License-Identifier: GPL-3.0-or-later
//! The results of the checks, the problems the verifier finds in them and the verdict.

use crate::config::Config;
use crate::error::VerifyError;
use crate::Code;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// How serious a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// A failure.
    Error,
    /// Something to look at.
    Warning,
    /// Information.
    Info,
}

/// Whether the check could decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// The check decided.
    Confirmed,
    /// The check could not decide (missing data, a rule it does not support).
    Inconclusive,
}

/// Whether a finding is about one object or about the whole design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// One object, named by its stable identity.
    #[default]
    Object,
    /// The whole design.
    Design,
}

/// One finding of a check.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    /// The stable code, `PHOENIX-<AREA>-<NNN>`.
    pub code: String,
    /// The severity.
    pub severity: Severity,
    /// Confirmed or inconclusive.
    pub state: State,
    /// The stable identity of the object concerned.
    #[serde(default)]
    pub object: Option<String>,
    /// One object or the whole design.
    #[serde(default)]
    pub scope: Scope,
    /// What is wrong, in English.
    #[serde(default)]
    pub message: String,
    /// The rule that was applied (its name and version).
    #[serde(default)]
    pub rule: String,
}

/// The results file: what the checks said about one design.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Results {
    /// The identity of the design.
    pub design: String,
    /// The version of the set of rules used.
    pub rule_version: String,
    /// The areas that were run (`schematic`, `board`, ...).
    #[serde(default)]
    pub areas_run: Vec<String>,
    /// The findings.
    #[serde(default)]
    pub findings: Vec<Finding>,
}

/// The verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// Everything required ran and nothing is confirmed wrong.
    Pass,
    /// A confirmed failure.
    Fail,
    /// Nothing is confirmed wrong, but something could not be decided or a required area was not run.
    Inconclusive,
}

/// A problem the verifier finds in the results themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The code.
    pub code: Code,
    /// What is wrong.
    pub message: String,
}

/// The report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The design.
    pub design: String,
    /// The version of the rules.
    pub rule_version: String,
    /// The verdict.
    pub verdict: Verdict,
    /// The findings in canonical order.
    pub findings: Vec<Finding>,
    /// The problems of the results.
    pub problems: Vec<Problem>,
    /// The areas that were run, in lower case and in order.
    pub areas_run: Vec<String>,
    /// The SHA-256 of the canonical content (design, rule version, areas and findings).
    pub hash: String,
}

/// What replaces a code that is not a code when a result is printed: a fixed text, so that nothing the results file wrote there can reach an output.
pub(crate) const MALFORMED_CODE: &str = "(malformed code)";

/// The code as it may be printed: itself when it is well formed, a fixed marker when it is not.
pub(crate) fn printable_code(code: &str) -> &str {
    if code_is_well_formed(code) {
        code
    } else {
        MALFORMED_CODE
    }
}

fn code_is_well_formed(code: &str) -> bool {
    let mut parts = code.split('-');
    let ok = parts.next() == Some("PHOENIX");
    let area = parts.next().unwrap_or("");
    let number = parts.next().unwrap_or("");
    ok && !area.is_empty()
        && area
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && number.len() == 3
        && number.chars().all(|c| c.is_ascii_digit())
        && parts.next().is_none()
}

impl Results {
    /// Read a results file from JSON, within the bounds of the configuration.
    pub fn from_json(text: &str, config: &Config) -> Result<Results, VerifyError> {
        if text.len() as u64 > config.max_file_bytes {
            return Err(VerifyError::TooLarge {
                what: "results bytes",
            });
        }
        // A JSON array would be read as a list of the fields in order, so only an object is a results.
        if !text.trim_start().starts_with('{') {
            return Err(VerifyError::BadJson("a JSON object was expected".into()));
        }
        let results: Results =
            serde_json::from_str(text).map_err(|e| VerifyError::BadJson(e.to_string()))?;
        if results.findings.len() > config.max_findings {
            return Err(VerifyError::TooLarge { what: "findings" });
        }
        Ok(results)
    }
}

impl Report {
    /// Build the report: check each finding, find the areas that did not run, decide the verdict and fix the canonical order and the hash.
    pub fn build(results: &Results, config: &Config) -> Report {
        let mut problems = Vec::new();
        for f in &results.findings {
            if !code_is_well_formed(&f.code) {
                problems.push(Problem {
                    code: Code::BadCode,
                    message: format!("`{}` is not of the form PHOENIX-<AREA>-<NNN>", f.code),
                });
            }
            if f.scope == Scope::Object && f.object.as_deref().is_none_or(|o| o.trim().is_empty()) {
                problems.push(Problem {
                    code: Code::MissingObject,
                    message: format!(
                        "the finding {} names no object and is not about the whole design",
                        f.code
                    ),
                });
            }
        }
        let run: BTreeSet<String> = results
            .areas_run
            .iter()
            .map(|a| a.to_ascii_lowercase())
            .collect();
        let mut missing_area = false;
        for area in &config.required_areas {
            if !run.contains(area) {
                missing_area = true;
                problems.push(Problem {
                    code: Code::AreaNotRun,
                    message: format!("the area `{area}` is required and was not run"),
                });
            }
        }
        let undecided = results
            .findings
            .iter()
            .filter(|f| f.state == State::Inconclusive && f.severity != Severity::Info)
            .count();
        if undecided > 0 {
            problems.push(Problem {
                code: Code::Undecided,
                message: format!("{undecided} findings could not be decided"),
            });
        }
        let confirmed_failure = results.findings.iter().any(|f| {
            f.state == State::Confirmed
                && (f.severity == Severity::Error
                    || (config.fail_on_warning && f.severity == Severity::Warning))
        });
        let verdict = if confirmed_failure {
            Verdict::Fail
        } else if undecided > 0 || missing_area {
            Verdict::Inconclusive
        } else {
            Verdict::Pass
        };
        let mut findings = results.findings.clone();
        findings.sort();
        problems.sort_by(|a, b| (a.code, &a.message).cmp(&(b.code, &b.message)));
        let mut areas: Vec<&String> = run.iter().collect();
        areas.sort();
        let canonical = serde_json::json!({
            "design": results.design,
            "rule_version": results.rule_version,
            "areas_run": areas,
            "findings": findings,
        });
        let digest = Sha256::digest(canonical.to_string().as_bytes());
        Report {
            design: results.design.clone(),
            rule_version: results.rule_version.clone(),
            verdict,
            findings,
            problems,
            areas_run: run.iter().cloned().collect(),
            hash: digest.iter().map(|b| format!("{b:02x}")).collect(),
        }
    }

    /// The counts by code, severity and state, which carry no name of an object and no message: what may leave the machine for a private corpus.
    pub fn redacted_counts(&self) -> BTreeMap<String, usize> {
        let mut out = BTreeMap::new();
        for f in &self.findings {
            let key = format!("{} {:?} {:?}", printable_code(&f.code), f.severity, f.state)
                .to_lowercase();
            *out.entry(key).or_insert(0) += 1;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(code: &str, severity: Severity, state: State, object: Option<&str>) -> Finding {
        Finding {
            code: code.into(),
            severity,
            state,
            object: object.map(str::to_string),
            scope: Scope::Object,
            message: "secret-net-name is shorted".into(),
            rule: "drc 1".into(),
        }
    }

    fn results(findings: Vec<Finding>, areas: &[&str]) -> Results {
        Results {
            design: "d-1".into(),
            rule_version: "rules-1".into(),
            areas_run: areas.iter().map(|a| a.to_string()).collect(),
            findings,
        }
    }

    const ALL: [&str; 4] = ["schematic", "board", "rules", "manufacturing"];

    #[test]
    fn nothing_found_and_everything_run_is_a_pass() {
        let r = Report::build(&results(vec![], &ALL), &Config::default());
        assert_eq!(r.verdict, Verdict::Pass);
        assert!(r.problems.is_empty());
        assert_eq!(r.hash.len(), 64);
    }

    #[test]
    fn a_confirmed_error_fails_and_a_confirmed_warning_only_when_the_setting_says_so() {
        let w = finding(
            "PHOENIX-DRC-002",
            Severity::Warning,
            State::Confirmed,
            Some("n1"),
        );
        let e = finding(
            "PHOENIX-DRC-001",
            Severity::Error,
            State::Confirmed,
            Some("n2"),
        );
        assert_eq!(
            Report::build(&results(vec![w.clone()], &ALL), &Config::default()).verdict,
            Verdict::Pass
        );
        let strict = Config {
            fail_on_warning: true,
            ..Config::default()
        };
        assert_eq!(
            Report::build(&results(vec![w], &ALL), &strict).verdict,
            Verdict::Fail
        );
        assert_eq!(
            Report::build(&results(vec![e], &ALL), &Config::default()).verdict,
            Verdict::Fail
        );
    }

    #[test]
    fn an_undecided_finding_is_never_a_pass() {
        let u = finding(
            "PHOENIX-DRC-009",
            Severity::Error,
            State::Inconclusive,
            Some("n1"),
        );
        let r = Report::build(&results(vec![u.clone()], &ALL), &Config::default());
        assert_eq!(r.verdict, Verdict::Inconclusive);
        assert!(r.problems.iter().any(|p| p.code == Code::Undecided));
        // an undecided information finding does not block, but a confirmed failure still wins over an undecided one
        let i = finding(
            "PHOENIX-DRC-010",
            Severity::Info,
            State::Inconclusive,
            Some("n1"),
        );
        assert_eq!(
            Report::build(&results(vec![i], &ALL), &Config::default()).verdict,
            Verdict::Pass
        );
        let e = finding(
            "PHOENIX-DRC-001",
            Severity::Error,
            State::Confirmed,
            Some("n2"),
        );
        assert_eq!(
            Report::build(&results(vec![u, e], &ALL), &Config::default()).verdict,
            Verdict::Fail
        );
    }

    #[test]
    fn a_required_area_that_did_not_run_makes_the_verdict_inconclusive() {
        let r = Report::build(
            &results(vec![], &["schematic", "board"]),
            &Config::default(),
        );
        assert_eq!(r.verdict, Verdict::Inconclusive);
        assert_eq!(
            r.problems
                .iter()
                .filter(|p| p.code == Code::AreaNotRun)
                .count(),
            2
        );
        let none = Config {
            required_areas: vec![],
            ..Config::default()
        };
        assert_eq!(
            Report::build(&results(vec![], &[]), &none).verdict,
            Verdict::Pass
        );
    }

    #[test]
    fn badly_formed_codes_and_findings_without_an_object_are_reported() {
        let bad = finding("DRC-1", Severity::Error, State::Confirmed, None);
        let mut design = finding("PHOENIX-ERC-001", Severity::Warning, State::Confirmed, None);
        design.scope = Scope::Design;
        let r = Report::build(&results(vec![bad, design], &ALL), &Config::default());
        let codes: Vec<Code> = r.problems.iter().map(|p| p.code).collect();
        assert_eq!(codes, vec![Code::BadCode, Code::MissingObject]);
        for good in ["PHOENIX-DRC-001", "PHOENIX-A1-999"] {
            assert!(code_is_well_formed(good), "{good}");
        }
        for bad in [
            "PHOENIX-DRC-1",
            "phoenix-DRC-001",
            "PHOENIX--001",
            "PHOENIX-DRC-001-2",
            "EH3D-DRC-001",
            "",
        ] {
            assert!(!code_is_well_formed(bad), "{bad:?}");
        }
    }

    #[test]
    fn the_hash_of_a_fixed_report_is_the_known_one() {
        // a fixed vector: a change of the order of the fields or of the way the content is written changes every hash that was ever quoted
        let a = finding(
            "PHOENIX-DRC-001",
            Severity::Error,
            State::Confirmed,
            Some("n1"),
        );
        let b = finding(
            "PHOENIX-DRC-002",
            Severity::Warning,
            State::Confirmed,
            Some("n2"),
        );
        let r = Report::build(&results(vec![a, b], &ALL), &Config::default());
        assert_eq!(
            r.hash,
            "00e2568ada2406ea618e22debc8af201041ce7077b5eabf02d238f00e548c1ea"
        );
    }

    #[test]
    fn the_same_results_in_another_order_give_the_same_report() {
        let a = finding(
            "PHOENIX-DRC-001",
            Severity::Error,
            State::Confirmed,
            Some("n1"),
        );
        let b = finding(
            "PHOENIX-DRC-002",
            Severity::Warning,
            State::Confirmed,
            Some("n2"),
        );
        let r1 = Report::build(
            &results(vec![a.clone(), b.clone()], &ALL),
            &Config::default(),
        );
        let mut shuffled = results(vec![b, a], &ALL);
        shuffled.areas_run.reverse();
        let r2 = Report::build(&shuffled, &Config::default());
        assert_eq!(r1, r2);
        let mut other = shuffled;
        other.rule_version = "rules-2".into();
        assert_ne!(r1.hash, Report::build(&other, &Config::default()).hash);
    }

    #[test]
    fn the_redacted_counts_carry_no_message_and_no_object() {
        let a = finding(
            "PHOENIX-DRC-001",
            Severity::Error,
            State::Confirmed,
            Some("secret-id"),
        );
        let r = Report::build(&results(vec![a.clone(), a], &ALL), &Config::default());
        let counts = r.redacted_counts();
        assert_eq!(counts.len(), 1);
        let text = format!("{counts:?}");
        assert!(!text.contains("secret"), "{text}");
        assert_eq!(counts.values().sum::<usize>(), 2);
    }

    #[test]
    fn bad_json_unknown_fields_and_the_size_limits_are_refused() {
        let config = Config::default();
        for text in [
            "",
            "[]",
            r#"{"design":"d"}"#,
            r#"{"design":"d","rule_version":"r","extra":1}"#,
        ] {
            assert!(
                matches!(
                    Results::from_json(text, &config),
                    Err(VerifyError::BadJson(_))
                ),
                "{text:?}"
            );
        }
        let ok = r#"{"design":"d","rule_version":"r"}"#;
        assert!(Results::from_json(ok, &config).is_ok());
        let small = Config {
            max_file_bytes: 4,
            ..Config::default()
        };
        assert!(matches!(
            Results::from_json(ok, &small),
            Err(VerifyError::TooLarge { .. })
        ));
        let few = Config {
            max_findings: 1,
            ..Config::default()
        };
        let two = serde_json::to_string(&results(
            vec![
                finding("PHOENIX-A-001", Severity::Info, State::Confirmed, Some("o")),
                finding("PHOENIX-A-002", Severity::Info, State::Confirmed, Some("o")),
            ],
            &[],
        ))
        .unwrap();
        assert!(matches!(
            Results::from_json(&two, &few),
            Err(VerifyError::TooLarge { .. })
        ));
    }

    #[test]
    fn only_a_code_of_the_form_phoenix_area_and_three_digits_is_printed_as_it_is() {
        for good in ["PHOENIX-DRC-001", "PHOENIX-A1-123", "PHOENIX-THLA-999"] {
            assert_eq!(printable_code(good), good);
        }
        for bad in [
            "",
            "PHOENIX",
            "PHOENIX-DRC-1",
            "PHOENIX-DRC-1234",
            "PHOENIX-drc-001",
            "PHOENIX--001",
            "PHOENIX-DRC-00A",
            "PHOENIX-DRC-001-X",
            "phoenix-DRC-001",
            "OTHER-DRC-001",
            "PHOENIX-DRC-001\n",
        ] {
            assert_eq!(printable_code(bad), MALFORMED_CODE, "{bad:?}");
        }
    }

    #[test]
    fn a_confirmed_failure_wins_over_an_undecided_finding_and_an_undecided_error_is_not_a_failure()
    {
        let failure = finding(
            "PHOENIX-DRC-001",
            Severity::Error,
            State::Confirmed,
            Some("a"),
        );
        let undecided = finding(
            "PHOENIX-DRC-002",
            Severity::Error,
            State::Inconclusive,
            Some("b"),
        );
        let both = results(vec![failure, undecided.clone()], &ALL);
        assert_eq!(
            Report::build(&both, &Config::default()).verdict,
            Verdict::Fail
        );
        let only = results(vec![undecided], &ALL);
        let report = Report::build(&only, &Config::default());
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert!(report.problems.iter().any(|p| p.code == Code::Undecided));
    }

    #[test]
    fn information_never_fails_and_an_undecided_information_does_not_count() {
        let confirmed = finding(
            "PHOENIX-DRC-003",
            Severity::Info,
            State::Confirmed,
            Some("a"),
        );
        let unsure = finding(
            "PHOENIX-DRC-004",
            Severity::Info,
            State::Inconclusive,
            Some("b"),
        );
        let r = Report::build(&results(vec![confirmed, unsure], &ALL), &Config::default());
        assert_eq!(r.verdict, Verdict::Pass);
        assert!(r.problems.is_empty());
    }

    #[test]
    fn with_no_area_required_a_design_that_ran_nothing_is_a_pass() {
        let config = Config {
            required_areas: vec![],
            ..Config::default()
        };
        let r = Report::build(&results(vec![], &[]), &config);
        assert_eq!(r.verdict, Verdict::Pass);
        assert!(r.areas_run.is_empty());
    }

    #[test]
    fn the_names_of_the_areas_are_compared_without_the_case_and_come_back_in_lower_case_in_order() {
        let r = Report::build(
            &results(
                vec![],
                &["SCHEMATIC", "Board", "rules", "MANUFACTURING", "board"],
            ),
            &Config::default(),
        );
        assert_eq!(r.verdict, Verdict::Pass);
        assert_eq!(
            r.areas_run,
            ["board", "manufacturing", "rules", "schematic"]
        );
    }

    #[test]
    fn a_change_of_the_design_the_rules_or_one_finding_changes_the_hash() {
        let f = finding(
            "PHOENIX-DRC-001",
            Severity::Warning,
            State::Confirmed,
            Some("a"),
        );
        let base = results(vec![f.clone()], &ALL);
        let hash = |r: &Results| Report::build(r, &Config::default()).hash;
        let mut other_design = base.clone();
        other_design.design = "d-2".into();
        let mut other_rules = base.clone();
        other_rules.rule_version = "rules-2".into();
        let mut other_message = base.clone();
        other_message.findings[0].message = "another sentence".into();
        let all = [
            hash(&base),
            hash(&other_design),
            hash(&other_rules),
            hash(&other_message),
        ];
        let distinct: BTreeSet<&String> = all.iter().collect();
        assert_eq!(distinct.len(), 4);
    }

    #[test]
    fn a_finding_about_the_whole_design_needs_no_object_and_one_about_an_object_needs_a_name() {
        let mut whole = finding("PHOENIX-DRC-001", Severity::Info, State::Confirmed, None);
        whole.scope = Scope::Design;
        let blank = finding(
            "PHOENIX-DRC-002",
            Severity::Info,
            State::Confirmed,
            Some("  "),
        );
        let r = Report::build(&results(vec![whole, blank], &ALL), &Config::default());
        assert_eq!(r.problems.len(), 1);
        assert_eq!(r.problems[0].code, Code::MissingObject);
    }

    #[test]
    fn ten_thousand_findings_are_judged_and_counted_without_a_panic() {
        let findings: Vec<Finding> = (0..10_000)
            .map(|n| {
                finding(
                    "PHOENIX-DRC-001",
                    Severity::Warning,
                    State::Confirmed,
                    Some(format!("o{n}").as_str()),
                )
            })
            .collect();
        let r = Report::build(&results(findings, &ALL), &Config::default());
        assert_eq!(r.verdict, Verdict::Pass);
        assert_eq!(r.findings.len(), 10_000);
        assert_eq!(r.redacted_counts().values().sum::<usize>(), 10_000);
    }

    #[test]
    fn a_code_that_is_not_a_code_is_counted_under_the_fixed_marker_and_never_under_its_own_text() {
        let evil = finding(
            "secret-net-name",
            Severity::Error,
            State::Confirmed,
            Some("a"),
        );
        let fine = finding(
            "PHOENIX-DRC-001",
            Severity::Error,
            State::Confirmed,
            Some("b"),
        );
        let r = Report::build(&results(vec![evil, fine], &ALL), &Config::default());
        let counts = r.redacted_counts();
        assert_eq!(counts.get("(malformed code) error confirmed"), Some(&1));
        assert_eq!(counts.get("phoenix-drc-001 error confirmed"), Some(&1));
        assert!(counts.keys().all(|k| !k.contains("secret")));
    }

    #[test]
    fn a_text_that_is_not_a_results_file_is_an_error_and_never_a_panic() {
        let config = Config::default();
        for text in [
            "",
            " ",
            "null",
            "[]",
            "12",
            "\"x\"",
            "{",
            "{}",
            "{\"design\":\"d\"}",
        ] {
            assert!(
                matches!(
                    Results::from_json(text, &config),
                    Err(VerifyError::BadJson(_))
                ),
                "{text:?}"
            );
        }
    }

    #[test]
    fn exactly_the_most_findings_is_read_and_one_more_is_not() {
        let f = finding(
            "PHOENIX-DRC-001",
            Severity::Info,
            State::Confirmed,
            Some("a"),
        );
        let config = Config {
            max_findings: 2,
            ..Config::default()
        };
        let two = serde_json::to_string(&results(vec![f.clone(), f.clone()], &ALL)).unwrap();
        assert!(Results::from_json(&two, &config).is_ok());
        let three = serde_json::to_string(&results(vec![f.clone(), f.clone(), f], &ALL)).unwrap();
        assert!(matches!(
            Results::from_json(&three, &config),
            Err(VerifyError::TooLarge { .. })
        ));
    }

    #[test]
    fn a_results_file_written_and_read_again_gives_the_same_report() {
        let f = finding(
            "PHOENIX-DRC-001",
            Severity::Warning,
            State::Inconclusive,
            Some("a"),
        );
        let original = results(vec![f], &ALL);
        let text = serde_json::to_string(&original).unwrap();
        let again = Results::from_json(&text, &Config::default()).unwrap();
        assert_eq!(again, original);
        assert_eq!(
            Report::build(&again, &Config::default()),
            Report::build(&original, &Config::default())
        );
    }

    #[test]
    fn the_findings_of_the_report_come_sorted_whatever_the_order_they_were_given() {
        let a = finding(
            "PHOENIX-DRC-001",
            Severity::Error,
            State::Confirmed,
            Some("x"),
        );
        let b = finding(
            "PHOENIX-DRC-002",
            Severity::Warning,
            State::Confirmed,
            Some("y"),
        );
        let c = finding(
            "PHOENIX-ERC-001",
            Severity::Info,
            State::Confirmed,
            Some("z"),
        );
        let forward = Report::build(
            &results(vec![a.clone(), b.clone(), c.clone()], &ALL),
            &Config::default(),
        );
        let backward = Report::build(&results(vec![c, b, a], &ALL), &Config::default());
        let mut sorted = forward.findings.clone();
        sorted.sort();
        assert_eq!(forward.findings, sorted);
        assert_eq!(forward, backward);
    }

    #[test]
    fn the_severity_and_the_state_are_written_in_lower_case_in_the_results_file() {
        let f = finding(
            "PHOENIX-DRC-001",
            Severity::Error,
            State::Inconclusive,
            Some("a"),
        );
        let text = serde_json::to_string(&f).unwrap();
        assert!(text.contains("\"severity\":\"error\""), "{text}");
        assert!(text.contains("\"state\":\"inconclusive\""), "{text}");
        assert!(text.contains("\"scope\":\"object\""), "{text}");
    }
}
