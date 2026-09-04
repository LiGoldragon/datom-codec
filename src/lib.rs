//! Datomic: positional typed data over Protos Protoform.
//!
//! The datom dialect: Concept layer between Protoform and Corporal.

use std::collections::BTreeMap;
use std::fmt;

pub use protos::{
    Boolean, Boundary, Decimal, Delineation, Embodied, Enclosure, Extent, Integer, Path, Potential,
    Printing, Protoform, Protosizable, Separator, Situating, Structural, Symbol, Text,
};

// ---------------------------------------------------------------------------
// Datom: the concept type of the datom dialect
// ---------------------------------------------------------------------------

/// The concept type of the datom dialect.
#[derive(Clone)]
pub enum Datom {
    /// Head, separator, optional body: `Head.body`
    Variant(Symbol, Separator, Option<Box<Datom>>),
    /// `{ ... }` — positional struct fields
    Struct(Vec<Datom>),
    /// `[ ... ]` — vector elements
    Vector(Vec<Datom>),
    /// `\u{00AB} k v ... \u{00BB}` — map pairs by position
    Map(Vec<Pair>),
    /// `\u{201C}...\u{201D}` — plain text
    Text(Text),
    /// `(...)` — meaning (today: plain text)
    Meaning(Text),
    /// A bare word
    Bare(Symbol),
}

impl fmt::Debug for Datom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variant(h, s, b) => f.debug_tuple("Variant").field(h).field(s).field(b).finish(),
            Self::Struct(fields) => f.debug_tuple("Struct").field(fields).finish(),
            Self::Vector(items) => f.debug_tuple("Vector").field(items).finish(),
            Self::Map(pairs) => f.debug_tuple("Map").field(pairs).finish(),
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
            (Self::Map(m1), Self::Map(m2)) => m1 == m2,
            (Self::Text(t1), Self::Text(t2)) => t1 == t2,
            (Self::Meaning(m1), Self::Meaning(m2)) => m1 == m2,
            (Self::Bare(s1), Self::Bare(s2)) => s1 == s2,
            _ => false,
        }
    }
}

impl Eq for Datom {}

/// A key-value pair in a datom map.
#[derive(Clone, PartialEq, Eq)]
pub struct Pair(pub Datom, pub Datom);

impl fmt::Debug for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Pair").field(&self.0).field(&self.1).finish()
    }
}

/// Meaning: today, parenthesized text lands as plain.
#[derive(Clone, PartialEq, Eq)]
pub enum MeaningValue {
    Plain(Text),
}

impl fmt::Debug for MeaningValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain(t) => f.debug_tuple("Plain").field(t).finish(),
        }
    }
}

// ---------------------------------------------------------------------------
// Fault types
// ---------------------------------------------------------------------------

/// What was expected at a position.
#[derive(Clone, PartialEq, Eq)]
pub enum Expected {
    Variant,
    Struct,
    Vector,
    Map,
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
            Self::Map => write!(f, "Map"),
            Self::Text => write!(f, "Text"),
            Self::Meaning => write!(f, "Meaning"),
            Self::Integer => write!(f, "Integer"),
            Self::Decimal => write!(f, "Decimal"),
            Self::Boolean => write!(f, "Boolean"),
            Self::Bare => write!(f, "Bare"),
        }
    }
}

/// The problem taxonomy of the datom dialect.
#[derive(Clone, PartialEq, Eq)]
pub enum Problem {
    Shape(Expected, Datom),
    Arity(Integer, Integer),
    UnknownVariant(Symbol),
    Separator(Separator),
    Value(Text),
    Pairing,
    DuplicateKey(Datom),
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
            Self::Pairing => write!(f, "Pairing"),
            Self::DuplicateKey(d) => f.debug_tuple("DuplicateKey").field(d).finish(),
            Self::OneValue => write!(f, "OneValue"),
        }
    }
}

/// The three-layer fault taxonomy.
#[derive(Clone, PartialEq, Eq)]
pub enum Fault {
    Structural(protos::Fault),
    Conceptual(Path, Problem),
    Corporal(Path, Problem),
}

impl fmt::Debug for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Structural(fault) => f.debug_tuple("Structural").field(fault).finish(),
            Self::Conceptual(p, prob) => f.debug_tuple("Conceptual").field(p).field(prob).finish(),
            Self::Corporal(p, prob) => f.debug_tuple("Corporal").field(p).field(prob).finish(),
        }
    }
}

