# EH3D-DESIGN-VERIFIER: dependencies

One row per relation with another repository. The kind is conceptual, compile-time or run-time. The state of a contract is: not written, drafted, accepted, frozen or published. A repository existing does not mean that its contract is ready: this project does not implement or consume a contract that is not accepted.

| From | To | Kind | Contract | Contract state | Minimum version | Tested version | Compatibility test | Change owner | What it blocks | Migration |
|---|---|---|---|---|---|---|---|---|---|---|
| EH3D-DESIGN-VERIFIER | EH3D-PHOENIX-PCB | compile | design model and stable identities | drafted | not set | not set | to be written with the first code | the owner of the contract | all of it | to be defined |
| EH3D-DESIGN-VERIFIER | EH3D-COMMON | compile | diagnostic crate and project container | drafted | not set | not set | to be written with the first code | the owner of the contract | codes and reports | to be defined |
| EH3D-DESIGN-VERIFIER | EH3D-MANUFACTURING-PROFILE | runtime | profile format (not written) | not written | not set | not set | to be written with the first code | the owner of the contract | the fabrication checks | to be defined |
| EH3D-DESIGN-VERIFIER | EH3D-THERMAL-LAB | conceptual | thermal results to check | not written | not set | not set | to be written with the first code | the owner of the contract | the thermal checks | to be defined |
| EH3D-DESIGN-VERIFIER | EH3D-VALIDATION-LAB | conceptual | reference cases | not written | not set | not set | to be written with the first code | the owner of the contract | its own test evidence | to be defined |

Rules: repositories are built next to each other as sibling folders and each declares in its manifest the versions of the others that it needs, once those exist. A lower layer never depends on a higher one. The manifest field for build dependencies stays empty until code exists.
