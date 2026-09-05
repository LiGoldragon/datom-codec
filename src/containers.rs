//! Text, meaning and the containers: each bears Datomic by the position rule.

use std::convert::Infallible;

use protos::{Classifying, Conceivable, Glyph, Situated, Situation, Symbol, Text, Word};

use crate::anatomy::{Datom, Expected, Fault, Meaning, Problem, WordProjecting};
use crate::kinds::{Carrying, Counted, Datomic, Headed, Positional, Sited};
use crate::site::{Incorporating, Site};

/// The kind whose capability says whether text writes bare: a non-empty run of plain and separator glyphs.
trait BareText {
    fn is_bare(&self) -> bool;
}

/// The kind whose capability makes a bare payload unambiguous after a variant head.
trait Carryable {
    fn carried(self) -> Self;
}

impl Carryable for Datom {
    fn carried(self) -> Self {
        if let Self::Word(word) = &self {
            if word.as_ref().needs_quotes_as_a_payload() {
                return Self::Text(
                    Text::try_from(word.as_ref())
                        .expect("a bare word cannot contain a closing quote"),
                );
            }
        }
        self
    }
}

/// The kind whose capability says whether a bare word would blur a preceding variant boundary.
trait Payload {
    fn needs_quotes_as_a_payload(&self) -> bool;
}

impl Payload for str {
    fn needs_quotes_as_a_payload(&self) -> bool {
        let mut previous_separator = false;
        for glyph in self.chars() {
            let separator = matches!(glyph.classify(), Glyph::Separate(_));
            if separator && previous_separator {
                return true;
            }
            previous_separator = separator;
        }
        self.chars()
            .next()
            .is_some_and(|glyph| matches!(glyph.classify(), Glyph::Separate(_)))
            || previous_separator
    }
}

impl BareText for str {
    fn is_bare(&self) -> bool {
        for glyph in self.chars() {
            if !matches!(glyph.classify(), Glyph::Plain | Glyph::Separate(_)) {
                return false;
            }
        }
        !self.is_empty()
    }
}

impl Datomic for Text {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        site.text()
    }
}

impl Conceivable<Datom> for Text {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        let datom = if self.is_bare() {
            Word::try_from(self.as_ref())
                .expect("bare text is a word run")
                .project_word()
        } else {
            Datom::Text(self.clone())
        };
        Ok(Situated(
            Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            datom,
        ))
    }
}

impl Datomic for Meaning {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        match site.datom {
            Datom::Meaning(text) => Ok(Meaning::Plain(text.clone())),
            _ => Err(site.refuse(Problem::Shape(Expected::Meaning, site.found()))),
        }
    }
}

impl Conceivable<Datom> for Meaning {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        let Meaning::Plain(text) = self;
        Ok(Situated(
            Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            Datom::Meaning(text.clone()),
        ))
    }
}

impl<T: Datomic> Datomic for Vec<T> {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut elements = site.elements()?;
        let mut values = Vec::with_capacity(elements.remaining() as usize);
        while elements.remaining() > 0 {
            values.push(elements.position()?);
        }
        Ok(values)
    }
}

impl<T: Datomic> Conceivable<Datom> for Vec<T> {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        let values = self
            .iter()
            .map(|value| value.conceive().map(|situated| situated.1))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Situated(
            Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            Datom::Vector(values),
        ))
    }
}

impl<T: Datomic> Datomic for Option<T> {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let variant = site.variant()?;
        match variant.name {
            "Some" => Ok(Some(variant.body()?)),
            "None" => {
                variant.nothing()?;
                Ok(None)
            }
            other => Err(variant.reject(Problem::UnknownVariant(Word::try_from(other).unwrap()))),
        }
    }
}

impl<T: Datomic> Conceivable<Datom> for Option<T> {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        let datom = match self {
            Some(value) => Datom::Variant(
                Symbol::try_from("Some").unwrap(),
                Box::new(value.conceive()?.1.carried()),
            ),
            None => Word::try_from("None").unwrap().project_word(),
        };
        Ok(Situated(
            Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            datom,
        ))
    }
}

impl<T: Datomic, E: Datomic> Datomic for Result<T, E> {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let variant = site.variant()?;
        match variant.name {
            "Ok" => Ok(Ok(variant.body()?)),
            "Err" => Ok(Err(variant.body()?)),
            other => Err(variant.reject(Problem::UnknownVariant(Word::try_from(other).unwrap()))),
        }
    }
}

impl<T: Datomic, E: Datomic> Conceivable<Datom> for Result<T, E> {
    type Fault = Infallible;

    fn conceive(&self) -> Result<protos::Situated<Datom>, Self::Fault> {
        let datom = match self {
            Ok(value) => Datom::Variant(
                Symbol::try_from("Ok").unwrap(),
                Box::new(value.conceive()?.1.carried()),
            ),
            Err(error) => Datom::Variant(
                Symbol::try_from("Err").unwrap(),
                Box::new(error.conceive()?.1.carried()),
            ),
        };
        Ok(Situated(
            Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            datom,
        ))
    }
}

impl<T: Datomic> Datomic for Box<T> {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        site.corporate().map(Box::new)
    }
}