/// A fault joined to its extent by actualize.
#[derive(Clone)]
pub struct Situated(pub Option<Extent>, pub Fault);

impl fmt::Debug for Situated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Situated")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Protosizable for Datom (Datom -> Protoform)
// ---------------------------------------------------------------------------

impl Protosizable for Datom {
    fn protosize(&self) -> Protoform {
        match self {
            Datom::Variant(head, sep, body) => {
                let body_pf = match body {
                    Some(b) => b.protosize(),
                    None => return Protoform::Bare(head.clone()),
                };
                Protoform::Headed(head.clone(), *sep, Box::new(body_pf))
            }
            Datom::Struct(fields) => {
                let children = fields.iter().map(Protosizable::protosize).collect();
                Protoform::Enclosed(Enclosure::Braced, children)
            }
            Datom::Vector(items) => {
                let children = items.iter().map(Protosizable::protosize).collect();
                Protoform::Enclosed(Enclosure::Bracketed, children)
            }
            Datom::Map(pairs) => {
                let mut children = Vec::with_capacity(pairs.len() * 2);
                for Pair(k, v) in pairs {
                    children.push(k.protosize());
                    children.push(v.protosize());
                }
                Protoform::Enclosed(Enclosure::Guillemets, children)
            }
            Datom::Text(content) => Protoform::Opaque(Boundary::CurlyQuotes, content.clone()),
            Datom::Meaning(content) => Protoform::Opaque(Boundary::Parentheses, content.clone()),
            Datom::Bare(symbol) => Protoform::Bare(symbol.clone()),
        }
    }
}

// ---------------------------------------------------------------------------
// Conceptual<Datom> for Protoform (Protoform -> Datom)
// ---------------------------------------------------------------------------

impl protos::Conceptual<Datom> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Datom, Fault> {
        conceive_protoform(self, &[])
    }
}

