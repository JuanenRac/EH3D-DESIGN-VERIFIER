# EH3D-DESIGN-VERIFIER: status

## Current state

EH3D-DESIGN-VERIFIER is early code. Version 0.0.1 holds only the base that every repository of the suite carries (manifest, README files in eight languages, changelog, CI, shared checks) and the written scope, requirements, dependencies, threat model and decisions. There is no code, nothing has been run on a machine, and no requirement is implemented.

## Capabilities

The official state of each capability is in REQUIREMENTS.md: 7 requirements: 5 `prototype` and 2 `planned` (DVER-005 and DVER-006). This page never states a state that REQUIREMENTS.md does not.

## Open questions

* The exact list of checks of the first version and their order.

## Needs from other repositories

* EH3D-PHOENIX-PCB: the design model and its rule results
* EH3D-COMMON: the diagnostic crate
* EH3D-MANUFACTURING-PROFILE: a profile format

## Known technical debt and omitted tests

13 tests of the Rust code and the 3 of the base; the Rust tests build their own results and never touch a real design. `cargo deny` is not run yet.
