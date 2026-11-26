// SPDX-License-Identifier: GPL-3.0-or-later
//! The simulation mode: a host twin of the checks of the suite, with the faults a check can have and the recovery from them.
//!
//! The checks themselves (schematic, board, rules, manufacturing) do not exist yet. [`CheckerTwin`] stands in for them: given a seed it writes the same
//! results every time (it never reads the clock), so the verdict, the canonical hash and the review document can be exercised against every kind of faulty
//! check ([`Fault`]) and against the recovery ([`recover`]). Nothing here has run against a real check.

use crate::config::Config;
use crate::report::{code_is_well_formed, Finding, Results, Scope, Severity, State};

/// A small deterministic generator (SplitMix64): the same seed always gives the same numbers.
#[derive(Debug, Clone)]
pub struct SimRng(u64);

impl SimRng {
    /// A generator that starts from `seed`.
    pub fn new(seed: u64) -> SimRng {
        SimRng(seed)
    }

    /// The next number.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number below `bound` (`bound` must not be 0).
    pub fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}

/// What can go wrong with a check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// The check of this area crashed: the area is not among the areas that ran.
    CrashedArea(String),
    /// A check could not decide.
    Undecided,
    /// A check wrote a code that is not `PHOENIX-<AREA>-<NNN>`.
    MalformedCode,
    /// A check wrote a finding about an object without naming the object.
    NoObject,
    /// A check confirmed an error (a real failure of the design, not a fault of the check).
    ConfirmedError,
}

/// The twin of the checks of the suite.
#[derive(Debug, Clone)]
pub struct CheckerTwin {
    rng: SimRng,
}

impl CheckerTwin {
    /// A twin that writes the same results for the same seed.
    pub fn new(seed: u64) -> CheckerTwin {
        CheckerTwin {
            rng: SimRng::new(seed),
        }
    }

    /// What the check of one area finds: a few confirmed notes and warnings, never an error.
    pub fn area_findings(&mut self, area: &str) -> Vec<Finding> {
        let count = 1 + self.rng.below(3);
        (0..count)
            .map(|n| {
                let warning = self.rng.below(2) == 0;
                Finding {
                    code: format!("PHOENIX-{}-{:03}", area.to_ascii_uppercase(), n + 1),
                    severity: if warning {
                        Severity::Warning
                    } else {
                        Severity::Info
                    },
                    state: State::Confirmed,
                    object: Some(format!("obj-{:03}", self.rng.below(500))),
                    scope: Scope::Object,
                    message: "close to the limit".to_string(),
                    rule: format!("{area}-rules 1"),
                }
            })
            .collect()
    }

    /// The results of running the areas of the configuration.
    pub fn results(&mut self, design: &str, config: &Config) -> Results {
        let mut findings = Vec::new();
        for area in &config.required_areas {
            findings.extend(self.area_findings(area));
        }
        Results {
            design: design.to_string(),
            rule_version: "twin-rules-1".to_string(),
            areas_run: config.required_areas.clone(),
            findings,
        }
    }
}

/// Inject a fault into results. Returns `false` when there is nothing to change (nothing is changed then).
pub fn inject(results: &mut Results, fault: &Fault) -> bool {
    match fault {
        Fault::CrashedArea(area) => {
            let before = results.areas_run.len();
            results.areas_run.retain(|a| !a.eq_ignore_ascii_case(area));
            let prefix = format!("PHOENIX-{}-", area.to_ascii_uppercase());
            results.findings.retain(|f| !f.code.starts_with(&prefix));
            results.areas_run.len() != before
        }
        Fault::Undecided => match results.findings.first_mut() {
            Some(f) => {
                f.state = State::Inconclusive;
                f.severity = Severity::Warning;
                true
            }
            None => false,
        },
        Fault::MalformedCode => match results.findings.first_mut() {
            Some(f) => {
                f.code = "drc-1".to_string();
                true
            }
            None => false,
        },
        Fault::NoObject => match results.findings.last_mut() {
            Some(f) => {
                f.object = None;
                true
            }
            None => false,
        },
        Fault::ConfirmedError => match results.findings.first_mut() {
            Some(f) => {
                f.severity = Severity::Error;
                f.state = State::Confirmed;
                true
            }
            None => false,
        },
    }
}

