// SPDX-License-Identifier: GPL-3.0-or-later
//! The review document: the report written as text a person can read, sign and file.
//!
//! It lists what a reviewer needs to know what the verdict rests on: the rules that were applied, the data that was used (the areas that were run and the hash of
//! the canonical report) and the version of the tool. It adds nothing that is not in the report, reads no clock and prints the same text for the same report.
//! The redacted form carries only codes, severities, states and counts, like `report --redacted`.

use crate::config::Config;
use crate::report::{Finding, Report, Scope, Severity, State, Verdict};
use std::collections::BTreeMap;

/// Text for a list: the items separated by commas, or `none`.
fn list(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(", ")
    }
}

/// A text from the results put on one line: control characters become spaces and the ends are trimmed.
fn one_line(text: &str) -> String {
    let spaced: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    spaced.trim().to_string()
}

/// A text for a table cell: one line, and the bar that separates cells is escaped.
fn cell(text: &str) -> String {
    one_line(text).replace('|', "\\|")
}

fn verdict_word(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Pass => "pass",
        Verdict::Fail => "fail",
        Verdict::Inconclusive => "inconclusive",
    }
}

fn severity_word(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "information",
    }
}

fn state_word(state: State) -> &'static str {
    match state {
        State::Confirmed => "confirmed",
        State::Inconclusive => "inconclusive",
    }
}

fn finding_line(f: &Finding) -> String {
    let object = if f.scope == Scope::Design {
        "(whole design)".to_string()
    } else {
        match f.object.as_deref().map(one_line) {
            Some(o) if !o.is_empty() => o,
            _ => "(no object)".to_string(),
        }
    };
    let message = one_line(&f.message);
    let mut line = format!("- `{}` {} {}", f.code, state_word(f.state), object);
    if !message.is_empty() {
        line.push_str(": ");
        line.push_str(&message);
    }
    line
}

