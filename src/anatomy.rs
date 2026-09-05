//! The types of the dialect: the concept, the meaning, and the faults.

use protos::{BareRefusal, Classifying, Extent, Glyph, Integer, Opaque, Path, Symbol, Text, Word};

/// A datom word that has one canonical concept.
///
/// A word whose root structural separator is a period is a variant in Datom,
/// not a word. Keeping that distinction here prevents public construction of
/// two concepts with the same canonical text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatomWord(Word);

/// Why a Protos word cannot occupy the Datom word position directly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WordRefusal {
    /// The source did not form a Protos word run.
    Bare(BareRefusal),
    /// Its root delineation is a period-headed variant.
    Period(Word),
    /// Its separator run cannot remain a word below a variant head.
    Unstable(Word),
}

/// The named behavior that examines a word's root structural separator.
trait Rooting {
    fn word_shape(&self) -> WordShape;
}

enum WordShape {
    Plain,
    Chain(protos::Separator),
    Malformed,
}

impl Rooting for str {
    fn word_shape(&self) -> WordShape {
        let mut first = None;
        let mut separated = false;
        let mut before = false;
        for glyph in self.chars() {
            if let Glyph::Separate(separator) = glyph.classify() {
                if !before || separated {
                    return WordShape::Malformed;
                }
                first.get_or_insert(separator);
                separated = true;
            } else {
                before = true;
                separated = false;
            }
        }
        if separated {
            WordShape::Malformed
        } else {
            match first {
                Some(separator) => WordShape::Chain(separator),
                None => WordShape::Plain,
            }
        }
    }
}

impl TryFrom<Word> for DatomWord {
    type Error = WordRefusal;

    fn try_from(word: Word) -> Result<Self, Self::Error> {
        match word.as_ref().word_shape() {
            WordShape::Chain(protos::Separator::Period) => Err(WordRefusal::Period(word)),
            WordShape::Malformed => Err(WordRefusal::Unstable(word)),
            WordShape::Plain | WordShape::Chain(_) => Ok(Self(word)),
        }
    }
}

impl TryFrom<&str> for DatomWord {
    type Error = WordRefusal;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Word::try_from(text)
            .map_err(WordRefusal::Bare)
            .and_then(Self::try_from)
    }
}

impl AsRef<str> for DatomWord {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// The named ascent from one Protos word to its canonical Datom anatomy.
pub(crate) trait WordProjecting {
    fn project_word(self) -> Datom;
}

impl WordProjecting for Word {
    fn project_word(self) -> Datom {
        match DatomWord::try_from(self) {
            Ok(word) => Datom::Word(word),
            Err(WordRefusal::Period(word)) => {
                let text = word.as_ref();
                let (offset, _) = text
                    .char_indices()
                    .find(|(_, glyph)| {
                        matches!(glyph.classify(), Glyph::Separate(protos::Separator::Period))
                    })
                    .expect("a period-root word contains a period");
                let head = Symbol::try_from(&text[..offset])
                    .expect("a period-root word has a symbol head");
                let tail = Word::try_from(&text[offset + 1..])
                    .expect("a period-root word has a word body");
                Datom::Variant(head, Box::new(tail.project_word()))
            }
            Err(WordRefusal::Unstable(word)) => {
                Datom::Text(Text::try_from(word.as_ref()).expect("a Protos word remains text"))
            }
            Err(WordRefusal::Bare(_)) => unreachable!("a Word was already validated"),
        }
    }
}

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
    Word(DatomWord),
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
