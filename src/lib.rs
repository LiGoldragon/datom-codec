//! Datomic: positional typed data over Protos.
//!
//! The datom dialect: Concept layer between Protoform and Corporate.

use std::convert::Infallible;
use std::fmt;

pub use protos::{
    Actualizable, Boolean, Boundary, Conceivable, Decimal, Delineation, Enclosure, Extent, Head,
    Incorporable, Integer, Path, Pathed, Potential, Protoform, Protosizable, Separator, Situated,
    Situating, Symbol, Text, Textualizable,
};

// ---------------------------------------------------------------------------
// Datom: the concept type of the datom dialect
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum Datom {
    Variant(Symbol, Separator, Option<Box<Datom>>),
    Struct(Vec<Datom>),
    Vector(Vec<Datom>),
    Text(Text),
    Meaning(Text),
    Bare(Symbol),
}

impl fmt::Debug for Datom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variant(h, s, b) => f.debug_tuple("Variant").field(h).field(s).field(b).finish(),
            Self::Struct(fields) => f.debug_tuple("Struct").field(fields).finish(),
            Self::Vector(items) => f.debug_tuple("Vector").field(items).finish(),
            Self::Text(t) => f.debug_tuple("Text").field(t).finish(),
            Self::Meaning(m) => f.debug_tuple("Meaning").field(m).finish(),
            Self::Bare(s) => f.debug_tuple("Bare").field(s).finish(),
        }
    }
}

impl PartialEq for Datom {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Variant(h1, s1, b1), Self::Variant(h2, s2, b2)) => {
                h1 == h2 && s1 == s2 && b1 == b2
            }
            (Self::Struct(f1), Self::Struct(f2)) => f1 == f2,
            (Self::Vector(v1), Self::Vector(v2)) => v1 == v2,
            (Self::Text(t1), Self::Text(t2)) => t1 == t2,
            (Self::Meaning(m1), Self::Meaning(m2)) => m1 == m2,
            (Self::Bare(s1), Self::Bare(s2)) => s1 == s2,
            _ => false,
        }
    }
}

impl Eq for Datom {}

#[derive(Clone, PartialEq, Eq)]
pub enum Meaning {
    Plain(Text),
}

impl fmt::Debug for Meaning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain(t) => f.debug_tuple("Plain").field(t).finish(),
        }
    }
}

// ---------------------------------------------------------------------------
// Fault types
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq)]
pub enum Expected {
    Variant,
    Struct,
    Vector,
    Text,
    Meaning,
    Integer,
    Decimal,
    Boolean,
    Bare,
}

impl fmt::Debug for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variant => write!(f, "Variant"),
            Self::Struct => write!(f, "Struct"),
            Self::Vector => write!(f, "Vector"),
            Self::Text => write!(f, "Text"),
            Self::Meaning => write!(f, "Meaning"),
            Self::Integer => write!(f, "Integer"),
            Self::Decimal => write!(f, "Decimal"),
            Self::Boolean => write!(f, "Boolean"),
            Self::Bare => write!(f, "Bare"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Problem {
    Shape(Expected, Datom),
    Arity(Integer, Integer),
    UnknownVariant(Symbol),
    Separator(Separator),
    Value(Text),
    OneValue,
}

impl fmt::Debug for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape(e, d) => f.debug_tuple("Shape").field(e).field(d).finish(),
            Self::Arity(e, a) => write!(f, "Arity({e}, {a})"),
            Self::UnknownVariant(s) => write!(f, "UnknownVariant({s:?})"),
            Self::Separator(s) => write!(f, "Separator({s:?})"),
            Self::Value(v) => write!(f, "Value({v:?})"),
            Self::OneValue => write!(f, "OneValue"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Fault {
    Structural(protos::Fault),
    Conceptual(Path, Problem),
    Corporate(Path, Problem),
}

impl fmt::Debug for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Structural(fault) => f.debug_tuple("Structural").field(fault).finish(),
            Self::Conceptual(p, prob) => f.debug_tuple("Conceptual").field(p).field(prob).finish(),
            Self::Corporate(p, prob) => f.debug_tuple("Corporate").field(p).field(prob).finish(),
        }
    }
}

