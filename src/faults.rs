//! The faults: pathed, converted from protos's, and themselves datomic.

use protos::{Extent, Integer, Path, Pathed, Text};

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
        Datom::Variant(self.to_owned(), Box::new(body))
    }
}

impl Datomic for Extent {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(Extent(p.position()?, p.position()?))
    }

    fn conceive(&self) -> Datom {
        Datom::Struct(vec![self.0.conceive(), self.1.conceive()])
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

    fn conceive(&self) -> Datom {
        Datom::Struct(vec![self.path.conceive(), self.extent.conceive()])
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
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }

    fn conceive(&self) -> Datom {
        match self {
            Self::Unclosed(enclosure) => "Unclosed".carrying(enclosure.conceive()),
            Self::Unopened(enclosure) => "Unopened".carrying(enclosure.conceive()),
            Self::Unterminated(boundary) => "Unterminated".carrying(boundary.conceive()),
            Self::Stray(boundary) => "Stray".carrying(boundary.conceive()),
        }
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

    fn conceive(&self) -> Datom {
        Datom::Struct(vec![self.extent.conceive(), self.problem.conceive()])
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
            "UnknownVariant" => Ok(Self::UnknownVariant(String::from(Carrying::<Text>::body(
                v,
            )?))),
            "Value" => Ok(Self::Value(String::from(Carrying::<Text>::body(v)?))),
            "Formless" => Ok(Self::Formless(v.body()?)),
            "OneValue" => Ok(Self::OneValue(v.body()?)),
            "Exhausted" => {
                v.nothing()?;
                Ok(Self::Exhausted)
            }
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }

    fn conceive(&self) -> Datom {
        match self {
            Self::Shape(expected, found) => {
                "Shape".carrying(Datom::Struct(vec![expected.conceive(), found.conceive()]))
            }
            Self::Arity(expected, found) => {
                "Arity".carrying(Datom::Struct(vec![expected.conceive(), found.conceive()]))
            }
            Self::UnknownVariant(name) => "UnknownVariant".carrying(Datom::Word(name.clone())),
            Self::Value(word) => "Value".carrying(Datom::Text(
                Text::try_from(word.as_str())
                    .expect("a datom problem value must be representable text"),
            )),
            Self::Formless(found) => "Formless".carrying(found.conceive()),
            Self::OneValue(count) => "OneValue".carrying(count.conceive()),
            Self::Exhausted => Datom::Word("Exhausted".to_owned()),
        }
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
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }

    fn conceive(&self) -> Datom {
        match self {
            Self::Structural(fault) => "Structural".carrying(fault.conceive()),
            Self::Conceptual(locus, problem) => {
                "Conceptual".carrying(Datom::Struct(vec![locus.conceive(), problem.conceive()]))
            }
            Self::Corporate(locus, problem) => {
                "Corporate".carrying(Datom::Struct(vec![locus.conceive(), problem.conceive()]))
            }
        }
    }
}
