# EH3D-DESIGN-VERIFIER: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because nothing of this program has been run on a real design or with the real checks of the suite.

The register has 25 requirements: 17 implemented, 6 prototype, 2 planned and 0 validated. By progress: 17 done, 6 partial, 1 pending and 1 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `DVER-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against a seeded twin of the checks of the suite whose faults are injected on purpose, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs` (`src/file::name` is a test inside `src/file.rs`), which must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## Findings and the verdict

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| DVER-001 | Classify findings as error, warning or information with a stable code. | prototype | partial | host-test | `src/report::badly_formed_codes_and_findings_without_an_object_are_reported` | It checks the form of the code (`PHOENIX-<AREA>-<NNN>`) and keeps the severity the check gave; it does not decide whether a severity is right. |
| DVER-002 | Link every finding to the stable identity of the affected object. | prototype | partial | host-test | `src/report::badly_formed_codes_and_findings_without_an_object_are_reported` | A finding must name an object or say that it is about the whole design; whether the identity exists in the design is not checked, because the design is not read. |
| DVER-003 | Reproduce a report: the same results in any order give the same report. | prototype | partial | host-test | `src/report::the_same_results_in_another_order_give_the_same_report` | The report is sorted and hashed from the results it was given; regenerating an older report needs the checks themselves, which are not here. |
| DVER-004 | Tell confirmed failures from inconclusive results. | prototype | partial | host-test | `src/report::an_undecided_finding_is_never_a_pass` | An inconclusive finding (other than information) makes the verdict inconclusive; a confirmed failure still wins. |
| DVER-005 | Check a design against a manufacturing profile and give the remedy. | planned | pending | none | `planned:profile::a_violation_names_the_profile_limit_and_what_to_change` | Not written: it needs the profile format of EH3D-MANUFACTURING-PROFILE and the limits it will carry. |
| DVER-006 | Export a design-review report that lists the rules, the data and the versions used. | prototype | partial | host-test | `src/review::the_review_lists_the_rules_the_data_and_the_findings` | The text lists the rule names it was given and cannot tell whether they are the right ones; its words are English only, it is not signed and it has only been tried on data made by the tests. |
| DVER-007 | Report on a private corpus in redacted mode: only codes, severities and counts leave the machine. | prototype | partial | host-test | `src/report::the_redacted_counts_carry_no_message_and_no_object` | Tested for leaks of the message and the object in the counts; the full report prints names. |
| DVER-008 | A confirmed error fails the verdict and a confirmed warning only when the setting says so. | implemented | done | host-test | `src/report::a_confirmed_error_fails_and_a_confirmed_warning_only_when_the_setting_says_so` | - |
| DVER-009 | A nothing-found report with every required area run is a pass. | implemented | done | host-test | `src/report::nothing_found_and_everything_run_is_a_pass` | - |
| DVER-010 | A required area that did not run makes the verdict inconclusive. | implemented | done | host-test | `src/report::a_required_area_that_did_not_run_makes_the_verdict_inconclusive` | - |
| DVER-011 | The hash of a fixed report is a known value, so a change of the format cannot go unnoticed. | implemented | done | host-test | `src/report::the_hash_of_a_fixed_report_is_the_known_one` | - |
| DVER-012 | Bad JSON, unknown fields and the size limits are refused without a panic. | implemented | done | host-test | `src/report::bad_json_unknown_fields_and_the_size_limits_are_refused` | - |

## The review document and the redacted mode

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| DVER-013 | The same report gives the same review text. | implemented | done | host-test | `src/review::the_same_report_gives_the_same_text` | - |
| DVER-014 | The review lists the most serious findings first up to a limit of the settings and counts the rest. | implemented | done | host-test | `src/review::the_limit_lists_the_most_serious_first_and_counts_the_rest` | - |
| DVER-015 | The redacted review carries only codes and counts. | implemented | done | host-test | `src/review::the_redacted_review_carries_only_codes_and_counts` | - |
| DVER-016 | Text written by a faulty check can neither break the layout of the review nor leave the machine in a code or a rule version. | implemented | done | host-test | `src/review::text_from_the_results_cannot_break_the_layout` | - |

## The command line and the settings

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| DVER-017 | The exit code follows the verdict: 0 pass, 1 fail, 3 inconclusive. | implemented | done | host-test | `src/cli::the_exit_code_follows_the_verdict` | - |
| DVER-018 | Bad input and bad arguments exit with 2. | implemented | done | host-test | `src/cli::bad_input_and_bad_arguments_exit_2` | - |
| DVER-019 | The settings file changes what is required. | implemented | done | host-test | `src/cli::a_settings_file_changes_what_is_required` | - |
| DVER-020 | The example settings file equals the defaults of the code. | implemented | done | host-test | `src/config::the_example_file_is_the_default_file_and_parses_to_the_defaults` | - |

## Simulation of the checks

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| DVER-021 | A seeded twin of the checks gives the same results and the same hash for the same seed. | implemented | done | simulator | `src/simulation::the_same_seed_gives_the_same_results_and_the_same_hash` | The twin imitates the form of the results, not the checks of a real design tool. |
| DVER-022 | Every fault that can be injected into a check is reported with its code. | implemented | done | simulator | `src/simulation::every_fault_of_a_check_is_reported_with_its_code` | - |
| DVER-023 | The recovery runs the area again and never hides a confirmed error. | implemented | done | simulator | `src/simulation::a_confirmed_error_fails_and_recovery_does_not_hide_it` | - |
| DVER-024 | The redacted review of a faulty run leaks no object or design name. | implemented | done | simulator | `src/simulation::the_redacted_review_of_a_faulty_run_leaks_no_object_or_design_name` | - |
| DVER-025 | Gather the results of the real checks of the suite. | planned | blocked | none | `planned:real_checks::the_results_of_the_board_rules_give_the_expected_verdict` | Waits for the rule results of EH3D-PHOENIX-PCB; until then only data made by the tests has been used. |
