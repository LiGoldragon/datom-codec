//! Datomic: positional typed data over Protos Protoform.
//!
//! The datom dialect: Concept layer between Protoform and Corporal.

use std::collections::BTreeMap;
use std::fmt;

pub use protos::{
    Actualizable, Boolean, Boundary, Corporal, Decimal, Delineation, Embodied, Enclosure, Extent,
    Integer, Path, Pathed, Potential, Printing, Protoform, Protosizable, Separator, Situated,
    Situating, Structural, Symbol, Text,
};

// ---------------------------------------------------------------------------
// Datom: the concept type of the datom dialect
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum Datom {
    Variant(Symbol, Separator, Option<Box<Datom>>),
    Struct(Vec<Datom>),
    Vector(Vec<Datom>),
    Map(Vec<Pair>),
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

#[derive(Clone, PartialEq, Eq)]
pub struct Pair(pub Datom, pub Datom);

impl fmt::Debug for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Pair").field(&self.0).field(&self.1).finish()
    }
}

/// Meaning: today, parenthesized text lands as plain.
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

impl From<protos::Fault> for Fault {
    fn from(f: protos::Fault) -> Self {
        Fault::Structural(f)
    }
}

impl Pathed for Fault {
    fn path(&self) -> &[Integer] {
        match self {
            Fault::Structural(_) => &[],
            Fault::Conceptual(path, _) | Fault::Corporal(path, _) => path,
        }
    }
}

/// Prepending a path index to a fault.
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
            Fault::Corporal(mut path, problem) => {
                path.insert(0, index);
                Fault::Corporal(path, problem)
            }
        }
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
                Protoform::Headed(protos::Head::Bare(head.clone()), *sep, Box::new(body_pf))
            }
            Datom::Struct(fields) => Protoform::Enclosed(
                Enclosure::Braced,
                fields.iter().map(Protosizable::protosize).collect(),
            ),
            Datom::Vector(items) => Protoform::Enclosed(
                Enclosure::Bracketed,
                items.iter().map(Protosizable::protosize).collect(),
            ),
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
// Conceptual<Datom> for Protoform and Delineation
// ---------------------------------------------------------------------------

trait Conceiving {
    fn conceive_at(&self, path: &[Integer]) -> Result<Datom, Fault>;
}

