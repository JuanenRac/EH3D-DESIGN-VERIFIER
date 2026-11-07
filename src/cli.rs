// SPDX-License-Identifier: GPL-3.0-or-later
//! The command line. It is a thin layer: parsing the arguments and printing; the checks live in the other modules.
//!
//! Exit codes: 0 pass, 1 fail (a confirmed failure), 2 the input or the arguments are unusable, 3 inconclusive (nothing confirmed wrong, but something could not be
//! decided or a required area did not run).

use crate::config::Config;
use crate::error::VerifyError;
use crate::report::{Report, Results, Verdict};
use crate::review;
use std::fs;
use std::io::Write;
use std::path::Path;

const USAGE: &str = "\
eh3d-design-verifier [--config FILE] <command>

  report [--redacted] <results.json>       gather the results of the checks into one report and give the verdict
                                           (--redacted prints only counts by code, severity and state, no names)
  review [--redacted] <results.json>       write the review document: the verdict, the rules applied, the data used and the findings
                                           (--redacted leaves out the design, the objects, the messages and the rule names)
  hash <results.json>                      print the SHA-256 of the canonical report (the same results give the same hash)

Exit codes: 0 pass, 1 fail, 2 unusable input, 3 inconclusive.
";

fn read_text(path: &Path, limit: u64) -> Result<String, VerifyError> {
    let io = |source| VerifyError::Io {
        path: path.to_path_buf(),
        source,
    };
    if fs::metadata(path).map_err(io)?.len() > limit {
        return Err(VerifyError::TooLarge { what: "input file" });
    }
    fs::read_to_string(path).map_err(io)
}

/// Run the command line with the given arguments (without the program name). Returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match dispatch(args, out) {
        Ok(code) => code,
        Err(None) => {
            let _ = err.write_all(USAGE.as_bytes());
            2
        }
        Err(Some(error)) => {
            let _ = writeln!(err, "error: {error}");
            2
        }
    }
}

fn exit_code(verdict: Verdict) -> i32 {
    match verdict {
        Verdict::Pass => 0,
        Verdict::Fail => 1,
        Verdict::Inconclusive => 3,
    }
}

