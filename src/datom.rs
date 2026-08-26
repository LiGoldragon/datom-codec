use std::{collections::BTreeMap, marker::PhantomData, path::PathBuf};

use protos::{
    Block, BlockScanning, CursorObserving, Head, Headed, Realize, RealizeDriving, RealizeScope,
    RealizeScoping, RealizeWalk, Shape, ShapeDefined, SourceText, StringCarrying, Textualize,
    TextualizeDriving, TextualizeScope, TextualizeScoping, TextualizeWalk, WalkFault,
    WalkObservation, WalkObserving,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatomFault {
    pub problem: DatomProblem,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DatomProblem {
    Shape,
    Head,
    Value,
    Path,
    Position,
    ExtraPosition,
    MissingPosition,
    AmbiguousMapPair,
    Protos(WalkFault),
}

impl From<WalkFault> for DatomFault {
    fn from(problem: WalkFault) -> Self {
        Self {
            problem: DatomProblem::Protos(problem),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Text(pub String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagList(pub Vec<Text>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Entry {
    Note(Text),
    Group(Group),
    Tags(TagList),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Group {
    pub title: Text,
    pub children: Vec<Entry>,
    pub annotations: BTreeMap<String, Text>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub heading: Text,
    pub groups: BTreeMap<String, Vec<Entry>>,
    pub latest: Option<Text>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterimNote {
    pub a: String,
    pub b: String,
    pub c: String,
    pub d: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportText {
    pub source: SourceText,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterimNoteText {
    pub source: SourceText,
}

/// Actual, read-only Protos transition evidence paired with a typed operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatomEvidence {
    observation: WalkObservation,
    cursor: usize,
}

/// A typed value returned together with the driver that actually produced it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Realized<T> {
    value: T,
    evidence: DatomEvidence,
}

/// Typed canonical text returned together with the driver that emitted it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Projected<T> {
    text: T,
    evidence: DatomEvidence,
}

/// Read-only access to a typed operation's actual Protos driver evidence.
pub trait EvidenceObserving {
    fn observation(&self) -> &WalkObservation;
    fn cursor(&self) -> usize;
}

/// Read-only access to a realized typed value and its evidence.
pub trait RealizationViewing<T> {
    fn value(&self) -> &T;
    fn evidence(&self) -> &DatomEvidence;
}

/// Read-only access to canonical typed text and its evidence.
pub trait ProjectionViewing<T> {
    fn text(&self) -> &T;
    fn evidence(&self) -> &DatomEvidence;
}

/// Typed realization which returns the same Protos driver's transition facts.
pub trait EvidencedRealizing {
    type Value;

    fn realize_evidenced(&self) -> Result<Realized<Self::Value>, DatomFault>;
}

/// Typed textualization which returns the same Protos driver's transition facts.
pub trait EvidencedTextualizing {
    type Text;

    fn textualize_evidenced(&self) -> Result<Projected<Self::Text>, DatomFault>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OptionalText {
    value: Option<Text>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GroupMapEntry {
    key: String,
    value: Vec<Entry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TextMapEntry {
    key: String,
    value: Text,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EntryVector {
    value: Vec<Entry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GroupMap {
    value: BTreeMap<String, Vec<Entry>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TextMap {
    value: BTreeMap<String, Text>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextSelection {
    Plain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntrySelection {
    Note,
    BareNote,
    Group,
    Tags,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupSelection {
    Payload,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TagListSelection {
    Payload,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum MapSelection {
    Entries,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum OptionalSelection {
    Bare,
    Some,
}

/// The position counter used while realizing a positional record body.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordPosition {
    ordinal: usize,
}

/// Advances a positional record counter owned by a dialect schema.
pub trait PositionAdvancing {
    fn next_position(&mut self) -> usize;
}

impl EvidenceObserving for DatomEvidence {
    fn observation(&self) -> &WalkObservation {
        &self.observation
    }

    fn cursor(&self) -> usize {
        self.cursor
    }
}

impl<T> RealizationViewing<T> for Realized<T> {
    fn value(&self) -> &T {
        &self.value
    }

    fn evidence(&self) -> &DatomEvidence {
        &self.evidence
    }
}

impl<T> ProjectionViewing<T> for Projected<T> {
    fn text(&self) -> &T {
        &self.text
    }

    fn evidence(&self) -> &DatomEvidence {
        &self.evidence
    }
}

impl PositionAdvancing for RecordPosition {
    fn next_position(&mut self) -> usize {
        let ordinal = self.ordinal;
        self.ordinal += 1;
        ordinal
    }
}

/// The Datom realization seam for one typed value in an active structural scope.
///
/// External schemas implement this on their own enum, record, and scalar wrapper
/// types. Recursive work must use the supplied scope rather than a new Protos walk.
pub trait DatomRealizing: Sized {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault>;
}

/// The Datom canonical-projection seam for one typed value in an active scope.
pub trait DatomTextualizing {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault>;
}

/// The Datom enum seam for a head and its selected payloadless unit.
///
/// Implement this on a unit-only enum, then implement `DatomRoot` to use it
/// as a document root. The type projects `Head.Unit`, such as
/// `Observe.Locks`; future units select through the same head without adding
/// a new dialect syntax form.
pub trait DatomHeadedUnit: Sized {
    fn head() -> &'static str;
    fn select_unit(unit: &str) -> Option<Self>;
    fn unit(&self) -> &'static str;
}

/// A top-level Datom value whose expected type selects its root shape.
///
/// The default operations start the only Protos walk used for a document. Child
/// records and variants stay inside the scopes supplied by `DatomRealizing` and
/// `DatomTextualizing`, so a consumer never needs a runtime-local parser.
pub trait DatomRoot: DatomRealizing + DatomTextualizing {
    fn realize_source(source: &SourceText) -> Result<Self, DatomFault> {
        let mut walk = RealizeWalk::default();
        let mut values =
            walk.realize_source(source, |scope, block| Self::realize_block(scope, block))?;
        if values.len() != 1 {
            return Err(DatomFault {
                problem: DatomProblem::Position,
            });
        }
        Ok(values.remove(0))
    }

    fn textualize_source(&self) -> Result<SourceText, DatomFault> {
        let mut walk = TextualizeWalk::default();
        walk.textualize_source(|scope| self.textualize_in(scope))?;
        Ok(walk.textual_source())
    }
}

/// The typed textual carrier for any external `DatomRoot`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatomText<T> {
    pub source: SourceText,
    real: PhantomData<fn() -> T>,
}

impl<T> From<SourceText> for DatomText<T> {
    fn from(source: SourceText) -> Self {
        Self {
            source,
            real: PhantomData,
        }
    }
}

impl<T: DatomRoot> Realize for DatomText<T> {
    type Real = T;
    type Fault = DatomFault;

    fn realize(&self) -> Result<Self::Real, Self::Fault> {
        T::realize_source(&self.source)
    }
}

impl DatomRealizing for String {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        Ok(Text::realize_block(scope, block)?.0)
    }
}

impl DatomTextualizing for String {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        Text(self.clone()).textualize_in(scope)
    }
}

trait CanonicalInteger {
    fn canonical_i64(&self) -> Option<i64>;
}

impl CanonicalInteger for str {
    fn canonical_i64(&self) -> Option<i64> {
        let valid = match self.as_bytes() {
            [b'0'] => true,
            [b'1'..=b'9', rest @ ..] => rest.iter().all(u8::is_ascii_digit),
            [b'-', b'1'..=b'9', rest @ ..] => rest.iter().all(u8::is_ascii_digit),
            _ => false,
        };
        valid.then(|| self.parse().ok()).flatten()
    }
}

impl DatomRealizing for i64 {
    fn realize_block(_: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if block.shape != Shape::Bare || block.head().is_some() {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        block.body.0.canonical_i64().ok_or(DatomFault {
            problem: DatomProblem::Value,
        })
    }
}

impl DatomTextualizing for i64 {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        scope.textualize_block(Shape::Bare, None, |body| {
            body.emit_scalar(&self.to_string());
            Ok(())
        })
    }
}

impl DatomRealizing for bool {
    fn realize_block(_: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if block.shape != Shape::Bare || block.head().is_some() {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        match block.body.0.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(DatomFault {
                problem: DatomProblem::Value,
            }),
        }
    }
}

impl DatomTextualizing for bool {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        scope.textualize_block(Shape::Bare, None, |body| {
            body.emit_scalar(if *self { "true" } else { "false" });
            Ok(())
        })
    }
}

impl DatomRealizing for PathBuf {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        Ok(Self::from(String::realize_block(scope, block)?))
    }
}

impl DatomTextualizing for PathBuf {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        let path = self.to_str().ok_or(DatomFault {
            problem: DatomProblem::Path,
        })?;
        path.to_owned().textualize_in(scope)
    }
}

impl<T: DatomHeadedUnit> DatomRealizing for T {
    fn realize_block(_: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if block.shape != Shape::DottedBare || block.head() != Some(&Head(T::head().into())) {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        T::select_unit(&block.body.0).ok_or(DatomFault {
            problem: DatomProblem::Value,
        })
    }
}

impl<T: DatomHeadedUnit> DatomTextualizing for T {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        let head = Head(T::head().into());
        scope.textualize_block(Shape::DottedBare, Some(&head), |body| {
            body.emit_scalar(T::unit(self));
            Ok(())
        })
    }
}

impl<T: DatomRealizing> DatomRealizing for Vec<T> {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if block.shape != Shape::SquareBracketed || block.head().is_some() {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        scope.realize_body(&mut |child_scope, child| T::realize_block(child_scope, child))
    }
}

impl<T: DatomTextualizing> DatomTextualizing for Vec<T> {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        scope.textualize_block(Shape::SquareBracketed, None, |body| {
            for value in self {
                value.textualize_in(body)?;
            }
            Ok(())
        })
    }
}

impl<T: DatomRealizing> DatomRealizing for BTreeMap<String, T> {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if block.shape != Shape::Guillemeted || block.head().is_some() {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        let entries = scope.realize_body(&mut |entry_scope, entry| {
            if entry.shape != Shape::DottedSquareBracketed {
                return Err(DatomFault {
                    problem: DatomProblem::Shape,
                });
            }
            let key = entry
                .head()
                .ok_or(DatomFault {
                    problem: DatomProblem::Head,
                })?
                .text();
            key.group_key()?;
            let mut values = entry_scope
                .realize_body(&mut |value_scope, value| T::realize_block(value_scope, value))?;
            match values.len() {
                0 => Err(DatomFault {
                    problem: DatomProblem::MissingPosition,
                }),
                1 => Ok((key, values.remove(0))),
                _ => Err(DatomFault {
                    problem: DatomProblem::ExtraPosition,
                }),
            }
        })?;
        let mut values = BTreeMap::new();
        for (key, value) in entries {
            if values.insert(key, value).is_some() {
                return Err(DatomFault {
                    problem: DatomProblem::Position,
                });
            }
        }
        Ok(values)
    }
}

impl<T: DatomTextualizing> DatomTextualizing for BTreeMap<String, T> {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        scope.textualize_block(Shape::Guillemeted, None, |body| {
            for (key, value) in self {
                key.group_key()?;
                let entry = Head(key.clone());
                body.textualize_block(Shape::DottedSquareBracketed, Some(&entry), |entry_scope| {
                    value.textualize_in(entry_scope)
                })?;
            }
            Ok(())
        })
    }
}

trait TagPayloading: Sized {
    fn realize_tag_payload(scope: &mut RealizeScope<'_>) -> Result<Self, DatomFault>;
    fn textualize_tag_payload(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault>;
}

trait GroupPayloading: Sized {
    fn realize_group_payload(scope: &mut RealizeScope<'_>) -> Result<Self, DatomFault>;
    fn textualize_group_payload(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault>;
}

trait BareProjecting {
    fn fits_bare(&self) -> bool;
}

impl BareProjecting for Text {
    fn fits_bare(&self) -> bool {
        let candidate = SourceText(self.0.clone());
        matches!(
            candidate.blocks(),
            Ok(blocks)
                if blocks.len() == 1
                    && matches!(blocks[0].shape, Shape::Bare | Shape::DottedBare)
                    && blocks[0]
                        .string_carrier
                        .as_ref()
                        .is_some_and(|carrier| carrier.textual_body() == self.0)
        )
    }
}

trait HeadReading {
    fn text(&self) -> String;
}

impl HeadReading for Head {
    fn text(&self) -> String {
        self.0.clone()
    }
}

trait PairDividing {
    fn divide(&self) -> Result<(String, Text), DatomFault>;
}

trait MapKeyChecking {
    fn group_key(&self) -> Result<(), DatomFault>;
    fn text_key(&self, value: &Text) -> Result<(), DatomFault>;
}

impl MapKeyChecking for String {
    fn group_key(&self) -> Result<(), DatomFault> {
        if self.is_empty() || self.contains('.') {
            return Err(DatomFault {
                problem: DatomProblem::AmbiguousMapPair,
            });
        }
        let candidate = SourceText(format!("{}.[ ]", self));
        match candidate.blocks() {
            Ok(blocks)
                if blocks.len() == 1
                    && blocks[0].shape == Shape::DottedSquareBracketed
                    && blocks[0].head() == Some(&Head(self.clone())) =>
            {
                Ok(())
            }
            _ => Err(DatomFault {
                problem: DatomProblem::AmbiguousMapPair,
            }),
        }
    }

    fn text_key(&self, value: &Text) -> Result<(), DatomFault> {
        if self.is_empty() || self.contains('.') {
            return Err(DatomFault {
                problem: DatomProblem::AmbiguousMapPair,
            });
        }
        let candidate = if value.fits_bare() {
            SourceText(format!("{}.{}", self, value.0))
        } else {
            SourceText(format!("{}.“{}”", self, value.0))
        };
        match candidate.blocks() {
            Ok(blocks)
                if blocks.len() == 1
                    && ((matches!(blocks[0].shape, Shape::Bare | Shape::DottedBare)
                        && blocks[0].divide().is_ok_and(|pair| pair.0 == *self))
                        || (blocks[0].shape == Shape::DottedCurlyQuoted
                            && blocks[0].head() == Some(&Head(self.clone())))) =>
            {
                Ok(())
            }
            _ => Err(DatomFault {
                problem: DatomProblem::AmbiguousMapPair,
            }),
        }
    }
}

impl PairDividing for Block {
    fn divide(&self) -> Result<(String, Text), DatomFault> {
        if !matches!(self.shape, Shape::Bare | Shape::DottedBare) {
            return Err(DatomFault {
                problem: DatomProblem::AmbiguousMapPair,
            });
        }
        let (key, value) = match self.shape {
            Shape::Bare => self.body.0.split_once('.').ok_or(DatomFault {
                problem: DatomProblem::AmbiguousMapPair,
            })?,
            Shape::DottedBare => (
                self.head()
                    .ok_or(DatomFault {
                        problem: DatomProblem::AmbiguousMapPair,
                    })?
                    .0
                    .as_str(),
                self.body.0.as_str(),
            ),
            _ => unreachable!(),
        };
        if key.is_empty() || value.is_empty() || key.contains('.') {
            return Err(DatomFault {
                problem: DatomProblem::AmbiguousMapPair,
            });
        }
        Ok((key.to_owned(), Text(value.to_owned())))
    }
}

impl ShapeDefined for Text {
    type Selection = TextSelection;

    fn shapes() -> &'static [Shape] {
        &[Shape::Bare, Shape::DottedBare, Shape::CurlyQuoted]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        // Plain String uses curly quotes. Parenthesis-delimited Meaning remains
        // unimplemented.
        match (shape, head) {
            (Shape::Bare | Shape::CurlyQuoted, None) | (Shape::DottedBare, Some(_)) => {
                Some(TextSelection::Plain)
            }
            _ => None,
        }
    }
}

impl ShapeDefined for Entry {
    type Selection = EntrySelection;

    fn shapes() -> &'static [Shape] {
        &[
            Shape::Bare,
            Shape::DottedBare,
            Shape::DottedCurlyQuoted,
            Shape::DottedBraced,
            Shape::DottedSquareBracketed,
        ]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        match (shape, head) {
            (Shape::Bare, None) | (Shape::DottedBare, Some(_)) => Some(EntrySelection::BareNote),
            (Shape::DottedCurlyQuoted, Some(value)) if value == &Head("Note".into()) => {
                Some(EntrySelection::Note)
            }
            (Shape::DottedBraced, Some(value)) if value == &Head("Group".into()) => {
                Some(EntrySelection::Group)
            }
            (Shape::DottedSquareBracketed, Some(value)) if value == &Head("Tags".into()) => {
                Some(EntrySelection::Tags)
            }
            _ => None,
        }
    }
}

impl ShapeDefined for Group {
    type Selection = GroupSelection;

    fn shapes() -> &'static [Shape] {
        &[Shape::Braced]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Braced && head.is_none()).then_some(GroupSelection::Payload)
    }
}

impl ShapeDefined for TagList {
    type Selection = TagListSelection;

    fn shapes() -> &'static [Shape] {
        &[Shape::SquareBracketed]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::SquareBracketed && head.is_none()).then_some(TagListSelection::Payload)
    }
}

impl ShapeDefined for EntryVector {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::SquareBracketed]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::SquareBracketed && head.is_none()).then_some(())
    }
}

impl ShapeDefined for GroupMap {
    type Selection = MapSelection;

    fn shapes() -> &'static [Shape] {
        &[Shape::Guillemeted]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Guillemeted && head.is_none()).then_some(MapSelection::Entries)
    }
}

impl ShapeDefined for TextMap {
    type Selection = MapSelection;

    fn shapes() -> &'static [Shape] {
        &[Shape::Guillemeted]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Guillemeted && head.is_none()).then_some(MapSelection::Entries)
    }
}

impl ShapeDefined for Report {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::Braced]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Braced && head.is_none()).then_some(())
    }
}