impl Conceiving for Protoform {
    fn conceive_at(&self, path: &[Integer]) -> Result<Datom, Fault> {
        match self {
            Protoform::Headed(head, sep, body) => {
                let symbol = match head {
                    protos::Head::Bare(s) => s.clone(),
                    protos::Head::Qualified(_, _) => {
                        return Err(Fault::Conceptual(
                            path.to_vec(),
                            Problem::Shape(Expected::Variant, Datom::Bare("Qualified".to_owned())),
                        ));
                    }
                };
                let body_path: Path = path.iter().copied().chain(std::iter::once(0)).collect();
                let body_datom = body.conceive_at(&body_path)?;
                Ok(Datom::Variant(symbol, *sep, Some(Box::new(body_datom))))
            }
            Protoform::Enclosed(enclosure, children) => match enclosure {
                Enclosure::Braced => {
                    let mut fields = Vec::with_capacity(children.len());
                    for (i, child) in children.iter().enumerate() {
                        let cp: Path = path
                            .iter()
                            .copied()
                            .chain(std::iter::once(i as Integer))
                            .collect();
                        fields.push(child.conceive_at(&cp)?);
                    }
                    Ok(Datom::Struct(fields))
                }
                Enclosure::Bracketed => {
                    let mut items = Vec::with_capacity(children.len());
                    for (i, child) in children.iter().enumerate() {
                        let cp: Path = path
                            .iter()
                            .copied()
                            .chain(std::iter::once(i as Integer))
                            .collect();
                        items.push(child.conceive_at(&cp)?);
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
                        let kp: Path = path
                            .iter()
                            .copied()
                            .chain(std::iter::once(ki as Integer))
                            .collect();
                        let vp: Path = path
                            .iter()
                            .copied()
                            .chain(std::iter::once(vi as Integer))
                            .collect();
                        pairs.push(Pair(chunk[0].conceive_at(&kp)?, chunk[1].conceive_at(&vp)?));
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
            Protoform::Qualified(_, _) => Err(Fault::Conceptual(
                path.to_vec(),
                Problem::Shape(Expected::Bare, Datom::Bare("Qualified".to_owned())),
            )),
        }
    }
}

impl protos::Conceptual<Datom> for Protoform {
    type Fault = Fault;
    fn conceive(&self) -> Result<Datom, Fault> {
        self.conceive_at(&[])
    }
}

impl protos::Conceptual<Datom> for Delineation {
    type Fault = Fault;
    fn conceive(&self) -> Result<Datom, Fault> {
        match self.protoforms.as_slice() {
            [pf] => pf.conceive_at(&[]),
            _ => Err(Fault::Conceptual(vec![], Problem::OneValue)),
        }
    }
}

// ---------------------------------------------------------------------------
// Datomic: the corporal kind of the datom dialect
// Datomic = Corporal<Datom, Fault = datomic::Fault> + datomize
// ---------------------------------------------------------------------------

pub trait Datomic: Corporal<Datom, Fault = Fault> {
    fn datomize(&self) -> Datom;
}

/// Provided for every Datomic: datomize -> protosize -> print.
pub trait Textualizable {
    fn textualize(&self) -> Text;
}

impl<T: Datomic> Textualizable for T {
    fn textualize(&self) -> Text {
        self.datomize().protosize().print()
    }
}

// ---------------------------------------------------------------------------
// Helpers: bare-string rule (as traits)
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

trait BareSafety {
    fn is_bare_safe(&self) -> bool;
}

impl BareSafety for str {
    fn is_bare_safe(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        let delimiters = [
            '{', '}', '[', ']', '\u{00AB}', '\u{00BB}', '<', '>', '\u{201C}', '\u{201D}', '(', ')',
            ';',
        ];
        for c in self.chars() {
            if c.is_whitespace() || delimiters.contains(&c) {
                return false;
            }
        }
        let first = self.chars().next().unwrap();
        let last = self.chars().next_back().unwrap();
        if matches!(first, '.' | '!' | ':') || matches!(last, '.' | '!' | ':') {
            return false;
        }
        let mut prev_sep = false;
        for c in self.chars() {
            let is_sep = matches!(c, '.' | '!' | ':');
            if is_sep && prev_sep {
                return false;
            }
            prev_sep = is_sep;
        }
        if let Ok(d) = self.to_owned().delineate() {
            if d.protoforms.len() == 1 {
                use protos::Conceptual;
                if let Ok(datom) = d.protoforms[0].conceive() {
                    if datom.is_all_bare_chain() {
                        return datom.rejoin_chain() == self;
                    }
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
            return Err(Fault::Corporal(vec![], Problem::Value(self.to_owned())));
        }
        let digits = if let Some(rest) = self.strip_prefix('-') {
            if rest.is_empty() {
                return Err(Fault::Corporal(vec![], Problem::Value(self.to_owned())));
            }
            rest
        } else {
            self
        };
        if self.starts_with('+') {
            return Err(Fault::Corporal(vec![], Problem::Value(self.to_owned())));
        }
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(Fault::Corporal(vec![], Problem::Value(self.to_owned())));
        }
        if digits.len() > 1 && digits.starts_with('0') {
            return Err(Fault::Corporal(vec![], Problem::Value(self.to_owned())));
        }
        if self == "-0" {
            return Err(Fault::Corporal(vec![], Problem::Value(self.to_owned())));
        }
        self.parse::<Integer>()
            .map_err(|_| Fault::Corporal(vec![], Problem::Value(self.to_owned())))
    }
}

// ---------------------------------------------------------------------------
// Datomic for scalars
// ---------------------------------------------------------------------------

impl Corporal<Datom> for Integer {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) => s.parse_integer(),
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Integer, datom),
            )),
        }
    }
}
impl Datomic for Integer {
    fn datomize(&self) -> Datom {
        Datom::Bare(self.to_string())
    }
}

