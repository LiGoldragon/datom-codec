//! The kinds of the dialect: what a corporate type bears, and what a reader is handed.

use std::{borrow::Cow, convert::Infallible};

use protos::{Conceivable, Integer, Opaque, Text, Textualizable, Word};

use crate::anatomy::{Datom, Expected, Fault, Found, Problem};
use crate::site::{Positions, Site, Variant};

/// The kind every corporate type of the dialect bears.
pub trait Datomic: Sized + Conceivable<Datom, Fault = Infallible> + Textualizable<Datom> {
    /// The value, from a datom at its situation.
    fn incorporate(site: Site<'_>) -> Result<Self, Fault>;
}

/// The kind a scalar bears: read from one bare word, written to one.
///
/// Its Datomic interaction is the two provided capabilities, the same for
/// every worded type; each type's own `Datomic` impl delegates to them, since a
/// blanket over `Worded` would collide with `Box<T>`, which Rust lets a
/// downstream crate declare worded.
pub trait Worded: Sized {
    /// What a position holding it expects.
    const EXPECTED: Expected;
    /// The value the word denotes, if it denotes one.
    fn from_word(word: &str) -> Option<Self>;
    /// The word denoting the value.
    fn to_word(&self) -> String;
    /// The value, from the bare word at the site.
    fn incorporate_word(site: Site<'_>) -> Result<Self, Fault> {
        let word = site.word(Self::EXPECTED)?;
        match Self::from_word(&word) {
            Some(value) => Ok(value),
            None => Err(site.refuse(Problem::Value(Opaque::from(word.into_owned())))),
        }
    }
    /// The datom of the value: its word.
    fn conceive_word(&self) -> Datom {
        Datom::Word(Word::try_from(self.to_word()).expect("a scalar word is a word run"))
    }
}

/// The kind a site bears: a datom at its situation, read as one form.
pub trait Sited<'a> {
    /// The positions of a struct of exactly this arity.
    fn positions(self, arity: Integer) -> Result<Positions<'a>, Fault>;
    /// The elements of a vector.
    fn elements(self) -> Result<Positions<'a>, Fault>;
    /// The variant: a bare word or a head with its body.
    fn variant(self) -> Result<Variant<'a>, Fault>;
    /// The bare word, or a chain of bare words rejoined, in a position expecting the scalar.
    fn word(&self, expected: Expected) -> Result<Cow<'a, str>, Fault>;
    /// The text: a word, quoted text, or a chain of words rejoined.
    fn text(&self) -> Result<Text, Fault>;
    /// The form found here.
    fn found(&self) -> Found;
    /// A corporate fault here.
    fn refuse(&self, problem: Problem) -> Fault;
}

/// The kind positions bear: each read in turn as the type `T` its position declares.
pub trait Positional<T: Datomic> {
    /// The next position, as its type.
    fn position(&mut self) -> Result<T, Fault>;
}

/// The kind positions bear: how many remain.
pub trait Counted {
    /// How many positions remain.
    fn remaining(&self) -> Integer;
}

/// The kind a variant bears: its body, read as the type `T` the variant declares.
pub trait Carrying<T: Datomic> {
    /// The body, as its type.
    fn body(self) -> Result<T, Fault>;
}

/// The kind a variant bears: what its head announces about its body.
pub trait Headed<'a>: Sized {
    /// The body's positions, as an inline struct of this arity.
    fn positions(self, arity: Integer) -> Result<Positions<'a>, Fault>;
    /// Nothing: the variant itself, which must be bare.
    fn nothing(self) -> Result<Self, Fault>;
    /// A corporate fault at this variant.
    fn reject(&self, problem: Problem) -> Fault;
}
