//! Text, meaning and the containers: each bears Datomic by the position rule.

use protos::{Classifying, Glyph, Text};

use crate::anatomy::{Datom, Expected, Fault, Meaning, Problem};
use crate::kinds::{Carrying, Counted, Datomic, Headed, Positional, Sited};
use crate::site::Site;

/// The kind whose capability says whether text writes bare: a non-empty run of plain and separator glyphs.
trait Bare {
    fn is_bare(&self) -> bool;
}

impl Bare for str {
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

    fn conceive(&self) -> Datom {
        if self.is_bare() {
            Datom::Word(self.to_string())
        } else {
            Datom::Text(self.clone())
        }
    }
}

impl Datomic for Meaning {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        match site.datom {
            Datom::Meaning(text) => Ok(Meaning::Plain(text.clone())),
            _ => Err(site.refuse(Problem::Shape(Expected::Meaning, site.found()))),
        }
    }

    fn conceive(&self) -> Datom {
        match self {
            Meaning::Plain(text) => Datom::Meaning(text.clone()),
        }
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

    fn conceive(&self) -> Datom {
        Datom::Vector(self.iter().map(T::conceive).collect())
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
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }

    fn conceive(&self) -> Datom {
        match self {
            Some(value) => Datom::Variant("Some".to_owned(), Box::new(value.conceive())),
            None => Datom::Word("None".to_owned()),
        }
    }
}

impl<T: Datomic, E: Datomic> Datomic for Result<T, E> {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let variant = site.variant()?;
        match variant.name {
            "Ok" => Ok(Ok(variant.body()?)),
            "Err" => Ok(Err(variant.body()?)),
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }

    fn conceive(&self) -> Datom {
        match self {
            Ok(value) => Datom::Variant("Ok".to_owned(), Box::new(value.conceive())),
            Err(error) => Datom::Variant("Err".to_owned(), Box::new(error.conceive())),
        }
    }
}

impl<T: Datomic> Datomic for Box<T> {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        T::incorporate(site).map(Box::new)
    }

    fn conceive(&self) -> Datom {
        self.as_ref().conceive()
    }
}