impl Corporal<Datom> for Boolean {
    type Fault = Fault;
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
}
impl Datomic for Boolean {
    fn datomize(&self) -> Datom {
        Datom::Bare(if *self { "True" } else { "False" }.to_owned())
    }
}

impl Corporal<Datom> for Decimal {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        let s = match &datom {
            Datom::Bare(s) => s.clone(),
            _ if datom.is_all_bare_chain() => datom.rejoin_chain(),
            _ => {
                return Err(Fault::Corporal(
                    vec![],
                    Problem::Shape(Expected::Decimal, datom),
                ));
            }
        };
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
}
impl Datomic for Decimal {
    fn datomize(&self) -> Datom {
        let s = format!("{self}");
        if s.contains('.') {
            let trimmed = s.trim_end_matches('0');
            let result = if trimmed.ends_with('.') {
                format!("{trimmed}0")
            } else {
                trimmed.to_owned()
            };
            Datom::Bare(result)
        } else {
            Datom::Bare(format!("{s}.0"))
        }
    }
}

impl Corporal<Datom> for Text {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Text(content) => Ok(content),
            Datom::Bare(symbol) => Ok(symbol),
            ref d if d.is_all_bare_chain() => Ok(d.rejoin_chain()),
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Text, datom),
            )),
        }
    }
}
impl Datomic for Text {
    fn datomize(&self) -> Datom {
        if self.is_bare_safe() {
            Datom::Bare(self.clone())
        } else {
            Datom::Text(self.clone())
        }
    }
}

impl Corporal<Datom> for Meaning {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Meaning(content) => Ok(Meaning::Plain(content)),
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Meaning, datom),
            )),
        }
    }
}
impl Datomic for Meaning {
    fn datomize(&self) -> Datom {
        match self {
            Meaning::Plain(c) => Datom::Meaning(c.clone()),
        }
    }
}

// ---------------------------------------------------------------------------
// Datomic for containers
// ---------------------------------------------------------------------------

impl<T: Datomic> Corporal<Datom> for Vec<T> {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Vector(items) => items
                .into_iter()
                .enumerate()
                .map(|(i, item)| T::incorporate(item).map_err(|f| f.prepend(i as Integer)))
                .collect(),
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Vector, datom),
            )),
        }
    }
}
impl<T: Datomic> Datomic for Vec<T> {
    fn datomize(&self) -> Datom {
        Datom::Vector(self.iter().map(Datomic::datomize).collect())
    }
}

impl<K: Datomic + Ord + Clone, V: Datomic> Corporal<Datom> for BTreeMap<K, V> {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Map(pairs) => {
                let mut map = BTreeMap::new();
                for (i, Pair(k_datom, v_datom)) in pairs.into_iter().enumerate() {
                    let key = K::incorporate(k_datom.clone())
                        .map_err(|f| f.prepend((i * 2) as Integer))?;
                    if map.contains_key(&key) {
                        return Err(Fault::Corporal(
                            vec![(i * 2) as Integer],
                            Problem::DuplicateKey(k_datom),
                        ));
                    }
                    let val =
                        V::incorporate(v_datom).map_err(|f| f.prepend((i * 2 + 1) as Integer))?;
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
}
impl<K: Datomic + Ord + Clone, V: Datomic> Datomic for BTreeMap<K, V> {
    fn datomize(&self) -> Datom {
        Datom::Map(
            self.iter()
                .map(|(k, v)| Pair(k.datomize(), v.datomize()))
                .collect(),
        )
    }
}

impl<T: Datomic> Corporal<Datom> for Option<T> {
    type Fault = Fault;
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
}
impl<T: Datomic> Datomic for Option<T> {
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

impl<T: Datomic, E: Datomic> Corporal<Datom> for Result<T, E> {
    type Fault = Fault;
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
}
impl<T: Datomic, E: Datomic> Datomic for Result<T, E> {
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
// Datomic for fault types — every fault printable as datom
// ---------------------------------------------------------------------------

impl Corporal<Datom> for Expected {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) => match s.as_str() {
                "Variant" => Ok(Self::Variant),
                "Struct" => Ok(Self::Struct),
                "Vector" => Ok(Self::Vector),
                "Map" => Ok(Self::Map),
                "Text" => Ok(Self::Text),
                "Meaning" => Ok(Self::Meaning),
                "Integer" => Ok(Self::Integer),
                "Decimal" => Ok(Self::Decimal),
                "Boolean" => Ok(Self::Boolean),
                "Bare" => Ok(Self::Bare),
                _ => Err(Fault::Corporal(vec![], Problem::UnknownVariant(s.clone()))),
            },
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Bare, datom),
            )),
        }
    }
}
impl Datomic for Expected {
    fn datomize(&self) -> Datom {
        Datom::Bare(
            match self {
                Self::Variant => "Variant",
                Self::Struct => "Struct",
                Self::Vector => "Vector",
                Self::Map => "Map",
                Self::Text => "Text",
                Self::Meaning => "Meaning",
                Self::Integer => "Integer",
                Self::Decimal => "Decimal",
                Self::Boolean => "Boolean",
                Self::Bare => "Bare",
            }
            .to_owned(),
        )
    }
}