impl From<protos::Fault> for Fault {
    fn from(f: protos::Fault) -> Self {
        Fault::Structural(f)
    }
}

impl Pathed for Fault {
    fn path(&self) -> &[Integer] {
        match self {
            Fault::Structural(_) => &[],
            Fault::Conceptual(path, _) | Fault::Corporate(path, _) => path,
        }
    }
}

trait Prepending {
    fn prepend(self, index: Integer) -> Self;
}

impl Prepending for Fault {
    fn prepend(self, index: Integer) -> Self {
        match self {
            Fault::Structural(f) => Fault::Structural(f),
            Fault::Conceptual(mut path, problem) => {
                path.insert(0, index);
                Fault::Conceptual(path, problem)
            }
            Fault::Corporate(mut path, problem) => {
                path.insert(0, index);
                Fault::Corporate(path, problem)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Protosizing: Datom -> Protoform (concept -> protoform layer, cannot fault)
// ---------------------------------------------------------------------------

trait Protosizing {
    fn to_protoform(&self) -> Protoform;
}

impl Protosizing for Datom {
    fn to_protoform(&self) -> Protoform {
        match self {
            Datom::Variant(head, sep, body) => {
                let body_pf = match body {
                    Some(b) => b.to_protoform(),
                    None => return Protoform::Bare(Head::Bare(head.clone())),
                };
                Protoform::Headed(Head::Bare(head.clone()), *sep, Box::new(body_pf))
            }
            Datom::Struct(fields) => Protoform::Enclosed(
                Enclosure::Braced,
                fields.iter().map(Protosizing::to_protoform).collect(),
            ),
            Datom::Vector(items) => Protoform::Enclosed(
                Enclosure::Bracketed,
                items.iter().map(Protosizing::to_protoform).collect(),
            ),
            Datom::Text(content) => Protoform::Opaque(Boundary::CurlyQuotes, content.clone()),
            Datom::Meaning(content) => Protoform::Opaque(Boundary::Parentheses, content.clone()),
            Datom::Bare(symbol) => Protoform::Bare(Head::Bare(symbol.clone())),
        }
    }
}

impl protos::Protosizable for Datom {
    type Fault = Infallible;

    fn protosize(&self) -> Result<Delineation, Infallible> {
        Ok(Delineation {
            protoforms: vec![self.to_protoform()],
            situation: protos::Situation::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Conceivable<Datom> for Protoform and Delineation (descent, may fault)
// ---------------------------------------------------------------------------

trait Conceiving {
    fn conceive_at(&self, path: &[Integer]) -> Result<Datom, Fault>;
}

impl Conceiving for Protoform {
    fn conceive_at(&self, path: &[Integer]) -> Result<Datom, Fault> {
        match self {
            Protoform::Headed(head, sep, body) => {
                let symbol = match head {
                    Head::Bare(s) => s.clone(),
                    Head::Qualified(_, _) => {
                        return Err(Fault::Conceptual(
                            path.to_vec(),
                            Problem::Shape(Expected::Variant, Datom::Bare("Qualified".to_owned())),
                        ));
                    }
                };
                let body_path: Path =
                    path.iter().copied().chain(std::iter::once(0)).collect();
                let body_datom = body.conceive_at(&body_path)?;
                Ok(Datom::Variant(symbol, *sep, Some(Box::new(body_datom))))
            }
            Protoform::Enclosed(enclosure, children) => match enclosure {
                Enclosure::Braced => {
                    let mut fields = Vec::with_capacity(children.len());
                    for (i, child) in children.iter().enumerate() {
                        let cp: Path = path
                            .iter().copied().chain(std::iter::once(i as Integer)).collect();
                        fields.push(child.conceive_at(&cp)?);
                    }
                    Ok(Datom::Struct(fields))
                }
                Enclosure::Bracketed => {
                    let mut items = Vec::with_capacity(children.len());
                    for (i, child) in children.iter().enumerate() {
                        let cp: Path = path
                            .iter().copied().chain(std::iter::once(i as Integer)).collect();
                        items.push(child.conceive_at(&cp)?);
                    }
                    Ok(Datom::Vector(items))
                }
                Enclosure::Angled => Err(Fault::Conceptual(
                    path.to_vec(),
                    Problem::Shape(Expected::Struct, Datom::Bare("Angled".to_owned())),
                )),
            },
            Protoform::Opaque(boundary, content) => match boundary {
                Boundary::CurlyQuotes => Ok(Datom::Text(content.clone())),
                Boundary::Parentheses => Ok(Datom::Meaning(content.clone())),
            },
            Protoform::Bare(head) => match head {
                Head::Bare(symbol) => Ok(Datom::Bare(symbol.clone())),
                Head::Qualified(_, _) => Err(Fault::Conceptual(
                    path.to_vec(),
                    Problem::Shape(Expected::Bare, Datom::Bare("Qualified".to_owned())),
                )),
            },
        }
    }
}

impl protos::Conceivable<Datom> for Protoform {
    type Fault = Fault;
    fn conceive(&self) -> Result<Datom, Fault> { self.conceive_at(&[]) }
}

impl protos::Conceivable<Datom> for Delineation {
    type Fault = Fault;
    fn conceive(&self) -> Result<Datom, Fault> {
        match self.protoforms.as_slice() {
            [pf] => pf.conceive_at(&[]),
            _ => Err(Fault::Conceptual(vec![], Problem::OneValue)),
        }
    }
}

// ---------------------------------------------------------------------------
// Datomic: the kind every corporate type of the dialect bears.
//
// Incorporable<T> for Datom is the public protos trait (concept bears it).
// Datomic::incorporate_from is the dispatch method that container impls call
// through T: Datomic to avoid trait-solver overflow on stable Rust.
// Each Incorporable<T> for Datom delegates to T::incorporate_from.
// ---------------------------------------------------------------------------

pub trait Datomic: Sized + protos::Conceivable<Datom, Fault = Infallible> {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault>;

    fn textualize(&self) -> Text {
        use protos::Textualizable as _;
        self.conceive().unwrap_or_else(|e| match e {}).to_protoform().textualize()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

trait VariantChaining {
    fn is_all_bare_chain(&self) -> bool;
    fn rejoin_chain(&self) -> String;
}

impl VariantChaining for Datom {
    fn is_all_bare_chain(&self) -> bool {
        match self {
            Datom::Bare(_) => true,
            Datom::Variant(_, _, Some(body)) => body.is_all_bare_chain(),
            Datom::Variant(_, _, None) => true,
            _ => false,
        }
    }
    fn rejoin_chain(&self) -> String {
        match self {
            Datom::Bare(s) => s.clone(),
            Datom::Variant(head, sep, Some(body)) => {
                format!("{}{}{}", head, sep.glyph(), body.rejoin_chain())
            }
            Datom::Variant(head, _, None) => head.clone(),
            _ => String::new(),
        }
    }
}

trait Glyphing {
    fn glyph(&self) -> char;
}

impl Glyphing for Separator {
    fn glyph(&self) -> char {
        match self {
            Self::Period => '.',
            Self::Exclamation => '!',
            Self::Colon => ':',
        }
    }
}

trait BareSafety {
    fn is_bare_safe(&self) -> bool;
}

impl BareSafety for str {
    fn is_bare_safe(&self) -> bool {
        if self.is_empty() { return false; }
        let delimiters = ['{', '}', '[', ']', '<', '>', '\u{201C}', '\u{201D}', '(', ')', ';'];
        for c in self.chars() {
            if c.is_whitespace() || delimiters.contains(&c) { return false; }
        }
        let first = self.chars().next().unwrap();
        let last = self.chars().next_back().unwrap();
        if matches!(first, '.' | '!' | ':') || matches!(last, '.' | '!' | ':') { return false; }
        let mut prev_sep = false;
        for c in self.chars() {
            let is_sep = matches!(c, '.' | '!' | ':');
            if is_sep && prev_sep { return false; }
            prev_sep = is_sep;
        }
        use protos::Protosizable as _;
        if let Ok(d) = self.to_owned().protosize() {
            if d.protoforms.len() == 1 {
                use protos::Conceivable;
                if let Ok(datom) = d.protoforms[0].conceive() {
                    if datom.is_all_bare_chain() { return datom.rejoin_chain() == self; }
                }
            }
        }
        false
    }
}

trait IntegerParsing {
    fn parse_integer(&self) -> Result<Integer, Fault>;
}

impl IntegerParsing for str {
    fn parse_integer(&self) -> Result<Integer, Fault> {
        if self.is_empty() {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        let digits = if let Some(rest) = self.strip_prefix('-') {
            if rest.is_empty() {
                return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
            }
            rest
        } else { self };
        if self.starts_with('+') {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        if digits.len() > 1 && digits.starts_with('0') {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        if self == "-0" {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        self.parse::<Integer>()
            .map_err(|_| Fault::Corporate(vec![], Problem::Value(self.to_owned())))
    }
}

// ---------------------------------------------------------------------------
// Datomic + Conceivable + Incorporable — scalars
// ---------------------------------------------------------------------------

impl protos::Conceivable<Datom> for Integer {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> { Ok(Datom::Bare(self.to_string())) }
}
impl Datomic for Integer {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Bare(s) => s.parse_integer(),
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Integer, other))),
        }
    }
}
impl protos::Incorporable<Integer> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Integer, Fault> { Integer::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for Boolean {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(Datom::Bare(if *self { "True" } else { "False" }.to_owned()))
    }
}
impl Datomic for Boolean {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Bare(s) if s == "True" => Ok(true),
            Datom::Bare(s) if s == "False" => Ok(false),
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Boolean, other))),
        }
    }
}
impl protos::Incorporable<Boolean> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Boolean, Fault> { Boolean::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for Decimal {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        let s = format!("{self}");
        if s.contains('.') {
            let trimmed = s.trim_end_matches('0');
            let result = if trimmed.ends_with('.') { format!("{trimmed}0") } else { trimmed.to_owned() };
            Ok(Datom::Bare(result))
        } else { Ok(Datom::Bare(format!("{s}.0"))) }
    }
}
impl Datomic for Decimal {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        if !datom.is_all_bare_chain() {
            return Err(Fault::Corporate(vec![], Problem::Shape(Expected::Decimal, datom)));
        }
        let s = datom.rejoin_chain();
        if !s.contains('.') { return Err(Fault::Corporate(vec![], Problem::Value(s))); }
        let value: Decimal = s.parse().map_err(|_| Fault::Corporate(vec![], Problem::Value(s.clone())))?;
        if !value.is_finite() { return Err(Fault::Corporate(vec![], Problem::Value(s))); }
        Ok(value)
    }
}
impl protos::Incorporable<Decimal> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Decimal, Fault> { Decimal::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for Text {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        if self.is_bare_safe() { Ok(Datom::Bare(self.clone())) }
        else { Ok(Datom::Text(self.clone())) }
    }
}
impl Datomic for Text {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Text(content) => Ok(content),
            Datom::Bare(symbol) => Ok(symbol),
            ref d if d.is_all_bare_chain() => Ok(d.rejoin_chain()),
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Text, other))),
        }
    }
}
impl protos::Incorporable<Text> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Text, Fault> { Text::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for Meaning {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        match self { Meaning::Plain(c) => Ok(Datom::Meaning(c.clone())) }
    }
}
impl Datomic for Meaning {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Meaning(content) => Ok(Meaning::Plain(content)),
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Meaning, other))),
        }
    }
}
impl protos::Incorporable<Meaning> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Meaning, Fault> { Meaning::incorporate_from(self) }
}

