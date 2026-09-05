//! A site: a datom at its situation, and the forms a corporate reader takes it as.

use std::borrow::Cow;

use protos::{Glyphing, Incorporable, Integer, Locating, Pathed, Separator, Situation, Text};

use crate::anatomy::{
    Budgeted, Datom, Expected, Fault, Found, IncorporationBudget, Locus, Problem,
};
use crate::kinds::{Carrying, Counted, Datomic, Headed, Positional, Sited};

/// A datom at its situation: what a corporate reader is handed.
#[derive(Debug)]
pub struct Site<'a> {
    /// The datom.
    pub datom: &'a Datom,
    /// Where it is.
    pub at: &'a Situation,
    /// The caller-owned allowance shared by this whole corporate descent.
    pub(crate) budget: &'a mut IncorporationBudget,
}

/// The positions of a struct or the elements of a vector, read in turn.
#[derive(Debug)]
pub struct Positions<'a> {
    datoms: &'a [Datom],
    at: &'a Situation,
    index: usize,
    body: bool,
    budget: &'a mut IncorporationBudget,
}

/// A variant: its name, and its body if it carries one.
#[derive(Debug)]
pub struct Variant<'a> {
    /// The variant's name.
    pub name: &'a str,
    body: Option<Site<'a>>,
    at: &'a Situation,
}

/// The kind whose capability rejoins a chain of words with the dot.
trait Chaining {
    fn chain(&self, out: &mut String) -> bool;
}

trait Finding {
    fn found(&self) -> Found;
}

impl Finding for Datom {
    fn found(&self) -> Found {
        match self {
            Self::Variant(..) => Found::Variant,
            Self::Struct(_) => Found::Struct,
            Self::Vector(_) => Found::Vector,
            Self::Text(_) => Found::Text,
            Self::Meaning(_) => Found::Meaning,
            Self::Word(_) => Found::Word,
        }
    }
}

trait Refusing {
    fn refuse(&self, problem: Problem) -> Fault;
}

impl Refusing for Variant<'_> {
    fn refuse(&self, problem: Problem) -> Fault {
        Fault::Corporate(
            Locus {
                path: vec![],
                extent: self.at.extent,
            },
            problem,
        )
    }
}

impl Chaining for Datom {
    fn chain(&self, out: &mut String) -> bool {
        let mut here = self;
        loop {
            match here {
                Datom::Word(word) => {
                    out.push_str(word.as_ref());
                    return true;
                }
                Datom::Variant(head, body) => {
                    out.push_str(head.as_ref());
                    out.push(Separator::Period.glyph());
                    here = body;
                }
                _ => return false,
            }
        }
    }
}

impl<'a> Sited<'a> for Site<'a> {
    fn positions(self, arity: Integer) -> Result<Positions<'a>, Fault> {
        match self.datom {
            Datom::Struct(datoms) if datoms.len() as Integer == arity => Ok(Positions {
                datoms,
                at: self.at,
                index: 0,
                body: false,
                budget: self.budget,
            }),
            Datom::Struct(datoms) => {
                Err(self.refuse(Problem::Arity(arity, datoms.len() as Integer)))
            }
            _ => Err(self.refuse(Problem::Shape(Expected::Struct, self.found()))),
        }
    }

    fn elements(self) -> Result<Positions<'a>, Fault> {
        match self.datom {
            Datom::Vector(datoms) => Ok(Positions {
                datoms,
                at: self.at,
                index: 0,
                body: false,
                budget: self.budget,
            }),
            _ => Err(self.refuse(Problem::Shape(Expected::Vector, self.found()))),
        }
    }

    fn variant(self) -> Result<Variant<'a>, Fault> {
        let Site { datom, at, budget } = self;
        match datom {
            Datom::Word(name) => Ok(Variant {
                name: name.as_ref(),
                body: None,
                at,
            }),
            Datom::Variant(name, body) => Ok(Variant {
                name: name.as_ref(),
                body: Some(Site {
                    datom: body,
                    at: at.part(1),
                    budget,
                }),
                at,
            }),
            _ => Err(Fault::Corporate(
                Locus {
                    path: vec![],
                    extent: at.extent,
                },
                Problem::Shape(Expected::Variant, datom.found()),
            )),
        }
    }

    fn word(&self, expected: Expected) -> Result<Cow<'a, str>, Fault> {
        if let Datom::Word(word) = self.datom {
            return Ok(Cow::Borrowed(word.as_ref()));
        }
        let mut joined = String::new();
        if self.datom.chain(&mut joined) {
            Ok(Cow::Owned(joined))
        } else {
            Err(self.refuse(Problem::Shape(expected, self.found())))
        }
    }

    fn text(&self) -> Result<Text, Fault> {
        if let Datom::Text(text) = self.datom {
            return Ok(text.clone());
        }
        let mut joined = String::new();
        if !self.datom.chain(&mut joined) {
            return Err(self.refuse(Problem::Shape(Expected::Text, self.found())));
        }
        match Text::try_from(joined) {
            Ok(text) => Ok(text),
            Err(refusal) => Err(self.refuse(Problem::Value(protos::Opaque::from(
                refusal.glyph.to_string(),
            )))),
        }
    }

    fn found(&self) -> Found {
        self.datom.found()
    }

    fn refuse(&self, problem: Problem) -> Fault {
        Fault::Corporate(
            Locus {
                path: vec![],
                extent: self.at.extent,
            },
            problem,
        )
    }
}