impl ShapeDefined for InterimNote {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::Braced]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Braced && head.is_none()).then_some(())
    }
}

impl ShapeDefined for OptionalText {
    type Selection = OptionalSelection;

    fn shapes() -> &'static [Shape] {
        &[Shape::Bare, Shape::DottedBare, Shape::DottedCurlyQuoted]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        match (shape, head) {
            (Shape::Bare, None) | (Shape::DottedBare, Some(_)) => Some(OptionalSelection::Bare),
            (Shape::DottedCurlyQuoted, Some(value)) if value == &Head("Some".into()) => {
                Some(OptionalSelection::Some)
            }
            _ => None,
        }
    }
}

impl DatomRealizing for Text {
    fn realize_block(_: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if Self::select(block.shape, block.head()).is_none() {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        let Some(carrier) = &block.string_carrier else {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        };
        Ok(Self(carrier.textual_body().to_owned()))
    }
}

impl DatomTextualizing for Text {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        if self.fits_bare() {
            scope.textualize_block(Shape::Bare, None, |body| {
                body.emit_scalar(&self.0);
                Ok(())
            })
        } else {
            scope.textualize_block(Shape::CurlyQuoted, None, |body| {
                body.emit_scalar(&self.0);
                Ok(())
            })
        }
    }
}

