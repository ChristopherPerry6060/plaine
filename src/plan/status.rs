#![allow(dead_code)]
use std::{fmt::Display, path::Path};

use crate::{Branch, Brn};

use super::{Entry, Plan};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct StatusRecord {
    branch: Branch,
    status: Status,
}

impl Status {
    pub fn mark_for_check<P>(path: P, branch: Brn) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let uuid = Uuid::new_v4();
        let full = format!("{branch}_{uuid}.json");

        let mut opath = path.as_ref().to_path_buf();
        opath.push(full);
        let sr = Self::Check;

        let json = serde_json::to_string(&sr)?;
        Ok(std::fs::write(opath, json)?)
    }
}

/// The current stage of progress for [`Plan`]s.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Status {
    #[default]
    Open,
    Check,
    Confirm,
    Measure,
    BoxContents,
    CaseLabel,
    Staged,
    Shipped,
}
impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Open => write!(f, "Open"),
            Status::Check => write!(f, "Check"),
            Status::Confirm => write!(f, "Confirm"),
            Status::Measure => write!(f, "Measure"),
            Status::BoxContents => write!(f, "BoxContents"),
            Status::CaseLabel => write!(f, "CaseLabel"),
            Status::Staged => write!(f, "Staged"),
            Status::Shipped => write!(f, "Shipped"),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Warn {
    Upc,
    Cases,
    Sku,
    Units,
    TeamLift(Vec<Entry>),
    Weight(Vec<Entry>),
    Dimesions(Vec<Entry>),
    MissingInfo(Vec<Entry>),
    Special,
    Forbid,
}

type Warnings = Option<Vec<Warn>>;
type Upc = Option<String>;
type Units = Option<u32>;
type Cases = Option<u32>;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CheckRow<P>
where
    P: Plan + Serialize,
{
    predicate: P,
    upc: Upc,
    units: Units,
    cases: Cases,
    warn: Warnings,
}

impl CheckRow<Vec<Entry>> {
    fn new(predicate: Vec<Entry>) -> Self {
        Self {
            predicate,
            ..Default::default()
        }
    }
    /// Checks [`Self`] and returns [`Warn`]ings of issues.
    ///
    /// See [`Warn`] for a description of the attributes being checked for.
    fn all_checks(&self) -> Warnings {
        let mut warnings = vec![];
        if !self.same_upc() {
            warnings.push(Warn::Upc);
        };
        if !self.same_num_cases() {
            warnings.push(Warn::Cases);
        };
        if !self.same_num_units() {
            warnings.push(Warn::Units);
        };
        if let Some(team_lift_entries) = self.cases_need_team_lift() {
            let tl = Warn::TeamLift(team_lift_entries);
            warnings.push(tl);
        };
        if let Some(too_heavy) = self.cases_too_heavy() {
            let attn = Warn::Dimesions(too_heavy);
            warnings.push(attn);
        };
        if let Some(dims_attn) = self.cases_too_long() {
            let attn = Warn::Dimesions(dims_attn);
            warnings.push(attn);
        };
        if let Some(needs_info) = self.needs_info() {
            let info = Warn::MissingInfo(needs_info);
            warnings.push(info)
        }
        Some(warnings)
    }

    /// Returns entries that may require a team lift sticker.
    fn cases_need_team_lift(&self) -> Option<Vec<Entry>> {
        let team_lift: Vec<_> = self
            .predicate
            .entries()
            .into_iter()
            .filter(|case| {
                let weight = case.get_total_pounds().unwrap_or_default();
                let units = case.get_units();
                units.eq(&1) && weight.gt(&49.00)
            })
            .collect();
        (!team_lift.is_empty()).then_some(team_lift)
    }

    /// Returns entries that have more than one unit and may be over 49 pounds.
    fn cases_too_heavy(&self) -> Option<Vec<Entry>> {
        let scary_dims: Vec<_> = self
            .predicate
            .entries()
            .into_iter()
            .filter(|case| {
                let weight = case.get_total_pounds().unwrap_or_default();
                let units = case.get_units();
                units.gt(&1) && weight.gt(&49.00)
            })
            .collect();
        (!scary_dims.is_empty()).then_some(scary_dims)
    }

    /// Returns entries that have dimensions greater than 24 inches.
    fn cases_too_long(&self) -> Option<Vec<Entry>> {
        let scary_dims: Vec<_> = self
            .predicate
            .entries()
            .into_iter()
            .filter(|case| {
                let dims = case.get_case_dimensions().unwrap_or_default();
                dims.into_iter().any(|dim| dim > 24.00)
            })
            .collect();
        (!scary_dims.is_empty()).then_some(scary_dims)
    }

    /// Returns entries that need additional information.
    fn needs_info(&self) -> Option<Vec<Entry>> {
        let missing_info: Vec<_> = self
            .predicate
            .entries()
            .into_iter()
            .filter(|case| {
                let missing_dims = case.get_case_dimensions().is_none();
                let missing_weight = case.get_total_pounds().is_none();
                missing_dims || missing_weight
            })
            .collect();
        (!missing_info.is_empty()).then_some(missing_info)
    }

    /// Returns true if the number of cases matches the predicate in [`Self`].
    fn same_num_cases(&self) -> bool {
        let Some(input) = self.cases else {return false};
        let expectation = self.predicate.number_of_real_cases() as u32;
        input == expectation
    }

    /// Returns true if the number of units matches the predicate in [`Self`].
    fn same_num_units(&self) -> bool {
        let Some(input) = self.units else {return false};
        let e: i32 = self.predicate.units_of_skus().values().sum();
        let Ok(expectation): Result<u32, _> = e.try_into() else {return false};
        input == expectation
    }

    /// Returns true if Upc matches the predicate in [`Self`].
    fn same_upc(&self) -> bool {
        let input_upc = &self.upc;
        self.predicate
            .entries()
            .into_iter()
            .any(|x| input_upc == x.get_upc())
    }
}