/// The recovery: run again the areas that did not run, decide what could not be decided and write again what was malformed. A confirmed error is a
/// failure of the design and stays. Returns how many things were done.
pub fn recover(results: &mut Results, twin: &mut CheckerTwin, config: &Config) -> usize {
    let mut done = 0;
    for area in &config.required_areas {
        if !results
            .areas_run
            .iter()
            .any(|a| a.eq_ignore_ascii_case(area))
        {
            results.areas_run.push(area.clone());
            let more = twin.area_findings(area);
            results.findings.extend(more);
            done += 1;
        }
    }
    for f in &mut results.findings {
        if f.state == State::Inconclusive {
            f.state = State::Confirmed;
            f.severity = Severity::Info;
            done += 1;
        }
    }
    let before = results.findings.len();
    results.findings.retain(|f| {
        // the same rule as the report, so that nothing the report calls malformed is left after the recovery
        let well_formed = code_is_well_formed(&f.code);
        let named =
            f.scope == Scope::Design || f.object.as_deref().is_some_and(|o| !o.trim().is_empty());
        // a confirmed error is a failure of the design: it stays even when it is badly written
        (well_formed && named) || (f.severity == Severity::Error && f.state == State::Confirmed)
    });
    done + (before - results.findings.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{Report, Verdict};
    use crate::review;
    use crate::Code;

    fn results(seed: u64) -> Results {
        CheckerTwin::new(seed).results("design-1", &Config::default())
    }

    fn build(r: &Results) -> Report {
        Report::build(r, &Config::default())
    }

    #[test]
    fn the_same_seed_gives_the_same_results_and_the_same_hash() {
        let (a, b) = (results(5), results(5));
        assert_eq!(a, b);
        assert_eq!(build(&a).hash, build(&b).hash);
        assert_ne!(build(&a).hash, build(&results(6)).hash);
    }

    #[test]
    fn what_the_twin_writes_passes_with_no_problems() {
        for seed in 0..20 {
            let report = build(&results(seed));
            assert_eq!(report.verdict, Verdict::Pass, "seed {seed}");
            assert!(report.problems.is_empty(), "seed {seed}");
        }
    }

    #[test]
    fn the_order_of_the_findings_does_not_change_the_hash() {
        let a = results(8);
        let mut b = a.clone();
        b.findings.reverse();
        assert_eq!(build(&a).hash, build(&b).hash);
    }

    #[test]
    fn a_crashed_area_makes_the_verdict_inconclusive() {
        let mut r = results(1);
        assert!(inject(&mut r, &Fault::CrashedArea("board".into())));
        let report = build(&r);
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert!(report.problems.iter().any(|p| p.code == Code::AreaNotRun));
        assert!(!report.areas_run.contains(&"board".to_string()));
    }

    #[test]
    fn every_fault_of_a_check_is_reported_with_its_code() {
        let cases = [
            (Fault::Undecided, Code::Undecided, Verdict::Inconclusive),
            (Fault::MalformedCode, Code::BadCode, Verdict::Pass),
            (Fault::NoObject, Code::MissingObject, Verdict::Pass),
        ];
        for (fault, code, verdict) in cases {
            let mut r = results(2);
            assert!(inject(&mut r, &fault), "{fault:?}");
            let report = build(&r);
            assert!(report.problems.iter().any(|p| p.code == code), "{fault:?}");
            assert_eq!(report.verdict, verdict, "{fault:?}");
        }
    }

    #[test]
    fn a_confirmed_error_fails_and_recovery_does_not_hide_it() {
        let mut r = results(3);
        inject(&mut r, &Fault::ConfirmedError);
        assert_eq!(build(&r).verdict, Verdict::Fail);
        let mut twin = CheckerTwin::new(77);
        recover(&mut r, &mut twin, &Config::default());
        assert_eq!(build(&r).verdict, Verdict::Fail);
    }

    #[test]
    fn a_confirmed_error_with_a_malformed_code_is_not_hidden_by_recovery() {
        let mut r = results(3);
        inject(&mut r, &Fault::ConfirmedError);
        inject(&mut r, &Fault::MalformedCode);
        assert_eq!(build(&r).verdict, Verdict::Fail);
        let mut twin = CheckerTwin::new(77);
        recover(&mut r, &mut twin, &Config::default());
        assert_eq!(build(&r).verdict, Verdict::Fail);
        assert!(r
            .findings
            .iter()
            .any(|f| f.severity == Severity::Error && f.state == State::Confirmed));
    }

    #[test]
    fn recovery_runs_the_area_again_and_the_verdict_goes_back_to_pass() {
        let config = Config::default();
        let mut r = results(4);
        inject(&mut r, &Fault::CrashedArea("rules".into()));
        inject(&mut r, &Fault::Undecided);
        inject(&mut r, &Fault::MalformedCode);
        inject(&mut r, &Fault::NoObject);
        assert_ne!(build(&r).verdict, Verdict::Pass);
        let mut twin = CheckerTwin::new(88);
        assert!(recover(&mut r, &mut twin, &config) >= 3);
        let report = build(&r);
        assert_eq!(report.verdict, Verdict::Pass);
        assert!(report.problems.is_empty());
        assert_eq!(recover(&mut r, &mut twin, &config), 0);
    }

    #[test]
    fn nothing_to_change_is_reported_as_such() {
        let mut empty = Results {
            design: "d".into(),
            rule_version: "r".into(),
            areas_run: vec![],
            findings: vec![],
        };
        for fault in [
            Fault::CrashedArea("board".into()),
            Fault::Undecided,
            Fault::MalformedCode,
            Fault::NoObject,
            Fault::ConfirmedError,
        ] {
            assert!(!inject(&mut empty, &fault), "{fault:?}");
        }
    }

    #[test]
    fn the_redacted_review_of_a_faulty_run_leaks_no_object_or_design_name() {
        let config = Config::default();
        let mut r = results(9);
        inject(&mut r, &Fault::ConfirmedError);
        let report = build(&r);
        let full = review::render(&report, &config, false);
        let redacted = review::render(&report, &config, true);
        assert!(full.contains("obj-"));
        assert!(!redacted.contains("obj-"));
        assert!(!redacted.contains("design-1"));
        assert_eq!(redacted, review::render(&report, &config, true));
    }

    #[test]
    fn recovery_leaves_nothing_the_report_calls_malformed() {
        let config = Config::default();
        for code in [
            "PHOENIX-drc-001",
            "PHOENIX-DRC-1",
            "PHOENIX--001",
            "PHOENIX-DRC-abc",
        ] {
            let mut r = results(6);
            r.findings[0].code = code.to_string();
            r.findings[0].severity = Severity::Warning;
            assert!(build(&r).problems.iter().any(|p| p.code == Code::BadCode));
            let mut twin = CheckerTwin::new(1);
            assert_eq!(recover(&mut r, &mut twin, &config), 1, "{code}");
            assert!(build(&r).problems.is_empty(), "{code}");
        }
    }

    #[test]
    fn a_seeded_set_of_results_is_judged_by_the_plain_rule() {
        let mut rng = SimRng::new(404);
        for round in 0..300 {
            let mut r = results(rng.next_u64());
            for f in &mut r.findings {
                f.severity =
                    [Severity::Error, Severity::Warning, Severity::Info][rng.below(3) as usize];
                f.state = [State::Confirmed, State::Inconclusive][rng.below(2) as usize];
            }
            if rng.below(4) == 0 {
                r.areas_run.pop();
            }
            let strict = rng.below(2) == 0;
            let config = Config {
                fail_on_warning: strict,
                ..Config::default()
            };
            let failed = r.findings.iter().any(|f| {
                f.state == State::Confirmed
                    && (f.severity == Severity::Error
                        || (strict && f.severity == Severity::Warning))
            });
            let open = r.areas_run.len() < 4
                || r.findings
                    .iter()
                    .any(|f| f.state == State::Inconclusive && f.severity != Severity::Info);
            let expected = if failed {
                Verdict::Fail
            } else if open {
                Verdict::Inconclusive
            } else {
                Verdict::Pass
            };
            let report = Report::build(&r, &config);
            assert_eq!(report.verdict, expected, "round {round}");
            let mut turned = r.clone();
            turned.findings.reverse();
            assert_eq!(Report::build(&turned, &config), report, "round {round}");
        }
    }
}