// ---------------------------------------------------------------------------
// Datomic + Conceivable + Incorporable — containers
// Container impls use T::incorporate_from (from Datomic) to dispatch
// element incorporation. This avoids trait-solver overflow that occurs
// when Datom: Incorporable<T> is used as a generic where clause.
// ---------------------------------------------------------------------------

impl<T: Datomic> protos::Conceivable<Datom> for Vec<T> {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(Datom::Vector(
            self.iter().map(|item| item.conceive().unwrap_or_else(|e| match e {})).collect(),
        ))
    }
}
impl<T: Datomic> Datomic for Vec<T> {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Vector(items) => items
                .into_iter()
                .enumerate()
                .map(|(i, item)| T::incorporate_from(item).map_err(|f| f.prepend(i as Integer)))
                .collect(),
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Vector, other))),
        }
    }
}
impl<T: Datomic> protos::Incorporable<Vec<T>> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Vec<T>, Fault> { Vec::<T>::incorporate_from(self) }
}

impl<T: Datomic> protos::Conceivable<Datom> for Option<T> {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        match self {
            None => Ok(Datom::Bare("None".to_owned())),
            Some(val) => Ok(Datom::Variant(
                "Some".to_owned(), Separator::Period,
                Some(Box::new(val.conceive().unwrap_or_else(|e| match e {}))),
            )),
        }
    }
}
impl<T: Datomic> Datomic for Option<T> {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Bare(ref s) if s == "None" => Ok(None),
            Datom::Variant(head, sep, body) if head == "Some" => {
                if sep != Separator::Period {
                    return Err(Fault::Corporate(vec![], Problem::Separator(sep)));
                }
                match body {
                    Some(b) => T::incorporate_from(*b).map(Some),
                    None => Err(Fault::Corporate(vec![], Problem::Shape(
                        Expected::Variant, Datom::Variant(head, sep, None)))),
                }
            }
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Variant, other))),
        }
    }
}
impl<T: Datomic> protos::Incorporable<Option<T>> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Option<T>, Fault> { Option::<T>::incorporate_from(self) }
}