impl DatomRealizing for EntryVector {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if Self::select(block.shape, block.head()).is_none() {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        Ok(Self {
            value: scope
                .realize_body(&mut |child_scope, child| Entry::realize_block(child_scope, child))?,
        })
    }
}

impl DatomTextualizing for EntryVector {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        scope.textualize_block(Shape::SquareBracketed, None, |body| {
            for entry in &self.value {
                entry.textualize_in(body)?;
            }
            Ok(())
        })
    }
}

impl TagPayloading for TagList {
    fn realize_tag_payload(scope: &mut RealizeScope<'_>) -> Result<Self, DatomFault> {
        Ok(Self(scope.realize_body(&mut |child_scope, child| {
            Text::realize_block(child_scope, child)
        })?))
    }

    fn textualize_tag_payload(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        for text in &self.0 {
            text.textualize_in(scope)?;
        }
        Ok(())
    }
}

impl DatomRealizing for GroupMapEntry {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if block.shape != Shape::DottedSquareBracketed {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        let Some(head) = block.head() else {
            return Err(DatomFault {
                problem: DatomProblem::Head,
            });
        };
        let key = head.text();
        key.group_key()?;
        let value = scope
            .realize_body(&mut |child_scope, child| Entry::realize_block(child_scope, child))?;
        Ok(Self { key, value })
    }
}

