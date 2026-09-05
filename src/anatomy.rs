//! The types of the dialect: the concept, the meaning, and the faults.

use protos::{Extent, Integer, Opaque, Path, Symbol, Text, Word};

/// The datom concept: what a protoform means in the dialect, before a type is known.
pub enum Datom {
    /// A head, the dot, and a body: a variant carrying data.
    Variant(Symbol, Box<Datom>),
    /// Positional fields between braces.
    Struct(Vec<Datom>),
    /// Elements between brackets.
    Vector(Vec<Datom>),
    /// Quoted text.
    Text(Text),
    /// Parenthesized meaning.
    Meaning(Opaque),
    /// A bare word: the position decides what it is.
    Word(Word),
}

/// A structured string; today a plain text, its structure still to be designed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Meaning {
    /// Plain text.
    Plain(Opaque),
}

/// What a position expected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Expected {
    /// A variant carrying data.
    Variant,
    /// A struct.
    Struct,
    /// A vector.
    Vector,
    /// Text, bare or quoted.
    Text,
    /// A meaning.
    Meaning,
    /// An integer.
    Integer,
    /// A decimal.
    Decimal,
    /// A boolean.
    Boolean,
    /// A bare word: a variant carrying nothing.
    Word,
}

/// What a position found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Found {
    /// A variant carrying data.
    Variant,
    /// A struct.
    Struct,
    /// A vector.
    Vector,
    /// Quoted text.
    Text,
    /// A meaning.
    Meaning,
    /// A bare word.
    Word,
    /// An angled enclosure: no datom form.
    Angled,
    /// A qualified head: no datom form.
    Qualified,
    /// A headed structure with a separator other than the dot whose chain is not all words: no datom form.
    Chain,
}

/// What can go wrong between the protoform and the corporate value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// The position expected one form and found another.
    Shape(Expected, Found),
    /// The struct has the wrong number of positions: expected, found.
    Arity(Integer, Integer),
    /// The variant name is not one of the type's.
    UnknownVariant(Word),
    /// The word is not a value of the scalar.
    Value(Opaque),
    /// A structure with no datom form.
    Formless(Found),
    /// The text holds this many top-level structures, not one.
    OneValue(Integer),
    /// A positional reader was asked for an absent position.
    Exhausted,
    /// The caller's incorporation allowance is exhausted.
    BudgetExhausted,
}

/// Where a fault is: its path from the root datom, and its extent in the text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locus {
    /// The path from the root datom.
    pub path: Path,
    /// The span in the text.
    pub extent: Extent,
}

/// A fault of the descent, at the layer that raised it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The text has no structure.
    Structural(protos::Fault),
    /// The structure has no datom form.
    Conceptual(Locus, Problem),
    /// The datom is not a value of the type.
    Corporate(Locus, Problem),
}

/// Text that may become a `T` through the datom concept.
pub type Potential<T> = protos::Potential<T, Datom>;

/// Caller-owned allowance for library-mediated corporate incorporation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IncorporationBudget {
    remaining: Integer,
}

impl TryFrom<Integer> for IncorporationBudget {
    type Error = ();

    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        (value >= 0).then_some(Self { remaining: value }).ok_or(())
    }
}

pub(crate) trait Budgeted {
    fn consume(&mut self) -> bool;
}

impl Budgeted for IncorporationBudget {
    fn consume(&mut self) -> bool {
        if self.remaining == 0 {
            false
        } else {
            self.remaining -= 1;
            true
        }
    }
}
