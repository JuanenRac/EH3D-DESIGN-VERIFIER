# EH3D-DESIGN-VERIFIER: scope

Status: **early code**. EH3D-DESIGN-VERIFIER has a library and a command line that gather the results of the checks and give a verdict; most of what this page lists is still planned. This page says what it is meant to do, what it will not do, and where it sits. Nothing has been run on a real design.

## Purpose

EH3D-DESIGN-VERIFIER is planned as an engineering verifier with cross-checks and quality control of the design.

Integral engineering verifier: collects the electrical, geometric, mechanical, thermal and manufacturing checks of a design in one reviewable report.

## What it is planned to do

* Collect design-rule checks, connectivity, schematic-to-board consistency, mechanical constraints, thermal checks and manufacturing rules in one report.
* Classify every finding as error, warning or information, with a stable code.
* Link each finding to the design object it concerns, by stable identity.
* Show the rule applied and the data used; repeat the analysis with the same rule version and get the same result.
* Tell CONFIRMED failures apart from INCONCLUSIVE results.
* Check a design against a manufacturing profile and explain what to change to make it fabricable.
* Export design-review reports. Planned as a library with a command-line interface and a server task kind.

## What it will not do

* It does not repair designs; it reports.
* It does not decide that a design is safe: limits, interlocks and emergency stops belong to the machine and its controller, and product safety to the applicable standards.
* It does not replace the rule engine of the board tool; it consumes and combines results.
* It keeps no private list of error codes: codes come from the registry of the PHOENIX family.

## Planned shape

A library without a window, written in Rust (planned), with a command-line interface and, where it applies, a kind of task for the server. Programs of the suite exchange work through project files or through the server, never by depending on each other.

The package is a single crate (a library and a command line, no window) with `report`, `review` and `hash`; the results of the checks are a JSON file and the settings are a file of `key = value` lines (`config/verifier.cfg` shows every key with its default). The checks themselves and the comparison with a manufacturing profile are not written.

## Where it sits

It belongs to the PHOENIX family of the EH3D Fabrication Suite: independent repositories that relate through versioned contracts. The design tool (EH3D-PHOENIX-PCB) does not depend at build time on the repositories that depend on it.
See DEPENDENCIES.md for the relations, REQUIREMENTS.md for the requirements and STATUS.md for the honest state.