impl DatomTextualizing for GroupMapEntry {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        self.key.group_key()?;
        let head = Head(self.key.clone());
        scope.textualize_block(Shape::DottedSquareBracketed, Some(&head), |body| {
            for entry in &self.value {
                entry.textualize_in(body)?;
            }
            Ok(())
        })
    }
}

impl DatomRealizing for GroupMap {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        let Some(MapSelection::Entries) = Self::select(block.shape, block.head()) else {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        };
        let entries = scope.realize_body(&mut |child_scope, child| {
            GroupMapEntry::realize_block(child_scope, child)
        })?;
        let mut map = BTreeMap::new();
        for entry in entries {
            if map.insert(entry.key, entry.value).is_some() {
                return Err(DatomFault {
                    problem: DatomProblem::Position,
                });
            }
        }
        Ok(Self { value: map })
    }
}

impl DatomTextualizing for GroupMap {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        for key in self.value.keys() {
            key.group_key()?;
        }
        scope.textualize_block(Shape::Guillemeted, None, |body| {
            for (key, value) in &self.value {
                GroupMapEntry {
                    key: key.clone(),
                    value: value.clone(),
                }
                .textualize_in(body)?;
            }
            Ok(())
        })
    }
}

impl DatomRealizing for TextMapEntry {
    fn realize_block(_scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        match block.shape {
            Shape::Bare | Shape::DottedBare => {
                let (key, value) = block.divide()?;
                key.text_key(&value)?;
                Ok(Self { key, value })
            }
            Shape::DottedCurlyQuoted => {
                let Some(head) = block.head() else {
                    return Err(DatomFault {
                        problem: DatomProblem::Head,
                    });
                };
                let key = head.text();
                let Some(carrier) = &block.string_carrier else {
                    return Err(DatomFault {
                        problem: DatomProblem::Shape,
                    });
                };
                let value = Text(carrier.textual_body().to_owned());
                key.text_key(&value)?;
                Ok(Self { key, value })
            }
            _ => Err(DatomFault {
                problem: DatomProblem::AmbiguousMapPair,
            }),
        }
    }
}

