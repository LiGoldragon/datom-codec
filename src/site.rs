//! A site: a datom at its situation, and the forms a corporate reader takes it as.

use std::borrow::Cow;

use protos::{Glyphing, Incorporable, Integer, Locating, Pathed, Separator, Situation, Text};

use crate::anatomy::{Datom, Expected, Fault, Found, Locus, Problem};
use crate::kinds::{Carrying, Datomic, Positional, Sited};

/// A datom at its situation: what a corporate reader is handed.
#[derive(Clone, Copy, Debug)]
pub struct Site<'a> {
    /// The datom.
    pub datom: &'a Datom,
    /// Where it is.
    pub at: &'a Situation,
}

/// The positions of a struct or the elements of a vector, read in turn.
#[derive(Debug)]
pub struct Positions<'a> {
    datoms: &'a [Datom],
    at: &'a Situation,
    index: usize,
    body: bool,
}

/// A variant: its name, and its body if it carries one.
#[derive(Clone, Copy, Debug)]
pub struct Variant<'a> {
    /// The variant's name.
    pub name: &'a str,
    body: Option<Site<'a>>,
    site: Site<'a>,
}

/// The kind whose capability rejoins a chain of words with the dot.
trait Chaining {
    fn chain(&self, out: &mut String) -> bool;
}

impl Chaining for Datom {
    fn chain(&self, out: &mut String) -> bool {
        let mut here = self;
        loop {
            match here {
                Datom::Word(word) => {
                    out.push_str(word);
                    return true;
                }
                Datom::Variant(head, body) => {
                    out.push_str(head);
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
            }),
            _ => Err(self.refuse(Problem::Shape(Expected::Vector, self.found()))),
        }
    }

    fn variant(self) -> Result<Variant<'a>, Fault> {
        match self.datom {
            Datom::Word(name) => Ok(Variant {
                name,
                body: None,
                site: self,
            }),
            Datom::Variant(name, body) => Ok(Variant {
                name,
                body: Some(Site {
                    datom: body,
                    at: self.at.part(1),
                }),
                site: self,
            }),
            _ => Err(self.refuse(Problem::Shape(Expected::Variant, self.found()))),
        }
    }

    fn word(self, expected: Expected) -> Result<Cow<'a, str>, Fault> {
        if let Datom::Word(word) = self.datom {
            return Ok(Cow::Borrowed(word));
        }
        let mut joined = String::new();
        if self.datom.chain(&mut joined) {
            Ok(Cow::Owned(joined))
        } else {
            Err(self.refuse(Problem::Shape(expected, self.found())))
        }
    }

    fn text(self) -> Result<Text, Fault> {
        if let Datom::Text(text) = self.datom {
            return Ok(text.clone());
        }
        let mut joined = String::new();
        if !self.datom.chain(&mut joined) {
            return Err(self.refuse(Problem::Shape(Expected::Text, self.found())));
        }
        match Text::try_from(joined) {
            Ok(text) => Ok(text),
            Err(refusal) => Err(self.refuse(Problem::Value(refusal.glyph.to_string()))),
        }
    }

    fn found(self) -> Found {
        match self.datom {
            Datom::Variant(..) => Found::Variant,
            Datom::Struct(_) => Found::Struct,
            Datom::Vector(_) => Found::Vector,
            Datom::Text(_) => Found::Text,
            Datom::Meaning(_) => Found::Meaning,
            Datom::Word(_) => Found::Word,
        }
    }

    fn refuse(self, problem: Problem) -> Fault {
        Fault::Corporate(
            Locus {
                path: vec![],
                extent: self.at.extent,
            },
            problem,
        )
    }
}

impl Positional for Positions<'_> {
    fn position<T: Datomic>(&mut self) -> Result<T, Fault> {
        let index = self.index;
        self.index += 1;
        let site = Site {
            datom: &self.datoms[index],
            at: self.at.part(index as Integer),
        };
        match T::incorporate(site) {
            Ok(value) => Ok(value),
            Err(fault) => {
                let fault = fault.within(index as Integer);
                Err(if self.body { fault.within(1) } else { fault })
            }
        }
    }

    fn remaining(&self) -> Integer {
        (self.datoms.len() - self.index) as Integer
    }
}

impl<'a> Carrying<'a> for Variant<'a> {
    fn body<T: Datomic>(self) -> Result<T, Fault> {
        match self.body {
            Some(body) => match T::incorporate(body) {
                Ok(value) => Ok(value),
                Err(fault) => Err(fault.within(1)),
            },
            None => Err(self
                .site
                .refuse(Problem::Shape(Expected::Variant, Found::Word))),
        }
    }

    fn positions(self, arity: Integer) -> Result<Positions<'a>, Fault> {
        let body = match self.body {
            Some(body) => body,
            None => {
                return Err(self
                    .site
                    .refuse(Problem::Shape(Expected::Variant, Found::Word)));
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

    fn nothing(self) -> Result<(), Fault> {
        match self.body {
            None => Ok(()),
            Some(_) => Err(self
                .site
                .refuse(Problem::Shape(Expected::Word, Found::Variant))),
        }
    }
}

/// The descent's last step: the datom at its situation becomes the corporate value.
impl<T: Datomic> Incorporable<T> for Datom {
    type Fault = Fault;

    fn incorporate(&self, at: &Situation) -> Result<T, Fault> {
        T::incorporate(Site { datom: self, at })
    }
}