impl<T: Datomic, E: Datomic> protos::Conceivable<Datom> for Result<T, E> {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        match self {
            Ok(val) => Ok(Datom::Variant(
                "Ok".to_owned(), Separator::Period,
                Some(Box::new(val.conceive().unwrap_or_else(|e| match e {}))),
            )),
            Err(err) => Ok(Datom::Variant(
                "Err".to_owned(), Separator::Period,
                Some(Box::new(err.conceive().unwrap_or_else(|e| match e {}))),
            )),
        }
    }
}
impl<T: Datomic, E: Datomic> Datomic for Result<T, E> {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(head, sep, body) => {
                if sep != Separator::Period {
                    return Err(Fault::Corporate(vec![], Problem::Separator(sep)));
                }
                match (head.as_str(), body) {
                    ("Ok", Some(b)) => T::incorporate_from(*b).map(Ok),
                    ("Err", Some(b)) => E::incorporate_from(*b).map(Err),
                    _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(head))),
                }
            }
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Variant, other))),
        }
    }
}
impl<T: Datomic, E: Datomic> protos::Incorporable<Result<T, E>> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Result<T, E>, Fault> { Result::<T, E>::incorporate_from(self) }
}

// ---------------------------------------------------------------------------
// Datomic + Conceivable + Incorporable — fault types
// ---------------------------------------------------------------------------