impl Corporal<Datom> for Problem {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Variant(head, sep, body) => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match (head.as_str(), body) {
                    ("Shape", Some(b)) => match b.as_ref() {
                        Datom::Struct(fields) if fields.len() == 2 => {
                            let expected = Expected::incorporate(fields[0].clone())?;
                            Ok(Problem::Shape(expected, fields[1].clone()))
                        }
                        _ => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Struct, *b.clone()),
                        )),
                    },
                    ("Arity", Some(b)) => match b.as_ref() {
                        Datom::Struct(fields) if fields.len() == 2 => {
                            let expected = Integer::incorporate(fields[0].clone())?;
                            let actual = Integer::incorporate(fields[1].clone())?;
                            Ok(Problem::Arity(expected, actual))
                        }
                        _ => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Struct, *b.clone()),
                        )),
                    },
                    ("UnknownVariant", Some(b)) => {
                        Text::incorporate(*b.clone()).map(Problem::UnknownVariant)
                    }
                    ("Value", Some(b)) => Text::incorporate(*b.clone()).map(Problem::Value),
                    ("DuplicateKey", Some(b)) => Ok(Problem::DuplicateKey(*b.clone())),
                    ("Separator", Some(b)) => {
                        Separator::incorporate(*b.clone()).map(Problem::Separator)
                    }
                    _ => Err(Fault::Corporal(
                        vec![],
                        Problem::UnknownVariant(head.clone()),
                    )),
                }
            }
            Datom::Bare(s) => match s.as_str() {
                "Pairing" => Ok(Problem::Pairing),
                "OneValue" => Ok(Problem::OneValue),
                _ => Err(Fault::Corporal(vec![], Problem::UnknownVariant(s.clone()))),
            },
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Variant, datom),
            )),
        }
    }
}
impl Datomic for Problem {
    fn datomize(&self) -> Datom {
        match self {
            Problem::Shape(expected, datom) => Datom::Variant(
                "Shape".to_owned(),
                Separator::Period,
                Some(Box::new(Datom::Struct(vec![
                    expected.datomize(),
                    datom.clone(),
                ]))),
            ),
            Problem::Arity(expected, actual) => Datom::Variant(
                "Arity".to_owned(),
                Separator::Period,
                Some(Box::new(Datom::Struct(vec![
                    expected.datomize(),
                    actual.datomize(),
                ]))),
            ),
            Problem::UnknownVariant(s) => Datom::Variant(
                "UnknownVariant".to_owned(),
                Separator::Period,
                Some(Box::new(s.datomize())),
            ),
            Problem::Separator(sep) => Datom::Variant(
                "Separator".to_owned(),
                Separator::Period,
                Some(Box::new(sep.datomize())),
            ),
            Problem::Value(v) => Datom::Variant(
                "Value".to_owned(),
                Separator::Period,
                Some(Box::new(v.datomize())),
            ),
            Problem::Pairing => Datom::Bare("Pairing".to_owned()),
            Problem::DuplicateKey(d) => Datom::Variant(
                "DuplicateKey".to_owned(),
                Separator::Period,
                Some(Box::new(d.clone())),
            ),
            Problem::OneValue => Datom::Bare("OneValue".to_owned()),
        }
    }
}

