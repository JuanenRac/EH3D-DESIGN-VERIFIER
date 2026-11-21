# EH3D-DESIGN-VERIFIER: scope

Status: **early code**. EH3D-DESIGN-VERIFIER is a library and a command line that gather the results of the checks of a design from a JSON file, give the verdict and write the review document. It has been tried on data made by its own tests only. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The integral engineering verifier of the PHOENIX family: the electrical, geometric, mechanical, thermal and manufacturing checks of a design end in one reviewable report in which a confirmed failure is kept apart from a result that could not be decided. The checks themselves belong to other repositories of the suite; this program gathers what they found.

## What it does

* Reads the results of the checks (findings with a code, a severity, a state and the object they concern) from a JSON file within a size limit, refusing unknown fields.
* Checks that each finding has a code of the form `PHOENIX-<AREA>-<NNN>` and names an object (or says that it is about the whole design), and reports the ones that do not.
* Gives the verdict: fail when a failure is confirmed, inconclusive when something could not be decided or a required area did not run, pass otherwise; the exit code of the command follows the verdict.
* Prints the canonical report (the same results give the same findings in the same order and the same hash), also redacted to counts only.
* Writes the review document: the verdict, the rules applied, the data used, the version of the tool and the findings up to a limit set in the settings.
* Runs a seeded twin of the checks with injectable faults and a recovery that runs an area again and never hides a confirmed error.

## What it will not do

* It does not run any check and does not read a design: it gathers the results that other programs give it.
* It does not repair a design; it reports.
* It does not compare a design with a manufacturing profile yet (see `REQUIREMENTS.md`).
* It does not keep a private list of error codes: codes come from the registry of the PHOENIX family.
* It does not claim that anything was proven on a real design: the evidence of every capability is `simulator` or `host-test`.

## Where it sits

* **It reads:** a JSON file with the results of the checks and a settings file (`config/verifier.cfg` shows every key with its default).
* **It writes:** the report, the redacted counts, the hash and the review document, to the standard output or to a file the person names; it never writes to the results it reads.
* **It is read by:** nothing reads it yet: the server and the clients will read its report as a file once the checks exist.
* **It depends on:** nothing of the suite at build time; the checks it will gather come from EH3D-PHOENIX-PCB and the other engines.

## What it never decides

Whether a design is safe, and the limits, the interlocks and the emergency stop of any machine, belong to the machine and to the standards that apply, not to this program. A verdict of pass means that no check found a confirmed failure and that every required area ran; it is not a certificate. Nothing here starts motion.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.