impl protos::Conceivable<Datom> for Expected {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(Datom::Bare(match self {
            Self::Variant => "Variant", Self::Struct => "Struct", Self::Vector => "Vector",
            Self::Text => "Text", Self::Meaning => "Meaning", Self::Integer => "Integer",
            Self::Decimal => "Decimal", Self::Boolean => "Boolean", Self::Bare => "Bare",
        }.to_owned()))
    }
}
impl Datomic for Expected {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Bare(s) => match s.as_str() {
                "Variant" => Ok(Self::Variant), "Struct" => Ok(Self::Struct),
                "Vector" => Ok(Self::Vector), "Text" => Ok(Self::Text),
                "Meaning" => Ok(Self::Meaning), "Integer" => Ok(Self::Integer),
                "Decimal" => Ok(Self::Decimal), "Boolean" => Ok(Self::Boolean),
                "Bare" => Ok(Self::Bare),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Bare, other))),
        }
    }
}
impl protos::Incorporable<Expected> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Expected, Fault> { Expected::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for Problem {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(match self {
            Problem::Shape(expected, datom) => Datom::Variant(
                "Shape".to_owned(), Separator::Period,
                Some(Box::new(Datom::Struct(vec![expected.conceive()?, datom.clone()]))),
            ),
            Problem::Arity(expected, actual) => Datom::Variant(
                "Arity".to_owned(), Separator::Period,
                Some(Box::new(Datom::Struct(vec![expected.conceive()?, actual.conceive()?]))),
            ),
            Problem::UnknownVariant(s) => Datom::Variant(
                "UnknownVariant".to_owned(), Separator::Period, Some(Box::new(s.conceive()?)),
            ),
            Problem::Separator(sep) => Datom::Variant(
                "Separator".to_owned(), Separator::Period, Some(Box::new(sep.conceive()?)),
            ),
            Problem::Value(v) => Datom::Variant(
                "Value".to_owned(), Separator::Period, Some(Box::new(v.conceive()?)),
            ),
            Problem::OneValue => Datom::Bare("OneValue".to_owned()),
        })
    }
}
impl Datomic for Problem {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(head, sep, body) => {
                if sep != Separator::Period {
                    return Err(Fault::Corporate(vec![], Problem::Separator(sep)));
                }
                match (head.as_str(), body) {
                    ("Shape", Some(b)) => match *b {
                        Datom::Struct(fields) if fields.len() == 2 => {
                            let mut it = fields.into_iter();
                            let expected = Expected::incorporate_from(it.next().unwrap())?;
                            Ok(Problem::Shape(expected, it.next().unwrap()))
                        }
                        other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Struct, other))),
                    },
                    ("Arity", Some(b)) => match *b {
                        Datom::Struct(fields) if fields.len() == 2 => {
                            let mut it = fields.into_iter();
                            let expected = Integer::incorporate_from(it.next().unwrap())?;
                            let actual = Integer::incorporate_from(it.next().unwrap())?;
                            Ok(Problem::Arity(expected, actual))
                        }
                        other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Struct, other))),
                    },
                    ("UnknownVariant", Some(b)) => Text::incorporate_from(*b).map(Problem::UnknownVariant),
                    ("Value", Some(b)) => Text::incorporate_from(*b).map(Problem::Value),
                    ("Separator", Some(b)) => Separator::incorporate_from(*b).map(Problem::Separator),
                    _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(head))),
                }
            }
            Datom::Bare(s) => match s.as_str() {
                "OneValue" => Ok(Problem::OneValue),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Variant, other))),
        }
    }
}
impl protos::Incorporable<Problem> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Problem, Fault> { Problem::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for Fault {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(match self {
            Fault::Structural(f) => Datom::Variant(
                "Structural".to_owned(), Separator::Period, Some(Box::new(f.conceive()?)),
            ),
            Fault::Conceptual(path, problem) => Datom::Variant(
                "Conceptual".to_owned(), Separator::Period,
                Some(Box::new(Datom::Struct(vec![path.conceive()?, problem.conceive()?]))),
            ),
            Fault::Corporate(path, problem) => Datom::Variant(
                "Corporate".to_owned(), Separator::Period,
                Some(Box::new(Datom::Struct(vec![path.conceive()?, problem.conceive()?]))),
            ),
        })
    }
}
impl Datomic for Fault {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(head, sep, body) => {
                if sep != Separator::Period {
                    return Err(Fault::Corporate(vec![], Problem::Separator(sep)));
                }
                match (head.as_str(), body) {
                    ("Structural", Some(b)) => protos::Fault::incorporate_from(*b).map(Fault::Structural),
                    ("Conceptual", Some(b)) => match *b {
                        Datom::Struct(fields) if fields.len() == 2 => {
                            let mut it = fields.into_iter();
                            let path = Vec::<Integer>::incorporate_from(it.next().unwrap())?;
                            let problem = Problem::incorporate_from(it.next().unwrap())?;
                            Ok(Fault::Conceptual(path, problem))
                        }
                        other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Struct, other))),
                    },
                    ("Corporate", Some(b)) => match *b {
                        Datom::Struct(fields) if fields.len() == 2 => {
                            let mut it = fields.into_iter();
                            let path = Vec::<Integer>::incorporate_from(it.next().unwrap())?;
                            let problem = Problem::incorporate_from(it.next().unwrap())?;
                            Ok(Fault::Corporate(path, problem))
                        }
                        other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Struct, other))),
                    },
                    _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(head))),
                }
            }
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Variant, other))),
        }
    }
}
impl protos::Incorporable<Fault> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Fault, Fault> { Fault::incorporate_from(self) }
}