/// Write the review document of a report. With `redacted` set the design name, the objects, the messages and the names of the rules are left out.
pub fn render(report: &Report, config: &Config, redacted: bool) -> String {
    let mut out: Vec<String> = Vec::new();
    out.push("# Design review".to_string());
    out.push(String::new());
    if !redacted {
        out.push(format!("- Design: {}", one_line(&report.design)));
    }
    out.push(format!("- Verdict: {}", verdict_word(report.verdict)));
    out.push(format!(
        "- Rule version: {}",
        one_line(&report.rule_version)
    ));
    out.push(format!("- Report hash (SHA-256): {}", report.hash));
    out.push(format!(
        "- Tool: eh3d-design-verifier {}",
        env!("CARGO_PKG_VERSION")
    ));

    let count = |s: Severity| report.findings.iter().filter(|f| f.severity == s).count();
    let inconclusive = report
        .findings
        .iter()
        .filter(|f| f.state == State::Inconclusive)
        .count();
    let not_run: Vec<String> = config
        .required_areas
        .iter()
        .filter(|a| !report.areas_run.contains(a))
        .cloned()
        .collect();
    out.push(String::new());
    out.push("## Data used".to_string());
    out.push(String::new());
    out.push(format!("- Areas run: {}", list(&report.areas_run)));
    out.push(format!(
        "- Areas required: {}",
        list(&config.required_areas)
    ));
    out.push(format!("- Required areas not run: {}", list(&not_run)));
    out.push(format!(
        "- Findings: {} (errors {}, warnings {}, information {}; inconclusive {})",
        report.findings.len(),
        count(Severity::Error),
        count(Severity::Warning),
        count(Severity::Info),
        inconclusive
    ));
    out.push(format!(
        "- Setting fail_on_warning: {}",
        config.fail_on_warning
    ));
    out.push(String::new());

    if redacted {
        out.push("## Counts".to_string());
        out.push(String::new());
        let mut rows: BTreeMap<(&str, Severity, State), usize> = BTreeMap::new();
        for f in &report.findings {
            *rows
                .entry((f.code.as_str(), f.severity, f.state))
                .or_insert(0) += 1;
        }
        if rows.is_empty() {
            out.push("No findings were reported.".to_string());
        } else {
            out.push("| Code | Severity | State | Findings |".to_string());
            out.push("|---|---|---|---|".to_string());
            for ((code, severity, state), n) in rows {
                out.push(format!(
                    "| {} | {} | {} | {} |",
                    cell(code),
                    severity_word(severity),
                    state_word(state),
                    n
                ));
            }
        }
    } else {
        out.push("## Rules applied".to_string());
        out.push(String::new());
        let mut rules: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        for f in &report.findings {
            *rules.entry((f.rule.as_str(), f.code.as_str())).or_insert(0) += 1;
        }
        if rules.is_empty() {
            out.push("No findings were reported.".to_string());
        } else {
            out.push("| Rule | Code | Findings |".to_string());
            out.push("|---|---|---|".to_string());
            for ((rule, code), n) in rules {
                let name = if one_line(rule).is_empty() {
                    "(not stated)".to_string()
                } else {
                    cell(rule)
                };
                out.push(format!("| {} | {} | {} |", name, cell(code), n));
            }
        }
        out.push(String::new());
        out.push("## Findings".to_string());
        out.push(String::new());
        if report.findings.is_empty() {
            out.push("No findings were reported.".to_string());
        } else {
            let mut budget = config.review_max_findings;
            let mut listed = 0usize;
            for (severity, title) in [
                (Severity::Error, "Errors"),
                (Severity::Warning, "Warnings"),
                (Severity::Info, "Information"),
            ] {
                let take: Vec<&Finding> = report
                    .findings
                    .iter()
                    .filter(|f| f.severity == severity)
                    .take(budget)
                    .collect();
                if take.is_empty() {
                    continue;
                }
                budget -= take.len();
                listed += take.len();
                out.push(format!("### {title}"));
                out.push(String::new());
                for f in take {
                    out.push(finding_line(f));
                }
                out.push(String::new());
            }
            let left = report.findings.len() - listed;
            if left == 1 {
                out.push(format!(
                    "1 more finding is not listed (review_max_findings = {}).",
                    config.review_max_findings
                ));
            } else if left > 1 {
                out.push(format!(
                    "{left} more findings are not listed (review_max_findings = {}).",
                    config.review_max_findings
                ));
            } else {
                // the blank line after the last list is not needed
                out.pop();
            }
        }
    }

    out.push(String::new());
    out.push("## Problems found in the results".to_string());
    out.push(String::new());
    if report.problems.is_empty() {
        out.push("None.".to_string());
    } else {
        for p in &report.problems {
            if redacted {
                out.push(format!("- {}", p.code.as_str()));
            } else {
                out.push(format!("- {}: {}", p.code.as_str(), one_line(&p.message)));
            }
        }
    }
    let mut text = out.join("\n");
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Problem;
    use crate::Code;

    fn finding(
        code: &str,
        severity: Severity,
        state: State,
        object: Option<&str>,
        scope: Scope,
        message: &str,
        rule: &str,
    ) -> Finding {
        Finding {
            code: code.to_string(),
            severity,
            state,
            object: object.map(str::to_string),
            scope,
            message: message.to_string(),
            rule: rule.to_string(),
        }
    }

    fn report() -> Report {
        Report {
            design: "board-7".to_string(),
            rule_version: "2.1".to_string(),
            verdict: Verdict::Fail,
            findings: vec![
                finding(
                    "PHOENIX-DRC-001",
                    Severity::Error,
                    State::Confirmed,
                    Some("R1"),
                    Scope::Object,
                    "too close",
                    "clearance v2",
                ),
                finding(
                    "PHOENIX-DRC-001",
                    Severity::Error,
                    State::Confirmed,
                    Some("R2"),
                    Scope::Object,
                    "too close",
                    "clearance v2",
                ),
                finding(
                    "PHOENIX-MFG-004",
                    Severity::Warning,
                    State::Inconclusive,
                    Some("U1"),
                    Scope::Object,
                    "profile\nunknown",
                    "drill min v1",
                ),
                finding(
                    "PHOENIX-NET-009",
                    Severity::Info,
                    State::Confirmed,
                    None,
                    Scope::Design,
                    "ok",
                    "",
                ),
            ],
            problems: vec![
                Problem {
                    code: Code::Undecided,
                    message: "1 findings could not be decided".to_string(),
                },
                Problem {
                    code: Code::AreaNotRun,
                    message: "the area `manufacturing` is required and was not run".to_string(),
                },
                Problem {
                    code: Code::AreaNotRun,
                    message: "the area `schematic` is required and was not run".to_string(),
                },
            ],
            areas_run: vec!["board".to_string(), "rules".to_string()],
            hash: "abc123".to_string(),
        }
    }

    #[test]
    fn the_review_lists_the_rules_the_data_and_the_findings() {
        let expected = format!(
            "\
# Design review

- Design: board-7
- Verdict: fail
- Rule version: 2.1
- Report hash (SHA-256): abc123
- Tool: eh3d-design-verifier {}

## Data used

- Areas run: board, rules
- Areas required: schematic, board, rules, manufacturing
- Required areas not run: schematic, manufacturing
- Findings: 4 (errors 2, warnings 1, information 1; inconclusive 1)
- Setting fail_on_warning: false

## Rules applied

| Rule | Code | Findings |
|---|---|---|
| (not stated) | PHOENIX-NET-009 | 1 |
| clearance v2 | PHOENIX-DRC-001 | 2 |
| drill min v1 | PHOENIX-MFG-004 | 1 |

## Findings

### Errors

- `PHOENIX-DRC-001` confirmed R1: too close
- `PHOENIX-DRC-001` confirmed R2: too close

### Warnings

- `PHOENIX-MFG-004` inconclusive U1: profile unknown

### Information

- `PHOENIX-NET-009` confirmed (whole design): ok

## Problems found in the results

- PHOENIX-DVER-004: 1 findings could not be decided
- PHOENIX-DVER-005: the area `manufacturing` is required and was not run
- PHOENIX-DVER-005: the area `schematic` is required and was not run
",
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(render(&report(), &Config::default(), false), expected);
    }

    #[test]
    fn the_same_report_gives_the_same_text() {
        let config = Config::default();
        assert_eq!(
            render(&report(), &config, false),
            render(&report(), &config, false)
        );
    }

    #[test]
    fn the_limit_lists_the_most_serious_first_and_counts_the_rest() {
        let config = Config {
            review_max_findings: 2,
            ..Config::default()
        };
        let text = render(&report(), &config, false);
        assert!(text.contains("### Errors"));
        assert!(text.contains("R1: too close") && text.contains("R2: too close"));
        assert!(!text.contains("### Warnings") && !text.contains("### Information"));
        assert!(text.contains("2 more findings are not listed (review_max_findings = 2)."));
        let config = Config {
            review_max_findings: 3,
            ..Config::default()
        };
        let text = render(&report(), &config, false);
        assert!(text.contains("### Warnings") && !text.contains("### Information"));
        assert!(text.contains("1 more finding is not listed (review_max_findings = 3)."));
    }

    #[test]
    fn the_redacted_review_carries_only_codes_and_counts() {
        let text = render(&report(), &Config::default(), true);
        for secret in [
            "board-7",
            "R1",
            "R2",
            "U1",
            "too close",
            "profile",
            "clearance",
        ] {
            assert!(!text.contains(secret), "{secret}");
        }
        assert!(!text.contains("## Findings") && !text.contains("## Rules applied"));
        assert!(text.contains("| PHOENIX-DRC-001 | error | confirmed | 2 |"));
        assert!(text.contains("| PHOENIX-MFG-004 | warning | inconclusive | 1 |"));
        assert!(text.contains("| PHOENIX-NET-009 | information | confirmed | 1 |"));
        assert!(text.contains("- PHOENIX-DVER-004\n- PHOENIX-DVER-005\n- PHOENIX-DVER-005\n"));
        assert!(text.contains("- Verdict: fail") && text.contains("abc123"));
    }

    #[test]
    fn a_clean_report_says_so() {
        let mut clean = report();
        clean.verdict = Verdict::Pass;
        clean.findings.clear();
        clean.problems.clear();
        clean.areas_run = ["schematic", "board", "rules", "manufacturing"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let text = render(&clean, &Config::default(), false);
        assert!(text.contains("- Verdict: pass"));
        assert!(text.contains("- Required areas not run: none"));
        assert!(
            text.contains("- Findings: 0 (errors 0, warnings 0, information 0; inconclusive 0)")
        );
        assert_eq!(text.matches("No findings were reported.").count(), 2);
        assert!(text.ends_with("## Problems found in the results\n\nNone.\n"));
        let text = render(&clean, &Config::default(), true);
        assert!(text.contains("## Counts\n\nNo findings were reported.\n"));
    }

    #[test]
    fn text_from_the_results_cannot_break_the_layout() {
        let mut odd = report();
        odd.findings = vec![finding(
            "PHOENIX-DRC-001",
            Severity::Error,
            State::Confirmed,
            Some("a\r\nb"),
            Scope::Object,
            "line one\n## Fake heading",
            "a|b",
        )];
        let text = render(&odd, &Config::default(), false);
        assert!(text.contains("| a\\|b | PHOENIX-DRC-001 | 1 |"));
        assert!(text.contains("- `PHOENIX-DRC-001` confirmed a  b: line one ## Fake heading\n"));
        assert!(!text.contains("\n## Fake heading"));
    }
}
