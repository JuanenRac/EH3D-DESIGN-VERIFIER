# EH3D-DESIGN-VERIFIER: status

## Current state

EH3D-DESIGN-VERIFIER is early code on a computer: a Rust library and a command line (`report`, `review`, `hash`) that gather the results of the checks from a JSON file, give the verdict, print the canonical report with its hash (also redacted) and write the review document. A seeded twin of the checks injects their faults and a recovery runs the failed area again. All of it has run only on data made by its own tests; the checks themselves are not here, no design is read and no requirement is validated.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 25 requirements: 17 implemented, 6 prototype, 2 planned and 0 validated. By progress: 17 done, 6 partial, 1 pending and 1 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* The exact list of checks of the first version and their order.
* Whether the review document needs a language other than English before a person signs it.

## What it needs from other repositories

* EH3D-PHOENIX-PCB: the design model and its rule results.
* EH3D-COMMON: the diagnostic crate.
* EH3D-MANUFACTURING-PROFILE: a profile format, for the check against a profile.

## Known technical debt and omitted tests

* 43 tests of the Rust code (nine of them keep this register true) and the 3 of the base; the Rust tests build their own results and never touch a real design.
* Ten of the tests run the simulation: the faults of a check that can be injected (an area that crashed, a finding that could not be decided, a malformed code, a finding without an object, a confirmed error), the recovery and the redacted review of a faulty run. What is pending is the proof against the real checks.
* The review document is compared with a text written by hand for one report; no review has been read by a person who signs it.
* `cargo deny` is not run yet.
