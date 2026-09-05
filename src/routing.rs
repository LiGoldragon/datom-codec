//! Target-selected descent through the datom concept.

use protos::{Conceivable, Protosizable, Route};

use crate::site::Incorporating;
use crate::{Datom, Datomic, Fault, IncorporationBudget, Site};

impl Route<Datom> for Datom {
    type Fault = Fault;
    type Budget = ();

    fn run(text: &str, (): Self::Budget) -> Result<Datom, Self::Fault> {
        Ok(text.protosize()?.conceive()?.1)
    }
}

impl<T: Datomic> Route<T> for Datom {
    type Fault = Fault;
    type Budget = IncorporationBudget;

    fn run(text: &str, budget: Self::Budget) -> Result<T, Self::Fault> {
        let situated = text.protosize()?.conceive()?;
        let mut budget = budget;
        Site {
            datom: &situated.1,
            at: &situated.0,
            budget: &mut budget,
        }
        .corporate()
    }
}