// ---------------------------------------------------------------------------
// Datomic + Conceivable + Incorporable — protos structural types
// ---------------------------------------------------------------------------

impl protos::Conceivable<Datom> for Separator {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(Datom::Bare(match self {
            Self::Period => "Period", Self::Exclamation => "Exclamation", Self::Colon => "Colon",
        }.to_owned()))
    }
}
impl Datomic for Separator {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Bare(s) => match s.as_str() {
                "Period" => Ok(Separator::Period),
                "Exclamation" => Ok(Separator::Exclamation),
                "Colon" => Ok(Separator::Colon),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Bare, other))),
        }
    }
}
impl protos::Incorporable<Separator> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Separator, Fault> { Separator::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for Enclosure {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(Datom::Bare(match self {
            Self::Braced => "Braced", Self::Bracketed => "Bracketed", Self::Angled => "Angled",
        }.to_owned()))
    }
}
impl Datomic for Enclosure {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Bare(s) => match s.as_str() {
                "Braced" => Ok(Enclosure::Braced),
                "Bracketed" => Ok(Enclosure::Bracketed),
                "Angled" => Ok(Enclosure::Angled),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Bare, other))),
        }
    }
}
impl protos::Incorporable<Enclosure> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Enclosure, Fault> { Enclosure::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for Boundary {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(Datom::Bare(match self {
            Self::CurlyQuotes => "CurlyQuotes", Self::Parentheses => "Parentheses",
        }.to_owned()))
    }
}
impl Datomic for Boundary {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Bare(s) => match s.as_str() {
                "CurlyQuotes" => Ok(Boundary::CurlyQuotes),
                "Parentheses" => Ok(Boundary::Parentheses),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Bare, other))),
        }
    }
}
impl protos::Incorporable<Boundary> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Boundary, Fault> { Boundary::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for Extent {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(Datom::Struct(vec![self.0.conceive()?, self.1.conceive()?]))
    }
}
impl Datomic for Extent {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Struct(fields) => {
                if fields.len() != 2 {
                    return Err(Fault::Corporate(vec![], Problem::Arity(2, fields.len() as Integer)));
                }
                let mut it = fields.into_iter();
                let start = Integer::incorporate_from(it.next().unwrap())?;
                let end = Integer::incorporate_from(it.next().unwrap())?;
                Ok(Extent(start, end))
            }
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Struct, other))),
        }
    }
}
impl protos::Incorporable<Extent> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Extent, Fault> { Extent::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for protos::Problem {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(match self {
            protos::Problem::Unclosed(e) => Datom::Variant(
                "Unclosed".to_owned(), Separator::Period, Some(Box::new(e.conceive()?)),
            ),
            protos::Problem::UnclosedBoundary(b) => Datom::Variant(
                "UnclosedBoundary".to_owned(), Separator::Period, Some(Box::new(b.conceive()?)),
            ),
            protos::Problem::Unopened => Datom::Bare("Unopened".to_owned()),
            protos::Problem::MissingBody => Datom::Bare("MissingBody".to_owned()),
            protos::Problem::MissingHead => Datom::Bare("MissingHead".to_owned()),
        })
    }
}
impl Datomic for protos::Problem {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(head, sep, body) => {
                if sep != Separator::Period {
                    return Err(Fault::Corporate(vec![], Problem::Separator(sep)));
                }
                match (head.as_str(), body) {
                    ("Unclosed", Some(b)) => Enclosure::incorporate_from(*b).map(protos::Problem::Unclosed),
                    ("UnclosedBoundary", Some(b)) => Boundary::incorporate_from(*b).map(protos::Problem::UnclosedBoundary),
                    _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(head))),
                }
            }
            Datom::Bare(s) => match s.as_str() {
                "Unopened" => Ok(protos::Problem::Unopened),
                "MissingBody" => Ok(protos::Problem::MissingBody),
                "MissingHead" => Ok(protos::Problem::MissingHead),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Variant, other))),
        }
    }
}
impl protos::Incorporable<protos::Problem> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<protos::Problem, Fault> { protos::Problem::incorporate_from(self) }
}