fn dispatch(args: &[String], out: &mut dyn Write) -> Result<i32, Option<VerifyError>> {
    let mut args = args.to_vec();
    let mut config = Config::default();
    if args.first().map(String::as_str) == Some("--config") {
        if args.len() < 3 {
            return Err(None);
        }
        config = Config::parse(&read_text(Path::new(&args[1]), 1024 * 1024).map_err(Some)?)
            .map_err(Some)?;
        args.drain(..2);
    }
    let load = |path: &String| -> Result<Report, Option<VerifyError>> {
        let text = read_text(Path::new(path), config.max_file_bytes).map_err(Some)?;
        let results = Results::from_json(&text, &config).map_err(Some)?;
        Ok(Report::build(&results, &config))
    };
    let rest: Vec<&str> = args.iter().map(String::as_str).collect();
    match rest.as_slice() {
        ["report", "--redacted", _] => {
            let report = load(&args[2])?;
            let _ = writeln!(out, "verdict: {:?}", report.verdict);
            for (key, count) in report.redacted_counts() {
                let _ = writeln!(out, "{key}: {count}");
            }
            for p in &report.problems {
                let _ = writeln!(out, "{}", p.code.as_str());
            }
            let _ = writeln!(out, "# hash {}", report.hash);
            Ok(exit_code(report.verdict))
        }
        ["report", _] => {
            let report = load(&args[1])?;
            let _ = writeln!(
                out,
                "design {} (rules {}): {:?}",
                report.design, report.rule_version, report.verdict
            );
            for f in &report.findings {
                let _ = writeln!(
                    out,
                    "{} {:?} {:?} {}: {}",
                    f.code,
                    f.severity,
                    f.state,
                    f.object.as_deref().unwrap_or("(design)"),
                    f.message
                );
            }
            for p in &report.problems {
                let _ = writeln!(out, "{}: {}", p.code.as_str(), p.message);
            }
            let _ = writeln!(out, "# hash {}", report.hash);
            Ok(exit_code(report.verdict))
        }
        ["review", "--redacted", _] => {
            let report = load(&args[2])?;
            let _ = write!(out, "{}", review::render(&report, &config, true));
            Ok(exit_code(report.verdict))
        }
        ["review", _] => {
            let report = load(&args[1])?;
            let _ = write!(out, "{}", review::render(&report, &config, false));
            Ok(exit_code(report.verdict))
        }
        ["hash", _] => {
            let report = load(&args[1])?;
            let _ = writeln!(out, "{}", report.hash);
            Ok(0)
        }
        _ => Err(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(list: &[&str]) -> (i32, String, String) {
        let args: Vec<String> = list.iter().map(|s| s.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    const ALL: &str = r#""areas_run":["schematic","board","rules","manufacturing"]"#;

    fn file(dir: &Path, name: &str, body: String) -> String {
        let path = dir.join(name);
        fs::write(&path, body).unwrap();
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn the_exit_code_follows_the_verdict() {
        let work = tempfile::tempdir().unwrap();
        let pass = file(
            work.path(),
            "pass.json",
            format!(r#"{{"design":"d","rule_version":"r",{ALL}}}"#),
        );
        let fail = file(
            work.path(),
            "fail.json",
            format!(
                r#"{{"design":"d","rule_version":"r",{ALL},"findings":[{{"code":"PHOENIX-DRC-001","severity":"error","state":"confirmed","object":"secret-net","message":"short"}}]}}"#
            ),
        );
        let unsure = file(
            work.path(),
            "unsure.json",
            r#"{"design":"d","rule_version":"r","areas_run":["board"]}"#.to_string(),
        );
        assert_eq!(call(&["report", &pass]).0, 0);
        let (code, text, _) = call(&["report", &fail]);
        assert_eq!(code, 1);
        assert!(text.contains("PHOENIX-DRC-001") && text.contains("secret-net"));
        let (code, text, _) = call(&["report", &unsure]);
        assert_eq!(code, 3);
        assert!(text.contains("PHOENIX-DVER-005"), "{text}");
        let (code, text, _) = call(&["report", "--redacted", &fail]);
        assert_eq!(code, 1);
        assert!(
            !text.contains("secret-net") && !text.contains("short"),
            "{text}"
        );
        let (code, hash, _) = call(&["hash", &pass]);
        assert_eq!((code, hash.trim().len()), (0, 64));
    }

    #[test]
    fn the_review_writes_the_document_with_the_exit_code_of_the_verdict() {
        let work = tempfile::tempdir().unwrap();
        let fail = file(
            work.path(),
            "fail.json",
            format!(
                r#"{{"design":"d-1","rule_version":"r9",{ALL},"findings":[{{"code":"PHOENIX-DRC-001","severity":"error","state":"confirmed","object":"secret-net","message":"short","rule":"gap v3"}}]}}"#
            ),
        );
        let (code, text, _) = call(&["review", &fail]);
        assert_eq!(code, 1);
        assert!(text.starts_with("# Design review\n\n- Design: d-1\n- Verdict: fail\n"));
        assert!(text.contains("| gap v3 | PHOENIX-DRC-001 | 1 |"), "{text}");
        assert!(text.contains("- `PHOENIX-DRC-001` confirmed secret-net: short"));
        let (_, hash, _) = call(&["hash", &fail]);
        assert!(text.contains(&format!("- Report hash (SHA-256): {}", hash.trim())));
        let (code, text, _) = call(&["review", "--redacted", &fail]);
        assert_eq!(code, 1);
        for secret in ["d-1", "secret-net", "short", "gap v3"] {
            assert!(!text.contains(secret), "{secret}");
        }
        assert!(text.contains("| PHOENIX-DRC-001 | error | confirmed | 1 |"));
        let cfg = file(
            work.path(),
            "c.cfg",
            "review_max_findings = 1\n".to_string(),
        );
        assert_eq!(call(&["--config", &cfg, "review", &fail]).0, 1);
        assert_eq!(call(&["review"]).0, 2);
        assert_eq!(call(&["review", "--redacted"]).0, 2);
        assert_eq!(call(&["review", "no-such-file.json"]).0, 2);
    }

    #[test]
    fn bad_input_and_bad_arguments_exit_2() {
        let work = tempfile::tempdir().unwrap();
        let broken = file(work.path(), "broken.json", "{".to_string());
        let (code, _, err) = call(&["report", &broken]);
        assert_eq!(code, 2);
        assert!(err.contains("cannot be read"));
        assert_eq!(call(&[]).0, 2);
        assert_eq!(call(&["report"]).0, 2);
        assert_eq!(call(&["report", "no-such-file.json"]).0, 2);
        assert_eq!(call(&["--config", "x"]).0, 2);
    }

    #[test]
    fn a_settings_file_changes_what_is_required() {
        let work = tempfile::tempdir().unwrap();
        let unsure = file(
            work.path(),
            "unsure.json",
            r#"{"design":"d","rule_version":"r","areas_run":["board"]}"#.to_string(),
        );
        let cfg = file(work.path(), "c.cfg", "required_areas = board\n".to_string());
        assert_eq!(call(&["--config", &cfg, "report", &unsure]).0, 0);
    }
}
