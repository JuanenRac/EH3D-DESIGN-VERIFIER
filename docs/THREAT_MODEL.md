# EH3D-DESIGN-VERIFIER: threat model

Basic model for early code. The rows marked partial have a mitigation written and tests for it, run only on data made by the tests themselves; the rows marked planned have nothing yet. No row is assessed against real data.

| Threat | Planned mitigation | Planned test | State | Residual risk |
|---|---|---|---|---|
| A report that hides a failure (an undecidable case shown as pass) | an undecided finding is never a pass, and an area that did not run or crashed makes the verdict inconclusive | `an_undecided_finding_is_never_a_pass`, `a_required_area_that_did_not_run_makes_the_verdict_inconclusive`, `a_crashed_area_makes_the_verdict_inconclusive` | partial | only the cases the tests build; no real design has been reviewed |
| Reports that leak names, values or paths from a private design | the redacted mode carries only codes and counts, never a message or an object | `the_redacted_counts_carry_no_message_and_no_object`, `the_redacted_review_of_a_faulty_run_leaks_no_object_or_design_name` | partial | the full review carries names by design; only the redacted mode is checked for leaks |
| Corrupt or malicious project or input files (oversized entries, path tricks, deep nesting, resource exhaustion) | the JSON reader refuses bad syntax, unknown fields and oversized input, and text from the results cannot break the layout of the review | `bad_json_unknown_fields_and_the_size_limits_are_refused`, `text_from_the_results_cannot_break_the_layout` | partial | hostile samples written by hand; no fuzzing yet |
| Temporary files, logs and reports that reveal private designs | no private data in outputs; redaction | automated pattern search | planned | not assessed: nothing is implemented |
| Compromised dependency or binary | pinned versions, licence and advisory checks, software bill of materials for releases | CI checks | planned | not assessed: nothing is implemented |
| Power loss or forced close during a save | atomic write: temporary file, then rename; old file never deleted first | interruption tests at controlled points | planned | not assessed: nothing is implemented |

Minimum negative tests once there is code: hostile files and resource limits; secrets absent from logs, errors and reports; pinned dependencies and licence review before publishing. Report a vulnerability privately as described in SECURITY.md, without attaching private designs.
