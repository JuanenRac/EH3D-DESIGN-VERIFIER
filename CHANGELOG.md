# Changelog

All notable changes to this project are documented here.

## [Unreleased]

- **The shared validation tools are updated:** the validator also runs the workflow and lessons checks of the suite with their latest rules, including a check to run before every push.
- **The CI validation checks the workflows and the lessons of past CI failures against a baseline:** `tools/workflow_hygiene.py` and `tools/lessons_lint.py` (copies of the shared tools) run in the validation, and only findings that are not in `tools/workflow_hygiene_baseline.json` and `tools/lessons_lint_baseline.json` fail it, so what exists today stays as it is and a new one is caught. `python tools/workflow_hygiene.py --root . --detail` and `python tools/lessons_lint.py --repo .` list the findings.
- **The validation of the CI is the reusable one of the documentation hub:** the job `validate` calls `validate.yml` of EH3D-DOCS (pinned by commit) and the check of the CHANGELOG is a job of its own.
- **Pilot of the shared validation workflow:** the CI also calls the reusable validation of the documentation hub next to its own validation job, so both can be compared on the same commit.
- **The CI is split in two workflows:** the validation of the repository (manifest, documentation, pictures, tools) stays in `ci.yml` and the Rust jobs move to `rust.yml`, which does not run for a change that only touches Markdown files or pictures, so a change of the documentation no longer builds the code on two systems, and they keep a cache of the build. The names of the jobs are the same.
- **The Rust toolchain is pinned:** `rust-toolchain.toml` fixes the version of Rust (the same one in every repository of the suite) with `rustfmt` and `clippy`, the workflows install that version and not `stable`, so a new release of Rust cannot turn the CI red by itself.
- **Review document:** add the `review` command (also `review --redacted`), which writes the report as text to file and sign: the verdict, the rule version, the hash of the report, the version of the tool, the areas that were run, required and missing, the rules applied with the number of findings of each, the findings by severity and the problems found in the results. The same report always gives the same text, text from the results cannot break its layout, and the redacted form carries only codes, severities, states and counts.
- **New setting:** `review_max_findings` (default 200) limits the findings listed one by one in the review; the rest are counted. `config/verifier.cfg` is the example settings file and a test requires it to equal the defaults of the code.
- **The report keeps the areas that were run**, in lower case and in order, so that the review can say which required areas are missing.

## [0.0.2] - First release: the library, the command line, the stable error codes and the tests

- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.

- Add the library: the results of the checks of a design as findings with a stable code, a severity, a state (confirmed or inconclusive) and the identity of the object, the report with a verdict, and the canonical order and hash of the report.
- Report, with stable codes `PHOENIX-DVER-001`, `-002`, `-004` and `-005`: a code that is not `PHOENIX-<AREA>-<NNN>`, a finding without an object, findings that could not be decided and a required area that was not run.
- Add the command line (`report`, `report --redacted`, `hash`) and a settings file; exit code 3 means inconclusive. Run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
