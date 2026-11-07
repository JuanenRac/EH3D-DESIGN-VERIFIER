# Changelog

All notable changes to this project are documented here.

## [Unreleased]

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