fn conceive_protoform(pf: &Protoform, path: &[Integer]) -> Result<Datom, Fault> {
    match pf {
        Protoform::Headed(head, sep, body) => {
            let body_path: Path = path.iter().copied().chain(std::iter::once(0)).collect();
            let body_datom = conceive_protoform(body, &body_path)?;
            Ok(Datom::Variant(
                head.clone(),
                *sep,
                Some(Box::new(body_datom)),
            ))
        }
        Protoform::Enclosed(enclosure, children) => match enclosure {
            Enclosure::Braced => {
                let mut fields = Vec::with_capacity(children.len());
                for (i, child) in children.iter().enumerate() {
                    let child_path: Path = path
                        .iter()
                        .copied()
                        .chain(std::iter::once(i as Integer))
                        .collect();
                    fields.push(conceive_protoform(child, &child_path)?);
                }
                Ok(Datom::Struct(fields))
            }
            Enclosure::Bracketed => {
                let mut items = Vec::with_capacity(children.len());
                for (i, child) in children.iter().enumerate() {
                    let child_path: Path = path
                        .iter()
                        .copied()
                        .chain(std::iter::once(i as Integer))
                        .collect();
                    items.push(conceive_protoform(child, &child_path)?);
                }
                Ok(Datom::Vector(items))
            }
            Enclosure::Guillemets => {
                if children.len() % 2 != 0 {
                    return Err(Fault::Conceptual(path.to_vec(), Problem::Pairing));
                }
                let mut pairs = Vec::with_capacity(children.len() / 2);
                for chunk in children.chunks_exact(2) {
                    let ki = pairs.len() * 2;
                    let vi = ki + 1;
                    let k_path: Path = path
                        .iter()
                        .copied()
                        .chain(std::iter::once(ki as Integer))
                        .collect();
                    let v_path: Path = path
                        .iter()
                        .copied()
                        .chain(std::iter::once(vi as Integer))
                        .collect();
                    let key = conceive_protoform(&chunk[0], &k_path)?;
                    let val = conceive_protoform(&chunk[1], &v_path)?;
                    pairs.push(Pair(key, val));
                }
                Ok(Datom::Map(pairs))
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
        Protoform::Bare(symbol) => Ok(Datom::Bare(symbol.clone())),
    }
}

// ---------------------------------------------------------------------------
// Conceptual<Datom> for Delineation (exactly one protoform)
// ---------------------------------------------------------------------------

impl protos::Conceptual<Datom> for Delineation {
    type Fault = Fault;

    fn conceive(&self) -> Result<Datom, Fault> {
        match self.protoforms.as_slice() {
            [pf] => pf.conceive(),
            _ => Err(Fault::Conceptual(vec![], Problem::OneValue)),
        }
    }
}

// ---------------------------------------------------------------------------
// Datomic: the corporal kind of the datom dialect
// ---------------------------------------------------------------------------

/// The corporal kind of the datom dialect.
pub trait Datomic: Embodied {
    /// Realize a corporal value from a datom. Static method.
    fn incorporate(datom: Datom) -> Result<Self, Fault>;

    /// Project a corporal value into a datom.
    fn datomize(&self) -> Datom;
}

/// Provided for every Datomic: the whole chain, datomize -> protosize -> print.
pub trait Textualizable {
    fn textualize(&self) -> Text;
}

impl<T: Datomic> Textualizable for T {
    fn textualize(&self) -> Text {
        self.datomize().protosize().print()
    }
}

// ---------------------------------------------------------------------------
// Actualize: the full chain for Potential<T: Datomic>
// ---------------------------------------------------------------------------
// API deviation: Actualizable<T> for Potential<T> cannot be implemented here
// due to the orphan rule (both trait and type are in protos, T is uncovered).
// Instead, Potential<T> gains an inherent `actualize` method via a trait
// extension defined in this crate.

/// Actualization for any datomic potential.
pub trait DatomicActualizable<T: Datomic> {
    /// Delineate, conceive, incorporate: the full realization chain.
    fn actualize(&self) -> Result<T, Situated>;
}

impl<T: Datomic> DatomicActualizable<T> for Potential<T> {
    fn actualize(&self) -> Result<T, Situated> {
        use protos::Conceptual;

        let delineation = self
            .text()
            .to_owned()
            .delineate()
            .map_err(|f| Situated(Some(f.extent), Fault::Structural(f.clone())))?;

        let datom: Datom = delineation
            .conceive()
            .map_err(|f| situate_fault(&delineation, f))?;

        T::incorporate(datom).map_err(|f| situate_fault(&delineation, f))
    }
}

fn situate_fault(delineation: &Delineation, fault: Fault) -> Situated {
    let extent = match &fault {
        Fault::Structural(f) => Some(f.extent),
        Fault::Conceptual(path, _) | Fault::Corporal(path, _) => delineation.situate(path),
    };
    Situated(extent, fault)
}

// ---------------------------------------------------------------------------
// Helpers: bare-string rule and rejoin
// ---------------------------------------------------------------------------

/// Check if a Datom is an all-bare Variant chain (for the bare-string rule).
fn is_all_bare_variant_chain(datom: &Datom) -> bool {
    match datom {
        Datom::Bare(_) => true,
        Datom::Variant(_, _, Some(body)) => is_all_bare_variant_chain(body),
        Datom::Variant(_, _, None) => true,
        _ => false,
    }
}

/// Rejoin an all-bare Variant chain into a string.
fn rejoin_variant_chain(datom: &Datom) -> String {
    match datom {
        Datom::Bare(s) => s.clone(),
        Datom::Variant(head, sep, Some(body)) => {
            format!("{}{}{}", head, sep.glyph(), rejoin_variant_chain(body))
        }
        Datom::Variant(head, _, None) => head.clone(),
        _ => String::new(),
    }
}

/// Check if a string is bare-safe for datomize: non-empty, no whitespace,
/// no delimiter/quote glyph, no leading/trailing/doubled separator,
/// and delineates-and-rejoins to itself.
fn is_bare_safe(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let delimiters = [
        '{', '}', '[', ']', '\u{00AB}', '\u{00BB}', '<', '>', '\u{201C}', '\u{201D}', '(', ')', ';',
    ];

    for c in s.chars() {
        if c.is_whitespace() || delimiters.contains(&c) {
            return false;
        }
    }

    // Check no leading/trailing separator
    let first = s.chars().next().unwrap();
    let last = s.chars().next_back().unwrap();
    if matches!(first, '.' | '!' | ':') || matches!(last, '.' | '!' | ':') {
        return false;
    }

    // Check no doubled separator
    let mut prev_sep = false;
    for c in s.chars() {
        let is_sep = matches!(c, '.' | '!' | ':');
        if is_sep && prev_sep {
            return false;
        }
        prev_sep = is_sep;
    }

    // Verify round-trip: delineate and rejoin must produce the same string
    if let Ok(d) = s.to_owned().delineate() {
        if d.protoforms.len() == 1 {
            use protos::Conceptual;
            if let Ok(datom) = d.protoforms[0].conceive() {
                if is_all_bare_variant_chain(&datom) {
                    return rejoin_variant_chain(&datom) == s;
                }
            }
        }
    }

    false
}

// ---------------------------------------------------------------------------
// Datomic for Integer
// ---------------------------------------------------------------------------

impl Datomic for Integer {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) => parse_integer(s),
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Integer, datom),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        Datom::Bare(self.to_string())
    }
}