impl DatomTextualizing for TextMapEntry {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        self.key.text_key(&self.value)?;
        if self.value.fits_bare() {
            return scope.textualize_block(Shape::Bare, None, |body| {
                body.emit_scalar(&format!("{}.{}", self.key, self.value.0));
                Ok(())
            });
        }
        let head = Head(self.key.clone());
        scope.textualize_block(Shape::DottedCurlyQuoted, Some(&head), |body| {
            body.emit_scalar(&self.value.0);
            Ok(())
        })
    }
}

impl DatomRealizing for TextMap {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        let Some(MapSelection::Entries) = Self::select(block.shape, block.head()) else {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        };
        let entries = scope.realize_body(&mut |child_scope, child| {
            TextMapEntry::realize_block(child_scope, child)
        })?;
        let mut map = BTreeMap::new();
        for entry in entries {
            if map.insert(entry.key, entry.value).is_some() {
                return Err(DatomFault {
                    problem: DatomProblem::Position,
                });
            }
        }
        Ok(Self { value: map })
    }
}

impl DatomTextualizing for TextMap {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        for (key, value) in &self.value {
            key.text_key(value)?;
        }
        scope.textualize_block(Shape::Guillemeted, None, |body| {
            for (key, value) in &self.value {
                TextMapEntry {
                    key: key.clone(),
                    value: value.clone(),
                }
                .textualize_in(body)?;
            }
            Ok(())
        })
    }
}

