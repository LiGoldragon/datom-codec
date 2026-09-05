//! The vision's example types, borne Datomic the way generated code bears it.
#![allow(dead_code)]

use datomic::{
    Carrying, Datom, Datomic, Fault, Headed, Meaning, Positional, Problem, Site, Sited, Text,
};

pub fn text(s: &str) -> Text {
    Text::try_from(s).unwrap()
}
pub fn meaning(s: &str) -> Meaning {
    Meaning::Plain(text(s))
}
fn variant(name: &str, body: Datom) -> Datom {
    Datom::Variant(name.to_owned(), Box::new(body))
}
fn word(name: &str) -> Datom {
    Datom::Word(name.to_owned())
}

#[derive(Debug, PartialEq)]
pub struct Address(pub Text, pub Text, pub Text);

impl Datomic for Address {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(3)?;
        Ok(Address(p.position()?, p.position()?, p.position()?))
    }
    fn conceive(&self) -> Datom {
        Datom::Struct(vec![
            self.0.conceive(),
            self.1.conceive(),
            self.2.conceive(),
        ])
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
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }
    fn conceive(&self) -> Datom {
        match self {
            Role::Author => word("Author"),
            Role::Reviewer(year, count) => variant(
                "Reviewer",
                Datom::Struct(vec![year.conceive(), count.conceive()]),
            ),
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
    fn conceive(&self) -> Datom {
        Datom::Struct(vec![
            self.0.conceive(),
            self.1.conceive(),
            self.2.conceive(),
            self.3.conceive(),
        ])
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
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }
    fn conceive(&self) -> Datom {
        match self {
            Reply::Accepted(id, at) => variant(
                "Accepted",
                Datom::Struct(vec![id.conceive(), at.conceive()]),
            ),
            Reply::Refused(reason, code) => variant(
                "Refused",
                Datom::Struct(vec![reason.conceive(), code.conceive()]),
            ),
            Reply::Pending => word("Pending"),
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
    fn conceive(&self) -> Datom {
        Datom::Struct(vec![self.0.conceive(), self.1.conceive()])
    }
}

#[derive(Debug, PartialEq)]
pub struct Note(pub Text, pub Meaning);

impl Datomic for Note {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(Note(p.position()?, p.position()?))
    }
    fn conceive(&self) -> Datom {
        Datom::Struct(vec![self.0.conceive(), self.1.conceive()])
    }
}

#[derive(Debug, PartialEq)]
pub struct Remark(pub Text, pub Text);

impl Datomic for Remark {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(Remark(p.position()?, p.position()?))
    }
    fn conceive(&self) -> Datom {
        Datom::Struct(vec![self.0.conceive(), self.1.conceive()])
    }
}

#[derive(Debug, PartialEq)]
pub struct Standup(pub Text, pub Vec<Meaning>);

impl Datomic for Standup {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let mut p = site.positions(2)?;
        Ok(Standup(p.position()?, p.position()?))
    }
    fn conceive(&self) -> Datom {
        Datom::Struct(vec![self.0.conceive(), self.1.conceive()])
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
    fn conceive(&self) -> Datom {
        Datom::Struct(vec![
            self.0.conceive(),
            self.1.conceive(),
            self.2.conceive(),
            self.3.conceive(),
        ])
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
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }
    fn conceive(&self) -> Datom {
        match self {
            Request::Lock(request) => variant("Lock", request.conceive()),
            Request::Release(id) => variant("Release", id.conceive()),
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
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }
    fn conceive(&self) -> Datom {
        match self {
            Observation::Locks(locks) => variant("Locks", locks.conceive()),
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
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }
    fn conceive(&self) -> Datom {
        match self {
            Response::Observed(o) => variant("Observed", o.conceive()),
            Response::Success => word("Success"),
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
    fn conceive(&self) -> Datom {
        Datom::Struct(vec![self.0.conceive()])
    }
}
