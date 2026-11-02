# EH3D-DESIGN-VERIFIER: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Every requirement is `planned` today: nothing is implemented or verified. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| DVER-001 | Classify findings as error, warning or information with a stable code. | planned | automatic: reference cases | Each reference case yields the expected code and severity. | Not started: no limits known yet. |
| DVER-002 | Link every finding to the stable identity of the affected object. | planned | automatic | No finding without an object, unless the finding is about the whole design. | Not started: no limits known yet. |
| DVER-003 | Reproduce a report: same design and same rule version give the same result. | planned | automatic: regeneration of an older report | Identical findings and identical canonical report. | Not started: no limits known yet. |
| DVER-004 | Distinguish confirmed failures from inconclusive results. | planned | automatic: cases that cannot be decided | An undecidable case is never reported as pass. | Not started: no limits known yet. |
| DVER-005 | Check a design against a manufacturing profile and give the remedy. | planned | automatic | A violation names the profile limit and what to change. | Not started: no limits known yet. |
| DVER-006 | Export a design-review report. | planned | automatic plus review | The report lists rules, data and versions used. | Not started: no limits known yet. |
| DVER-007 | Report on a private corpus in redacted mode. | planned | automatic: leak test | Only codes, severities and counts leave the machine. | Not started: no limits known yet. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
