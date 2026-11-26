# Changelog

All notable changes to this project are documented here.

## [Unreleased]
- **A text of the results file can no longer write lines of its own in `report`:** a line feed in the design, the rules, a code, an object or a message could add a false verdict or hash line. Control characters are printed as spaces, as `review` already did.
- **The simulated recovery removes every code the report calls malformed**, such as one in lower case.
- **7 more tests** of the Rust code: a seeded set judged by the plain rule of the verdict, a results file cut at every place, words that are not a severity or a state and the limit of the review.
- **More tests, for the shared tools of the base and for the report of the verifier:** `tools/test_shared_tools.py` adds 65 tests that run the lessons linter, the workflow check, the CHANGELOG check of a pull request and the banner check on small inputs made by the test itself (findings, baselines, exit codes and the shape of what they print). The Rust code gets 14 more: which codes are printed as they are and which as a fixed marker, a confirmed failure wins over an undecided finding and an undecided error is not a failure, information never fails, the areas are compared without the case, a change of the design, the rules or one finding changes the hash, ten thousand findings are judged and counted without a panic, the redacted counts never carry the text of a bad code, a text that is not a results file is an error, the most findings is read and one more is not, and the severity and the state are written in lower case.
- **The register of requirements names a test for each row:** `docs/REQUIREMENTS.md` has one row for each capability with an identifier (`DVER-001` on), its state (planned, prototype, implemented or validated), how far it is (done, partial, pending or blocked), its evidence (simulator, host-test or none), the test that shows it and its limits; a capability that is not written names the test that will show it. `tests/reqs.rs` reads the table and fails when a test does not exist, when a state does not agree with the progress or the evidence, when `docs/STATUS.md` does not say the same numbers, when a README does not link the documents or when a document holds a date or a private reference. `docs/SCOPE.md`, `docs/STATUS.md` and `docs/NOT_YET.md` are rewritten to the same shape and the eight READMEs link the five documents.
- The shared workflow checker (`tools/workflow_hygiene.py`) is the current one of the documentation hub: the filters under a trigger (branches, paths, types) are no longer read as events.

## [0.0.4] - A threat model, a pinned report hash and a simulation that never removes a confirmed error

- **A test fixes the known value of the hash of a fixed report:** until now the tests compared two calculations of the same code, so an accidental change of the order of the fields or of the way the content is written would have changed every hash that was ever quoted without a test failing. The test writes the expected hash of a fixed case.

- **The threat model says what the code already does:** the rows of `docs/THREAT_MODEL.md` that have a mitigation in the code and a test for it are marked `partial` and name the tests and what is still not covered; the rows without either stay `planned`.

- **The recovery of the simulation never removes a confirmed error:** a finding that is an error and confirmed stays even when its code is malformed or it names no object, so the verdict of a design with such a finding stays `fail`; a test combines the two faults.
- **A version of the rules or an area that is not a plain name is never printed:** the review prints the fixed text `(malformed value)` instead of a rule version or an area run that is empty, longer than 64 characters or has anything but letters, digits, dots, dashes, underscores and plus signs.
- **A code that is not a code is never printed:** the redacted counts, the redacted review and the findings of the review print the fixed text `(malformed code)` instead of a finding code that does not have the form of a rule code, so text written there by a faulty check can neither leave the machine in the redacted outputs nor break the layout of the review. Tested with a code made of private text and line breaks.

- **Simulation mode:** add `src/simulation.rs`, a host twin of the checks of the suite: a seeded checker (the same seed gives the same results and the same hash), five injectable faults (a crashed area, an undecided finding, a malformed code, a finding without an object, a confirmed error) and a recovery that runs the area again and never hides a confirmed error. Nine deterministic tests cover them. The manifest now says `evidence: simulator`; nothing has run against the real checks.

## [0.0.3] - Review document and a report that keeps the areas run

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