impl GroupPayloading for Group {
    fn realize_group_payload(scope: &mut RealizeScope<'_>) -> Result<Self, DatomFault> {
        let mut position = RecordPosition { ordinal: 0 };
        let mut title = None;
        let mut children = None;
        let mut annotations = None;
        scope.realize_body(&mut |child_scope, child| {
            match position.next_position() {
                0 => title = Some(Text::realize_block(child_scope, child)?),
                1 => children = Some(EntryVector::realize_block(child_scope, child)?.value),
                2 => annotations = Some(TextMap::realize_block(child_scope, child)?.value),
                _ => {
                    return Err(DatomFault {
                        problem: DatomProblem::ExtraPosition,
                    });
                }
            }
            Ok(())
        })?;
        Ok(Self {
            title: title.ok_or(DatomFault {
                problem: DatomProblem::MissingPosition,
            })?,
            children: children.ok_or(DatomFault {
                problem: DatomProblem::MissingPosition,
            })?,
            annotations: annotations.ok_or(DatomFault {
                problem: DatomProblem::MissingPosition,
            })?,
        })
    }

    fn textualize_group_payload(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        self.title.textualize_in(scope)?;
        EntryVector {
            value: self.children.clone(),
        }
        .textualize_in(scope)?;
        TextMap {
            value: self.annotations.clone(),
        }
        .textualize_in(scope)
    }
}

impl DatomRealizing for Entry {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        match Self::select(block.shape, block.head()).ok_or(DatomFault {
            problem: DatomProblem::Shape,
        })? {
            EntrySelection::Note => {
                let Some(carrier) = &block.string_carrier else {
                    return Err(DatomFault {
                        problem: DatomProblem::Shape,
                    });
                };
                Ok(Self::Note(Text(carrier.textual_body().to_owned())))
            }
            EntrySelection::BareNote => {
                let (head, text) = block.divide()?;
                if head != "Note" {
                    return Err(DatomFault {
                        problem: DatomProblem::Shape,
                    });
                }
                Ok(Self::Note(text))
            }
            EntrySelection::Group => {
                let Some(GroupSelection::Payload) = Group::select(Shape::Braced, None) else {
                    return Err(DatomFault {
                        problem: DatomProblem::Shape,
                    });
                };
                Ok(Self::Group(Group::realize_group_payload(scope)?))
            }
            EntrySelection::Tags => {
                let Some(TagListSelection::Payload) = TagList::select(Shape::SquareBracketed, None)
                else {
                    return Err(DatomFault {
                        problem: DatomProblem::Shape,
                    });
                };
                Ok(Self::Tags(TagList::realize_tag_payload(scope)?))
            }
        }
    }
}

