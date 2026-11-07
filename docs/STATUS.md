# EH3D-DESIGN-VERIFIER: status

## Current state

EH3D-DESIGN-VERIFIER is early code. It has the base that every repository of the suite carries (manifest, README files in eight languages, changelog, CI, shared checks), the written scope, requirements, dependencies, threat model and decisions, and a Rust library and command line that gather the results of the checks from a JSON file, give the verdict, print the canonical report with its hash (also redacted) and write the review document. They have run only on data made by their own tests; the checks themselves are not here, no design is read, and no requirement is validated.

## Capabilities

The official state of each capability is in REQUIREMENTS.md: 7 requirements: 6 `prototype` and 1 `planned` (DVER-005). This page never states a state that REQUIREMENTS.md does not.

## Open questions

* The exact list of checks of the first version and their order.

## Needs from other repositories

* EH3D-PHOENIX-PCB: the design model and its rule results
* EH3D-COMMON: the diagnostic crate
* EH3D-MANUFACTURING-PROFILE: a profile format

## Known technical debt and omitted tests

21 tests of the Rust code and the 3 of the base; the Rust tests build their own results and never touch a real design. The review document is compared with a text written by hand for one report; no review has been read by a person who signs it. `cargo deny` is not run yet.