fn parse_integer(s: &str) -> Result<Integer, Fault> {
    // Rules: ASCII decimal, optional leading `-`, no `+`, no leading zero except `0` itself
    if s.is_empty() {
        return Err(Fault::Corporal(vec![], Problem::Value(s.to_owned())));
    }

    let digits = if let Some(rest) = s.strip_prefix('-') {
        if rest.is_empty() {
            return Err(Fault::Corporal(vec![], Problem::Value(s.to_owned())));
        }
        rest
    } else {
        s
    };

    // No leading `+`
    if s.starts_with('+') {
        return Err(Fault::Corporal(vec![], Problem::Value(s.to_owned())));
    }

    // All digits must be ASCII
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(Fault::Corporal(vec![], Problem::Value(s.to_owned())));
    }

    // No leading zero except `0` itself
    if digits.len() > 1 && digits.starts_with('0') {
        return Err(Fault::Corporal(vec![], Problem::Value(s.to_owned())));
    }

    // `-0` is invalid
    if s == "-0" {
        return Err(Fault::Corporal(vec![], Problem::Value(s.to_owned())));
    }

    s.parse::<Integer>()
        .map_err(|_| Fault::Corporal(vec![], Problem::Value(s.to_owned())))
}

// ---------------------------------------------------------------------------
// Datomic for Boolean
// ---------------------------------------------------------------------------

impl Datomic for Boolean {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) if s == "True" => Ok(true),
            Datom::Bare(s) if s == "False" => Ok(false),
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Boolean, datom),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        Datom::Bare(if *self { "True" } else { "False" }.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Datomic for Decimal
// ---------------------------------------------------------------------------

impl Datomic for Decimal {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        // A Bare or an all-bare Variant chain rejoined (e.g., `3.14` delineates as Headed)
        let s = match &datom {
            Datom::Bare(s) => s.clone(),
            _ if is_all_bare_variant_chain(&datom) => rejoin_variant_chain(&datom),
            _ => {
                return Err(Fault::Corporal(
                    vec![],
                    Problem::Shape(Expected::Decimal, datom),
                ));
            }
        };

        // Must contain a `.`
        if !s.contains('.') {
            return Err(Fault::Corporal(vec![], Problem::Value(s)));
        }

        let value: Decimal = s
            .parse()
            .map_err(|_| Fault::Corporal(vec![], Problem::Value(s.clone())))?;

        if !value.is_finite() {
            return Err(Fault::Corporal(vec![], Problem::Value(s)));
        }

        Ok(value)
    }

    fn datomize(&self) -> Datom {
        // Shortest exact form with a `.`
        let s = format!("{self}");
        if s.contains('.') {
            // Remove trailing zeros after the decimal point, but keep at least one digit
            let trimmed = s.trim_end_matches('0');
            let result = if trimmed.ends_with('.') {
                format!("{trimmed}0")
            } else {
                trimmed.to_owned()
            };
            // This will delineate as a Headed if it contains a period
            // which is correct -- it rejoins on incorporate
            Datom::Bare(result)
        } else {
            Datom::Bare(format!("{s}.0"))
        }
    }
}

// ---------------------------------------------------------------------------
// Datomic for Text (String)
// ---------------------------------------------------------------------------

