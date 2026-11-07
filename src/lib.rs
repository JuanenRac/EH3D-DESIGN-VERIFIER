// SPDX-License-Identifier: GPL-3.0-or-later
//! EH3D-DESIGN-VERIFIER: the integral verification of a design in one reviewable report.
//!
//! The checks themselves (connectivity, rules, manufacturing, thermal) belong to other repositories of the suite. This crate gathers their results, checks that
//! each finding is well formed (a stable code `PHOENIX-<AREA>-<NNN>` and the identity of the object it concerns), and gives a verdict that never calls a result
//! a pass when it could not be decided: **confirmed** failures are kept apart from **inconclusive** results, and an area that was required but not run makes
//! the verdict inconclusive.
//!
//! The report is canonical: the same results and the same rule version give the same findings in the same order and the same hash. Nothing here reads the clock.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod config;
pub mod error;
pub mod report;
pub mod review;

pub use error::VerifyError;
pub use report::{Report, Verdict};

/// The stable code of a problem the verifier itself finds in the results. Codes are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// A finding whose code is not `PHOENIX-<AREA>-<NNN>`.
    BadCode,
    /// A finding about an object that names no object and is not about the whole design.
    MissingObject,
    /// A finding that could not be decided.
    Undecided,
    /// An area that is required was not run.
    AreaNotRun,
}

impl Code {
    /// The text of the code, such as `PHOENIX-DVER-001`.
    pub fn as_str(self) -> &'static str {
        match self {
            Code::BadCode => "PHOENIX-DVER-001",
            Code::MissingObject => "PHOENIX-DVER-002",
            Code::Undecided => "PHOENIX-DVER-004",
            Code::AreaNotRun => "PHOENIX-DVER-005",
        }
    }
}