impl Corporal<Datom> for Fault {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Variant(head, sep, body) => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match (head.as_str(), body) {
                    ("Structural", Some(b)) => {
                        protos::Fault::incorporate(*b.clone()).map(Fault::Structural)
                    }
                    ("Conceptual", Some(b)) => match b.as_ref() {
                        Datom::Struct(fields) if fields.len() == 2 => {
                            let path = Vec::<Integer>::incorporate(fields[0].clone())?;
                            let problem = Problem::incorporate(fields[1].clone())?;
                            Ok(Fault::Conceptual(path, problem))
                        }
                        _ => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Struct, *b.clone()),
                        )),
                    },
                    ("Corporal", Some(b)) => match b.as_ref() {
                        Datom::Struct(fields) if fields.len() == 2 => {
                            let path = Vec::<Integer>::incorporate(fields[0].clone())?;
                            let problem = Problem::incorporate(fields[1].clone())?;
                            Ok(Fault::Corporal(path, problem))
                        }
                        _ => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Struct, *b.clone()),
                        )),
                    },
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
}
impl Datomic for Fault {
    fn datomize(&self) -> Datom {
        match self {
            Fault::Structural(f) => Datom::Variant(
                "Structural".to_owned(),
                Separator::Period,
                Some(Box::new(f.datomize())),
            ),
            Fault::Conceptual(path, problem) => Datom::Variant(
                "Conceptual".to_owned(),
                Separator::Period,
                Some(Box::new(Datom::Struct(vec![
                    path.datomize(),
                    problem.datomize(),
                ]))),
            ),
            Fault::Corporal(path, problem) => Datom::Variant(
                "Corporal".to_owned(),
                Separator::Period,
                Some(Box::new(Datom::Struct(vec![
                    path.datomize(),
                    problem.datomize(),
                ]))),
            ),
        }
    }
}

// Datomic for Datom itself (identity)

// ---------------------------------------------------------------------------
// Datomic for protos structural types (Separator, Enclosure, Boundary, Extent, Problem, Fault)
// ---------------------------------------------------------------------------

impl Corporal<Datom> for Separator {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) => match s.as_str() {
                "Period" => Ok(Separator::Period),
                "Exclamation" => Ok(Separator::Exclamation),
                "Colon" => Ok(Separator::Colon),
                _ => Err(Fault::Corporal(vec![], Problem::UnknownVariant(s.clone()))),
            },
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Bare, datom),
            )),
        }
    }
}
impl Datomic for Separator {
    fn datomize(&self) -> Datom {
        Datom::Bare(
            match self {
                Self::Period => "Period",
                Self::Exclamation => "Exclamation",
                Self::Colon => "Colon",
            }
            .to_owned(),
        )
    }
}

impl Corporal<Datom> for Enclosure {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) => match s.as_str() {
                "Braced" => Ok(Enclosure::Braced),
                "Bracketed" => Ok(Enclosure::Bracketed),
                "Guillemets" => Ok(Enclosure::Guillemets),
                "Angled" => Ok(Enclosure::Angled),
                _ => Err(Fault::Corporal(vec![], Problem::UnknownVariant(s.clone()))),
            },
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Bare, datom),
            )),
        }
    }
}
impl Datomic for Enclosure {
    fn datomize(&self) -> Datom {
        Datom::Bare(
            match self {
                Self::Braced => "Braced",
                Self::Bracketed => "Bracketed",
                Self::Guillemets => "Guillemets",
                Self::Angled => "Angled",
            }
            .to_owned(),
        )
    }
}