pub(crate) trait Incorporating<T: Datomic> {
    fn corporate(self) -> Result<T, Fault>;
}

impl<T: Datomic> Incorporating<T> for Site<'_> {
    fn corporate(self) -> Result<T, Fault> {
        if !self.budget.consume() {
            return Err(self.refuse(Problem::BudgetExhausted));
        }
        T::incorporate(self)
    }
}

impl<T: Datomic> Positional<T> for Positions<'_> {
    fn position(&mut self) -> Result<T, Fault> {
        let index = self.index;
        if index == self.datoms.len() {
            return Err(Fault::Corporate(
                Locus {
                    path: vec![],
                    extent: self.at.extent,
                },
                Problem::Exhausted,
            ));
        }
        self.index += 1;
        let site = Site {
            datom: &self.datoms[index],
            at: self.at.part(index as Integer),
            budget: &mut *self.budget,
        };
        match site.corporate() {
            Ok(value) => Ok(value),
            Err(fault) => {
                let fault = fault.within(index as Integer);
                Err(if self.body { fault.within(1) } else { fault })
            }
        }
    }
}

impl Counted for Positions<'_> {
    fn remaining(&self) -> Integer {
        self.datoms.len().saturating_sub(self.index) as Integer
    }
}

impl<T: Datomic> Carrying<T> for Variant<'_> {
    fn body(self) -> Result<T, Fault> {
        match self.body {
            Some(body) => match body.corporate() {
                Ok(value) => Ok(value),
                Err(fault) => Err(fault.within(1)),
            },
            None => Err(self.refuse(Problem::Shape(Expected::Variant, Found::Word))),
        }
    }
}

impl<'a> Headed<'a> for Variant<'a> {
    fn positions(self, arity: Integer) -> Result<Positions<'a>, Fault> {
        let body = match self.body {
            Some(body) => body,
            None => {
                return Err(self.refuse(Problem::Shape(Expected::Variant, Found::Word)));
            }
        };
        match body.positions(arity) {
            Ok(mut positions) => {
                positions.body = true;
                Ok(positions)
            }
            Err(fault) => Err(fault.within(1)),
        }
    }

    fn nothing(self) -> Result<Self, Fault> {
        match self.body {
            None => Ok(self),
            Some(_) => Err(self.refuse(Problem::Shape(Expected::Word, Found::Variant))),
        }
    }

    fn reject(&self, problem: Problem) -> Fault {
        self.refuse(problem)
    }
}

/// The descent's last step: the datom at its situation becomes the corporate value.
impl<T: Datomic> Incorporable<T> for Datom {
    type Fault = Fault;
    type Budget = IncorporationBudget;

    fn incorporate(&self, at: &Situation, mut budget: Self::Budget) -> Result<T, Fault> {
        Site {
            datom: self,
            at,
            budget: &mut budget,
        }
        .corporate()
    }
}
