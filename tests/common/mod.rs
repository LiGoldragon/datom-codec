//! The vision's example types, borne Datomic the way generated code bears it.
#![allow(dead_code)]

use datom_codec::{
    Carrying, Conceivable, Datom, Datomic, Fault, Headed, Meaning, Opaque, Positional, Problem,
    Site, Sited, Situated, Situation, Text, Word,
};
use std::convert::Infallible;

pub fn text(s: &str) -> Text {
    Text::try_from(s).unwrap()
}
pub fn meaning(s: &str) -> Meaning {
    Meaning::Plain(Opaque::from(s))
}
pub fn budget() -> datom_codec::IncorporationBudget {
    datom_codec::IncorporationBudget::try_from(1_000_000).unwrap()
}
fn variant(name: &str, body: Datom) -> Datom {
    Datom::Variant(protos::Symbol::try_from(name).unwrap(), Box::new(body))
}
fn word(name: &str) -> Datom {
    Datom::Word(Word::try_from(name).unwrap())
}
fn datum<T: Conceivable<Datom, Fault = Infallible>>(value: &T) -> Datom {
    match value.conceive() {
        Ok(Situated(_, datom)) => datom,
        Err(never) => match never {},
    }
}
macro_rules! projection {
    ($type:ty, $value:ident => $datom:expr) => {
        impl Conceivable<Datom> for $type {
            type Fault = Infallible;

            fn conceive(&self) -> Result<Situated<Datom>, Self::Fault> {
                let $value = self;
                Ok(Situated(
                    Situation {
                        extent: datom_codec::Extent(0, 0),
                        children: vec![],
                    },
                    $datom,
                ))
            }
        }
    };
}

#[derive(Debug, PartialEq)]
pub struct Address(pub Text, pub Text, pub Text);

impl Datomic for Address {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(3)?;
        Ok(Address(p.position()?, p.position()?, p.position()?))
    }
}

#[derive(Debug, PartialEq)]
pub enum Role {
    Author,
    Reviewer(i64, i64),
}

impl Datomic for Role {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let v = site.variant()?;
        match v.name {
            "Author" => {
                v.nothing()?;
                Ok(Role::Author)
            }
            "Reviewer" => {
                let mut p = v.positions(2)?;
                Ok(Role::Reviewer(p.position()?, p.position()?))
            }
            other => Err(v.reject(Problem::UnknownVariant(Word::try_from(other).unwrap()))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Person(pub Text, pub i64, pub Address, pub Vec<Role>);

impl Datomic for Person {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(4)?;
        Ok(Person(
            p.position()?,
            p.position()?,
            p.position()?,
            p.position()?,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub enum Reply {
    Accepted(i64, Text),
    Refused(Text, i64),
    Pending,
}

impl Datomic for Reply {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let v = site.variant()?;
        match v.name {
            "Accepted" => {
                let mut p = v.positions(2)?;
                Ok(Reply::Accepted(p.position()?, p.position()?))
            }
            "Refused" => {
                let mut p = v.positions(2)?;
                Ok(Reply::Refused(p.position()?, p.position()?))
            }
            "Pending" => {
                v.nothing()?;
                Ok(Reply::Pending)
            }
            other => Err(v.reject(Problem::UnknownVariant(Word::try_from(other).unwrap()))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Scores(pub Text, pub Vec<i64>);

impl Datomic for Scores {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(Scores(p.position()?, p.position()?))
    }
}

#[derive(Debug, PartialEq)]
pub struct Note(pub Text, pub Meaning);

impl Datomic for Note {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(Note(p.position()?, p.position()?))
    }
}

#[derive(Debug, PartialEq)]
pub struct Remark(pub Text, pub Text);

impl Datomic for Remark {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(Remark(p.position()?, p.position()?))
    }
}

#[derive(Debug, PartialEq)]
pub struct Standup(pub Text, pub Vec<Meaning>);

impl Datomic for Standup {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(Standup(p.position()?, p.position()?))
    }
}

#[derive(Debug, PartialEq)]
pub struct LockRequest(pub Text, pub Text, pub Vec<Text>, pub Text);

impl Datomic for LockRequest {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(4)?;
        Ok(LockRequest(
            p.position()?,
            p.position()?,
            p.position()?,
            p.position()?,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub enum Request {
    Lock(LockRequest),
    Release(i64),
}

impl Datomic for Request {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let v = site.variant()?;
        match v.name {
            "Lock" => Ok(Request::Lock(v.body()?)),
            "Release" => Ok(Request::Release(v.body()?)),
            other => Err(v.reject(Problem::UnknownVariant(Word::try_from(other).unwrap()))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Observation {
    Locks(Vec<LockRequest>),
}

impl Datomic for Observation {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let v = site.variant()?;
        match v.name {
            "Locks" => Ok(Observation::Locks(v.body()?)),
            other => Err(v.reject(Problem::UnknownVariant(Word::try_from(other).unwrap()))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Response {
    Observed(Observation),
    Success,
}

impl Datomic for Response {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let v = site.variant()?;
        match v.name {
            "Observed" => Ok(Response::Observed(v.body()?)),
            "Success" => {
                v.nothing()?;
                Ok(Response::Success)
            }
            other => Err(v.reject(Problem::UnknownVariant(Word::try_from(other).unwrap()))),
        }
    }
}

/// Struct, Vector, Option, Result, Box and Variant, nested.
#[derive(Debug, PartialEq)]
pub struct Deep(pub Vec<Option<Result<Box<Role>, Text>>>);

impl Datomic for Deep {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(1)?;
        Ok(Deep(p.position()?))
    }
}

projection!(Address, value => Datom::Struct(vec![datum(&value.0), datum(&value.1), datum(&value.2)]));
projection!(Role, value => match value {
    Role::Author => word("Author"),
    Role::Reviewer(year, count) => variant("Reviewer", Datom::Struct(vec![datum(year), datum(count)])),
});
projection!(Person, value => Datom::Struct(vec![datum(&value.0), datum(&value.1), datum(&value.2), datum(&value.3)]));
projection!(Reply, value => match value {
    Reply::Accepted(id, at) => variant("Accepted", Datom::Struct(vec![datum(id), datum(at)])),
    Reply::Refused(reason, code) => variant("Refused", Datom::Struct(vec![datum(reason), datum(code)])),
    Reply::Pending => word("Pending"),
});
projection!(Scores, value => Datom::Struct(vec![datum(&value.0), datum(&value.1)]));
projection!(Note, value => Datom::Struct(vec![datum(&value.0), datum(&value.1)]));
projection!(Remark, value => Datom::Struct(vec![datum(&value.0), datum(&value.1)]));
projection!(Standup, value => Datom::Struct(vec![datum(&value.0), datum(&value.1)]));
projection!(LockRequest, value => Datom::Struct(vec![datum(&value.0), datum(&value.1), datum(&value.2), datum(&value.3)]));
projection!(Request, value => match value {
    Request::Lock(request) => variant("Lock", datum(request)),
    Request::Release(id) => variant("Release", datum(id)),
});
projection!(Observation, value => match value {
    Observation::Locks(locks) => variant("Locks", datum(locks)),
});
projection!(Response, value => match value {
    Response::Observed(observation) => variant("Observed", datum(observation)),
    Response::Success => word("Success"),
});
projection!(Deep, value => Datom::Struct(vec![datum(&value.0)]));