impl protos::Conceivable<Datom> for protos::Fault {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(Datom::Struct(vec![self.extent.conceive()?, self.problem.conceive()?]))
    }
}
impl Datomic for protos::Fault {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Struct(fields) => {
                if fields.len() != 2 {
                    return Err(Fault::Corporate(vec![], Problem::Arity(2, fields.len() as Integer)));
                }
                let mut it = fields.into_iter();
                let extent = Extent::incorporate_from(it.next().unwrap())?;
                let problem = protos::Problem::incorporate_from(it.next().unwrap())?;
                Ok(protos::Fault { extent, problem })
            }
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Struct, other))),
        }
    }
}
impl protos::Incorporable<protos::Fault> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<protos::Fault, Fault> { protos::Fault::incorporate_from(self) }
}

// Datom identity

impl protos::Conceivable<Datom> for Datom {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> { Ok(self.clone()) }
}
impl Datomic for Datom {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> { Ok(datom) }
}
impl protos::Incorporable<Datom> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Datom, Fault> { Ok(self) }
}

// ---------------------------------------------------------------------------
// Box<T>: blanket Incorporable, macro for Conceivable + Datomic
// ---------------------------------------------------------------------------

impl<T: Datomic> protos::Incorporable<Box<T>> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Box<T>, Fault> { T::incorporate_from(self).map(Box::new) }
}