impl DatomTextualizing for Entry {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        match self {
            Self::Note(text) => {
                if text.fits_bare() {
                    return scope.textualize_block(Shape::Bare, None, |body| {
                        body.emit_scalar(&format!("Note.{}", text.0));
                        Ok(())
                    });
                }
                let head = Head("Note".into());
                scope.textualize_block(Shape::DottedCurlyQuoted, Some(&head), |body| {
                    body.emit_scalar(&text.0);
                    Ok(())
                })
            }
            Self::Group(group) => {
                let head = Head("Group".into());
                scope.textualize_block(Shape::DottedBraced, Some(&head), |body| {
                    let Some(GroupSelection::Payload) = Group::select(Shape::Braced, None) else {
                        return Err(DatomFault {
                            problem: DatomProblem::Shape,
                        });
                    };
                    group.textualize_group_payload(body)
                })
            }
            Self::Tags(tags) => {
                let head = Head("Tags".into());
                scope.textualize_block(Shape::DottedSquareBracketed, Some(&head), |body| {
                    let Some(TagListSelection::Payload) =
                        TagList::select(Shape::SquareBracketed, None)
                    else {
                        return Err(DatomFault {
                            problem: DatomProblem::Shape,
                        });
                    };
                    tags.textualize_tag_payload(body)
                })
            }
        }
    }
}

impl DatomRealizing for OptionalText {
    fn realize_block(_scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        match Self::select(block.shape, block.head()).ok_or(DatomFault {
            problem: DatomProblem::Shape,
        })? {
            OptionalSelection::Bare if block.body.0 == "None" => Ok(Self { value: None }),
            OptionalSelection::Bare => {
                let (head, value) = block.divide()?;
                if head != "Some" {
                    return Err(DatomFault {
                        problem: DatomProblem::Shape,
                    });
                }
                head.text_key(&value)?;
                Ok(Self { value: Some(value) })
            }
            OptionalSelection::Some => {
                let Some(carrier) = &block.string_carrier else {
                    return Err(DatomFault {
                        problem: DatomProblem::Shape,
                    });
                };
                Ok(Self {
                    value: Some(Text(carrier.textual_body().to_owned())),
                })
            }
        }
    }
}

impl DatomTextualizing for OptionalText {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        match &self.value {
            None => scope.textualize_block(Shape::Bare, None, |body| {
                body.emit_scalar("None");
                Ok(())
            }),
            Some(text) => {
                if text.fits_bare() {
                    "Some".to_owned().text_key(text)?;
                    return scope.textualize_block(Shape::Bare, None, |body| {
                        body.emit_scalar(&format!("Some.{}", text.0));
                        Ok(())
                    });
                }
                let head = Head("Some".into());
                scope.textualize_block(Shape::DottedCurlyQuoted, Some(&head), |body| {
                    body.emit_scalar(&text.0);
                    Ok(())
                })
            }
        }
    }
}

impl DatomRealizing for Report {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if Self::select(block.shape, block.head()).is_none() {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        let mut position = RecordPosition { ordinal: 0 };
        let mut heading = None;
        let mut groups = None;
        let mut latest = None;
        scope.realize_body(&mut |child_scope, child| {
            match position.next_position() {
                0 => heading = Some(Text::realize_block(child_scope, child)?),
                1 => groups = Some(GroupMap::realize_block(child_scope, child)?.value),
                2 => latest = Some(OptionalText::realize_block(child_scope, child)?.value),
                _ => {
                    return Err(DatomFault {
                        problem: DatomProblem::ExtraPosition,
                    });
                }
            }
            Ok(())
        })?;
        Ok(Self {
            heading: heading.ok_or(DatomFault {
                problem: DatomProblem::MissingPosition,
            })?,
            groups: groups.ok_or(DatomFault {
                problem: DatomProblem::MissingPosition,
            })?,
            latest: latest.ok_or(DatomFault {
                problem: DatomProblem::MissingPosition,
            })?,
        })
    }
}

impl DatomTextualizing for Report {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        self.heading.textualize_in(scope)?;
        GroupMap {
            value: self.groups.clone(),
        }
        .textualize_in(scope)?;
        OptionalText {
            value: self.latest.clone(),
        }
        .textualize_in(scope)
    }
}