impl Datomic for Text {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Text(content) => Ok(content),
            Datom::Bare(symbol) => Ok(symbol),
            ref d if is_all_bare_variant_chain(d) => Ok(rejoin_variant_chain(d)),
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Text, datom),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        if is_bare_safe(self) {
            Datom::Bare(self.clone())
        } else {
            Datom::Text(self.clone())
        }
    }
}

// ---------------------------------------------------------------------------
// Datomic for MeaningValue
// ---------------------------------------------------------------------------

impl Datomic for MeaningValue {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Meaning(content) => Ok(MeaningValue::Plain(content)),
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Meaning, datom),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        match self {
            MeaningValue::Plain(content) => Datom::Meaning(content.clone()),
        }
    }
}

// ---------------------------------------------------------------------------
// Datomic for Vec<T>
// ---------------------------------------------------------------------------

impl<T: Datomic> Datomic for Vec<T> {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Vector(items) => items
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    T::incorporate(item).map_err(|f| prepend_path_to_fault(f, i as Integer))
                })
                .collect(),
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Vector, datom),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        Datom::Vector(self.iter().map(Datomic::datomize).collect())
    }
}

// ---------------------------------------------------------------------------
// Datomic for BTreeMap<K, V>
// ---------------------------------------------------------------------------

impl<K: Datomic + Ord + Clone, V: Datomic> Datomic for BTreeMap<K, V> {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Map(pairs) => {
                let mut map = BTreeMap::new();
                for (i, Pair(k_datom, v_datom)) in pairs.into_iter().enumerate() {
                    let key = K::incorporate(k_datom.clone())
                        .map_err(|f| prepend_path_to_fault(f, (i * 2) as Integer))?;
                    if map.contains_key(&key) {
                        return Err(Fault::Corporal(
                            vec![(i * 2) as Integer],
                            Problem::DuplicateKey(k_datom),
                        ));
                    }
                    let val = V::incorporate(v_datom)
                        .map_err(|f| prepend_path_to_fault(f, (i * 2 + 1) as Integer))?;
                    map.insert(key, val);
                }
                Ok(map)
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Map, datom),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        let pairs = self
            .iter()
            .map(|(k, v)| Pair(k.datomize(), v.datomize()))
            .collect();
        Datom::Map(pairs)
    }
}

// ---------------------------------------------------------------------------
// Datomic for Option<T>
// ---------------------------------------------------------------------------

impl<T: Datomic> Datomic for Option<T> {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) if s == "None" => Ok(None),
            Datom::Variant(head, sep, body) if head == "Some" => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match body {
                    Some(b) => T::incorporate(*b.clone()).map(Some),
                    None => Err(Fault::Corporal(
                        vec![],
                        Problem::Shape(Expected::Variant, datom),
                    )),
                }
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Variant, datom),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        match self {
            None => Datom::Bare("None".to_owned()),
            Some(val) => Datom::Variant(
                "Some".to_owned(),
                Separator::Period,
                Some(Box::new(val.datomize())),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Datomic for Result<T, E>
// ---------------------------------------------------------------------------

impl<T: Datomic, E: Datomic> Datomic for Result<T, E> {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Variant(head, sep, body) => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match (head.as_str(), body) {
                    ("Ok", Some(b)) => T::incorporate(*b.clone()).map(Ok),
                    ("Err", Some(b)) => E::incorporate(*b.clone()).map(Err),
                    _ => Err(Fault::Corporal(
                        vec![],
                        Problem::UnknownVariant(head.clone()),
                    )),
                }
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Variant, datom),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        match self {
            Ok(val) => Datom::Variant(
                "Ok".to_owned(),
                Separator::Period,
                Some(Box::new(val.datomize())),
            ),
            Err(err) => Datom::Variant(
                "Err".to_owned(),
                Separator::Period,
                Some(Box::new(err.datomize())),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: prepend a path index to a fault
// ---------------------------------------------------------------------------

fn prepend_path_to_fault(fault: Fault, index: Integer) -> Fault {
    match fault {
        Fault::Structural(f) => Fault::Structural(f),
        Fault::Conceptual(mut path, problem) => {
            path.insert(0, index);
            Fault::Conceptual(path, problem)
        }
        Fault::Corporal(mut path, problem) => {
            path.insert(0, index);
            Fault::Corporal(path, problem)
        }
    }
}