#[macro_export]
macro_rules! impl_datomic_box {
    ($t:ty) => {
        impl protos::Conceivable<$crate::Datom> for Box<$t> {
            type Fault = std::convert::Infallible;
            fn conceive(&self) -> Result<$crate::Datom, std::convert::Infallible> {
                protos::Conceivable::<$crate::Datom>::conceive(self.as_ref())
            }
        }
        impl $crate::Datomic for Box<$t> {
            fn incorporate_from(datom: $crate::Datom) -> Result<Self, $crate::Fault> {
                <$t as $crate::Datomic>::incorporate_from(datom).map(Box::new)
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Situated<F>
// ---------------------------------------------------------------------------

impl<F: Datomic> protos::Conceivable<Datom> for Situated<F> {
    type Fault = Infallible;
    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(Datom::Struct(vec![self.0.conceive()?, self.1.conceive()?]))
    }
}
impl<F: Datomic> Datomic for Situated<F> {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Struct(fields) if fields.len() == 2 => {
                let mut it = fields.into_iter();
                let extent = Option::<Extent>::incorporate_from(it.next().unwrap())?;
                let fault = F::incorporate_from(it.next().unwrap())?;
                Ok(Situated(extent, fault))
            }
            Datom::Struct(fields) => Err(Fault::Corporate(vec![], Problem::Arity(2, fields.len() as Integer))),
            other => Err(Fault::Corporate(vec![], Problem::Shape(Expected::Struct, other))),
        }
    }
}
impl<F: Datomic> protos::Incorporable<Situated<F>> for Datom {
    type Fault = Fault;
    fn incorporate(self) -> Result<Situated<F>, Fault> { Situated::<F>::incorporate_from(self) }
}