impl DatomRealizing for InterimNote {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if Self::select(block.shape, block.head()).is_none() {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        let mut position = RecordPosition { ordinal: 0 };
        let mut fields = Vec::new();
        scope.realize_body(&mut |child_scope, child| {
            if position.next_position() >= 4 {
                return Err(DatomFault {
                    problem: DatomProblem::ExtraPosition,
                });
            }
            fields.push(Text::realize_block(child_scope, child)?.0);
            Ok(())
        })?;
        if fields.len() != 4 {
            return Err(DatomFault {
                problem: DatomProblem::MissingPosition,
            });
        }
        Ok(Self {
            a: fields.remove(0),
            b: fields.remove(0),
            c: fields.remove(0),
            d: fields.remove(0),
        })
    }
}

impl DatomTextualizing for InterimNote {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        for value in [&self.a, &self.b, &self.c, &self.d] {
            Text(value.clone()).textualize_in(scope)?;
        }
        Ok(())
    }
}

impl DatomRoot for Report {}

impl DatomRoot for InterimNote {}

impl EvidencedRealizing for ReportText {
    type Value = Report;

    fn realize_evidenced(&self) -> Result<Realized<Self::Value>, DatomFault> {
        let mut walk = RealizeWalk::default();
        let mut values = walk.realize_source(&self.source, |scope, block| {
            Report::realize_block(scope, block)
        })?;
        if values.len() != 1 {
            return Err(DatomFault {
                problem: DatomProblem::Position,
            });
        }
        Ok(Realized {
            value: values.remove(0),
            evidence: DatomEvidence {
                observation: walk.observation(),
                cursor: walk.cursor(),
            },
        })
    }
}

impl EvidencedRealizing for InterimNoteText {
    type Value = InterimNote;

    fn realize_evidenced(&self) -> Result<Realized<Self::Value>, DatomFault> {
        let mut walk = RealizeWalk::default();
        let mut values = walk.realize_source(&self.source, |scope, block| {
            InterimNote::realize_block(scope, block)
        })?;
        if values.len() != 1 {
            return Err(DatomFault {
                problem: DatomProblem::Position,
            });
        }
        Ok(Realized {
            value: values.remove(0),
            evidence: DatomEvidence {
                observation: walk.observation(),
                cursor: walk.cursor(),
            },
        })
    }
}

impl EvidencedTextualizing for Report {
    type Text = ReportText;

    fn textualize_evidenced(&self) -> Result<Projected<Self::Text>, DatomFault> {
        let mut walk = TextualizeWalk::default();
        let result: Result<(), DatomFault> = walk.textualize_source(|scope| {
            scope.textualize_block(Shape::Braced, None, |body| self.textualize_in(body))
        });
        result.map(|()| Projected {
            text: ReportText {
                source: walk.textual_source(),
            },
            evidence: DatomEvidence {
                observation: walk.observation(),
                cursor: walk.cursor(),
            },
        })
    }
}

impl EvidencedTextualizing for InterimNote {
    type Text = InterimNoteText;

    fn textualize_evidenced(&self) -> Result<Projected<Self::Text>, DatomFault> {
        let mut walk = TextualizeWalk::default();
        let result: Result<(), DatomFault> = walk.textualize_source(|scope| {
            scope.textualize_block(Shape::Braced, None, |body| self.textualize_in(body))
        });
        result.map(|()| Projected {
            text: InterimNoteText {
                source: walk.textual_source(),
            },
            evidence: DatomEvidence {
                observation: walk.observation(),
                cursor: walk.cursor(),
            },
        })
    }
}

impl Realize for ReportText {
    type Real = Report;
    type Fault = DatomFault;

    fn realize(&self) -> Result<Self::Real, Self::Fault> {
        self.realize_evidenced().map(|realized| realized.value)
    }
}

impl Realize for InterimNoteText {
    type Real = InterimNote;
    type Fault = DatomFault;

    fn realize(&self) -> Result<Self::Real, Self::Fault> {
        self.realize_evidenced().map(|realized| realized.value)
    }
}

impl Textualize for Report {
    type Textual = Result<ReportText, DatomFault>;

    fn textualize(&self) -> Self::Textual {
        self.textualize_evidenced().map(|projected| projected.text)
    }
}

impl Textualize for InterimNote {
    type Textual = Result<InterimNoteText, DatomFault>;

    fn textualize(&self) -> Self::Textual {
        self.textualize_evidenced().map(|projected| projected.text)
    }
}
