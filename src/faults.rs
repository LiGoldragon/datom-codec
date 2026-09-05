//! The faults: pathed, converted from protos's, and themselves datomic.

use std::convert::Infallible;

use protos::{Conceivable, Extent, Integer, Path, Pathed, Situated, Situation, Symbol, Text, Word};

use crate::anatomy::{Datom, Fault, Locus, Problem};
use crate::kinds::{Carrying, Datomic, Headed, Positional, Sited};
use crate::site::Site;

impl From<protos::Fault> for Fault {
    fn from(fault: protos::Fault) -> Self {
        Fault::Structural(fault)
    }
}

impl Pathed for Fault {
    fn path(&self) -> &[Integer] {
        match self {
            Fault::Structural(_) => &[],
            Fault::Conceptual(locus, _) | Fault::Corporate(locus, _) => &locus.path,
        }
    }

    fn within(self, index: Integer) -> Self {
        match self {
            Fault::Structural(fault) => Fault::Structural(fault),
            Fault::Conceptual(mut locus, problem) => {
                locus.path.insert(0, index);
                Fault::Conceptual(locus, problem)
            }
            Fault::Corporate(mut locus, problem) => {
                locus.path.insert(0, index);
                Fault::Corporate(locus, problem)
            }
        }
    }
}

/// The kind whose capability builds a variant datom.
trait Heading {
    fn carrying(&self, body: Datom) -> Datom;
}

impl Heading for str {
    fn carrying(&self, body: Datom) -> Datom {
        Datom::Variant(Symbol::try_from(self).unwrap(), Box::new(body))
    }
}

impl Datomic for Extent {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(Extent(p.position()?, p.position()?))
    }
}

impl Conceivable<Datom> for Extent {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        Ok(Situated(
            Situation {
                extent: Extent(0, 0),
                children: vec![],
            },
            Datom::Struct(vec![self.0.conceive()?.1, self.1.conceive()?.1]),
        ))
    }
}

impl Datomic for Locus {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(Locus {
            path: Positional::<Path>::position(&mut p)?,
            extent: p.position()?,
        })
    }
}

impl Conceivable<Datom> for Locus {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        Ok(Situated(
            Situation {
                extent: Extent(0, 0),
                children: vec![],
            },
            Datom::Struct(vec![self.path.conceive()?.1, self.extent.conceive()?.1]),
        ))
    }
}

impl Datomic for protos::Problem {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let v = site.variant()?;
        match v.name {
            "Unclosed" => Ok(Self::Unclosed(v.body()?)),
            "Unopened" => Ok(Self::Unopened(v.body()?)),
            "Unterminated" => Ok(Self::Unterminated(v.body()?)),
            "Stray" => Ok(Self::Stray(v.body()?)),
            "OneForm" => Ok(Self::OneForm(v.body()?)),
            other => Err(v.reject(Problem::UnknownVariant(Word::try_from(other).unwrap()))),
        }
    }
}

impl Conceivable<Datom> for protos::Problem {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        let datom = match self {
            Self::Unclosed(enclosure) => "Unclosed".carrying(enclosure.conceive()?.1),
            Self::Unopened(enclosure) => "Unopened".carrying(enclosure.conceive()?.1),
            Self::Unterminated(boundary) => "Unterminated".carrying(boundary.conceive()?.1),
            Self::Stray(boundary) => "Stray".carrying(boundary.conceive()?.1),
            Self::OneForm(count) => "OneForm".carrying(count.conceive()?.1),
        };
        Ok(Situated(
            Situation {
                extent: Extent(0, 0),
                children: vec![],
            },
            datom,
        ))
    }
}

impl Datomic for protos::Fault {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(protos::Fault {
            extent: p.position()?,
            problem: p.position()?,
        })
    }
}

impl Conceivable<Datom> for protos::Fault {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        Ok(Situated(
            Situation {
                extent: Extent(0, 0),
                children: vec![],
            },
            Datom::Struct(vec![self.extent.conceive()?.1, self.problem.conceive()?.1]),
        ))
    }
}

impl Datomic for Problem {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let v = site.variant()?;
        match v.name {
            "Shape" => {
                let mut p = v.positions(2)?;
                Ok(Self::Shape(p.position()?, p.position()?))
            }
            "Arity" => {
                let mut p = v.positions(2)?;
                Ok(Self::Arity(p.position()?, p.position()?))
            }
            "UnknownVariant" => Ok(Self::UnknownVariant(
                Word::try_from(Carrying::<Text>::body(v)?.as_ref())
                    .expect("a readable unknown variant is a word"),
            )),
            "Value" => {
                let crate::anatomy::Meaning::Plain(value) =
                    Carrying::<crate::anatomy::Meaning>::body(v)?;
                Ok(Self::Value(value))
            }
            "Formless" => Ok(Self::Formless(v.body()?)),
            "OneValue" => Ok(Self::OneValue(v.body()?)),
            "Exhausted" => {
                v.nothing()?;
                Ok(Self::Exhausted)
            }
            "BudgetExhausted" => {
                v.nothing()?;
                Ok(Self::BudgetExhausted)
            }
            other => Err(v.reject(Problem::UnknownVariant(Word::try_from(other).unwrap()))),
        }
    }
}

impl Conceivable<Datom> for Problem {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        let datom = match self {
            Self::Shape(expected, found) => "Shape".carrying(Datom::Struct(vec![
                expected.conceive()?.1,
                found.conceive()?.1,
            ])),
            Self::Arity(expected, found) => "Arity".carrying(Datom::Struct(vec![
                expected.conceive()?.1,
                found.conceive()?.1,
            ])),
            Self::UnknownVariant(name) => "UnknownVariant".carrying(Datom::Word(name.clone())),
            Self::Value(value) => "Value".carrying(Datom::Meaning(value.clone())),
            Self::Formless(found) => "Formless".carrying(found.conceive()?.1),
            Self::OneValue(count) => "OneValue".carrying(count.conceive()?.1),
            Self::Exhausted => {
                Datom::Word(Word::try_from("Exhausted").expect("Exhausted is a word"))
            }
            Self::BudgetExhausted => {
                Datom::Word(Word::try_from("BudgetExhausted").expect("BudgetExhausted is a word"))
            }
        };
        Ok(Situated(
            Situation {
                extent: Extent(0, 0),
                children: vec![],
            },
            datom,
        ))
    }
}

impl Datomic for Fault {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let v = site.variant()?;
        match v.name {
            "Structural" => Ok(Self::Structural(v.body()?)),
            "Conceptual" => {
                let mut p = v.positions(2)?;
                Ok(Self::Conceptual(p.position()?, p.position()?))
            }
            "Corporate" => {
                let mut p = v.positions(2)?;
                Ok(Self::Corporate(p.position()?, p.position()?))
            }
            other => Err(v.reject(Problem::UnknownVariant(Word::try_from(other).unwrap()))),
        }
    }
}

impl Conceivable<Datom> for Fault {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        let datom = match self {
            Self::Structural(fault) => "Structural".carrying(fault.conceive()?.1),
            Self::Conceptual(locus, problem) => "Conceptual".carrying(Datom::Struct(vec![
                locus.conceive()?.1,
                problem.conceive()?.1,
            ])),
            Self::Corporate(locus, problem) => "Corporate".carrying(Datom::Struct(vec![
                locus.conceive()?.1,
                problem.conceive()?.1,
            ])),
        };
        Ok(Situated(
            Situation {
                extent: Extent(0, 0),
                children: vec![],
            },
            datom,
        ))
    }
}