impl Corporal<Datom> for Boundary {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) => match s.as_str() {
                "CurlyQuotes" => Ok(Boundary::CurlyQuotes),
                "Parentheses" => Ok(Boundary::Parentheses),
                _ => Err(Fault::Corporal(vec![], Problem::UnknownVariant(s.clone()))),
            },
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Bare, datom),
            )),
        }
    }
}
impl Datomic for Boundary {
    fn datomize(&self) -> Datom {
        Datom::Bare(
            match self {
                Self::CurlyQuotes => "CurlyQuotes",
                Self::Parentheses => "Parentheses",
            }
            .to_owned(),
        )
    }
}

impl Corporal<Datom> for Extent {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Struct(fields) => {
                if fields.len() != 2 {
                    return Err(Fault::Corporal(
                        vec![],
                        Problem::Arity(2, fields.len() as Integer),
                    ));
                }
                let start = Integer::incorporate(fields[0].clone())?;
                let end = Integer::incorporate(fields[1].clone())?;
                Ok(Extent(start, end))
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Struct, datom),
            )),
        }
    }
}
impl Datomic for Extent {
    fn datomize(&self) -> Datom {
        Datom::Struct(vec![self.0.datomize(), self.1.datomize()])
    }
}

impl Corporal<Datom> for protos::Problem {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Variant(head, sep, body) => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match (head.as_str(), body) {
                    ("Unclosed", Some(b)) => {
                        Enclosure::incorporate(*b.clone()).map(protos::Problem::Unclosed)
                    }
                    ("UnclosedBoundary", Some(b)) => {
                        Boundary::incorporate(*b.clone()).map(protos::Problem::UnclosedBoundary)
                    }
                    _ => Err(Fault::Corporal(
                        vec![],
                        Problem::UnknownVariant(head.clone()),
                    )),
                }
            }
            Datom::Bare(s) => match s.as_str() {
                "Unopened" => Ok(protos::Problem::Unopened),
                "MissingBody" => Ok(protos::Problem::MissingBody),
                "MissingHead" => Ok(protos::Problem::MissingHead),
                "EmptyInput" => Ok(protos::Problem::EmptyInput),
                _ => Err(Fault::Corporal(vec![], Problem::UnknownVariant(s.clone()))),
            },
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Variant, datom),
            )),
        }
    }
}
impl Datomic for protos::Problem {
    fn datomize(&self) -> Datom {
        match self {
            protos::Problem::Unclosed(e) => Datom::Variant(
                "Unclosed".to_owned(),
                Separator::Period,
                Some(Box::new(e.datomize())),
            ),
            protos::Problem::UnclosedBoundary(b) => Datom::Variant(
                "UnclosedBoundary".to_owned(),
                Separator::Period,
                Some(Box::new(b.datomize())),
            ),
            protos::Problem::Unopened => Datom::Bare("Unopened".to_owned()),
            protos::Problem::MissingBody => Datom::Bare("MissingBody".to_owned()),
            protos::Problem::MissingHead => Datom::Bare("MissingHead".to_owned()),
            protos::Problem::EmptyInput => Datom::Bare("EmptyInput".to_owned()),
        }
    }
}

impl Corporal<Datom> for protos::Fault {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Struct(fields) => {
                if fields.len() != 2 {
                    return Err(Fault::Corporal(
                        vec![],
                        Problem::Arity(2, fields.len() as Integer),
                    ));
                }
                let extent = Extent::incorporate(fields[0].clone())?;
                let problem = protos::Problem::incorporate(fields[1].clone())?;
                Ok(protos::Fault { extent, problem })
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Struct, datom),
            )),
        }
    }
}
impl Datomic for protos::Fault {
    fn datomize(&self) -> Datom {
        Datom::Struct(vec![self.extent.datomize(), self.problem.datomize()])
    }
}
impl Corporal<Datom> for Datom {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        Ok(datom)
    }
}
impl Datomic for Datom {
    fn datomize(&self) -> Datom {
        self.clone()
    }
}
