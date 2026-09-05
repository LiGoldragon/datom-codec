//! Datomic: positional typed data over Protos.
//!
//! The datom dialect carries data, strictly typed. Schema-driven and
//! positional: the reader walks the expected type, writing is the exact
//! reverse projection.

use std::convert::Infallible;

pub use protos::{
    Actualizable, Boolean, Boundary, Conceivable, Decimal, Delineation, Enclosure, Extent, Head,
    Incorporable, Integer, Path, Pathed, Potential, Protoform, Protosizable, Separator, Situated,
    Situating, Symbol, Text, Textualizable,
};

// ---------------------------------------------------------------------------
// Datom: the concept type of the datom dialect
// ---------------------------------------------------------------------------

/// A datom value: the concept layer between protoform and corporate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Datom {
    /// A headed structure with a body (dot separator implied).
    Variant(Head, Box<Datom>),
    /// Positional fields between braces.
    Struct(Vec<Datom>),
    /// Homogeneous elements between brackets.
    Vector(Vec<Datom>),
    /// A plain string (curly-quoted or bare).
    Text(Text),
    /// A parenthesized meaning (today a plain string).
    Meaning(Text),
    /// A bare symbol: the position decides its meaning.
    Bare(Symbol),
}

/// The meaning type (today a plain string, structured meaning is future).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Meaning {
    /// A plain text meaning.
    Plain(Text),
}

// ---------------------------------------------------------------------------
// Fault types
// ---------------------------------------------------------------------------

/// What form was expected at a position.
#[derive(Clone, Debug, PartialEq, Eq)]
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

/// A conceptual or corporate problem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// Wrong structural form.
    Shape(Expected, Datom),
    /// Wrong number of fields.
    Arity(Integer, Integer),
    /// Unknown variant name.
    UnknownVariant(Symbol),
    /// Wrong separator (datom uses only the dot).
    Separator(Separator),
    /// Invalid scalar value.
    Value(Text),
    /// Expected exactly one top-level value.
    OneValue,
}

/// A datom fault at a layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A protos structural fault.
    Structural(protos::Fault),
    /// A conceptual fault (protoform to datom).
    Conceptual(Path, Problem),
    /// A corporate fault (datom to Rust value).
    Corporate(Path, Problem),
}

/// The kind whose capability prepends an index to a fault's path.
pub trait Prepending {
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

// ---------------------------------------------------------------------------
// Protosizing: Datom -> Protoform (concept to protoform, cannot fault)
// ---------------------------------------------------------------------------

/// The kind whose capability converts a datom to a protoform.
trait Protosizing {
    fn to_protoform(&self) -> Protoform;
}

impl Protosizing for Datom {
    fn to_protoform(&self) -> Protoform {
        match self {
            Datom::Variant(head, body) => Protoform::Headed(
                head.clone(),
                Separator::Period,
                Box::new(body.to_protoform()),
            ),
            Datom::Struct(fields) => {
                let mut children = Vec::with_capacity(fields.len());
                for field in fields {
                    children.push(field.to_protoform());
                }
                Protoform::Enclosed(Enclosure::Braced, children)
            }
            Datom::Vector(items) => {
                let mut children = Vec::with_capacity(items.len());
                for item in items {
                    children.push(item.to_protoform());
                }
                Protoform::Enclosed(Enclosure::Bracketed, children)
            }
            Datom::Text(content) => Protoform::Opaque(Boundary::CurlyQuotes, content.clone()),
            Datom::Meaning(content) => Protoform::Opaque(Boundary::Parentheses, content.clone()),
            Datom::Bare(symbol) => Protoform::Bare(Head::Bare(symbol.clone())),
        }
    }
}

impl protos::Protosizable for Datom {
    type Fault = Infallible;

    fn protosize(&self) -> Result<Delineation, Infallible> {
        // Compute situation by textualizing and re-delineating
        let pf = self.to_protoform();
        let text = pf.textualize();
        let delineation =
            <Text as protos::Protosizable>::protosize(&text).expect("canonical text delineates");
        Ok(delineation)
    }
}

impl protos::Textualizable for Datom {
    fn textualize(&self) -> Text {
        self.to_protoform().textualize()
    }
}

// ---------------------------------------------------------------------------
// Conceiving: Protoform -> Datom (descent, may fault)
// ---------------------------------------------------------------------------

/// The kind whose capability conceives a datom from a protoform at a path.
trait Conceiving {
    fn conceive_at(&self, path: &[Integer]) -> Result<Datom, Fault>;
}

impl Conceiving for Protoform {
    fn conceive_at(&self, path: &[Integer]) -> Result<Datom, Fault> {
        match self {
            Protoform::Headed(head, sep, body) => {
                if *sep == Separator::Period {
                    let mut body_path = path.to_vec();
                    body_path.push(0);
                    let body_datom = body.conceive_at(&body_path)?;
                    Ok(Datom::Variant(head.clone(), Box::new(body_datom)))
                } else {
                    // Non-dot separator: the headed protoform is text content
                    Ok(Datom::Bare(self.textualize()))
                }
            }
            Protoform::Enclosed(enclosure, children) => match enclosure {
                Enclosure::Braced => {
                    let mut fields = Vec::with_capacity(children.len());
                    for (i, child) in children.iter().enumerate() {
                        let mut cp = path.to_vec();
                        cp.push(i as Integer);
                        fields.push(child.conceive_at(&cp)?);
                    }
                    Ok(Datom::Struct(fields))
                }
                Enclosure::Bracketed => {
                    let mut items = Vec::with_capacity(children.len());
                    for (i, child) in children.iter().enumerate() {
                        let mut cp = path.to_vec();
                        cp.push(i as Integer);
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

    fn conceive(&self) -> Result<Datom, Fault> {
        self.conceive_at(&[0])
    }
}

impl protos::Conceivable<Datom> for Delineation {
    type Fault = Fault;

    fn conceive(&self) -> Result<Datom, Fault> {
        match self.protoforms.as_slice() {
            [pf] => pf.conceive_at(&[0]),
            _ => Err(Fault::Conceptual(vec![], Problem::OneValue)),
        }
    }
}

// ---------------------------------------------------------------------------
// Datomic: the kind every corporate type of the dialect bears
// ---------------------------------------------------------------------------

/// The kind every corporate type of the datom dialect bears.
pub trait Datomic: Sized + protos::Conceivable<Datom, Fault = Infallible> {
    /// Incorporate a corporate value from a datom.
    fn incorporate_from(datom: Datom) -> Result<Self, Fault>;

    /// Textualize through the chain: conceive, protosize, textualize.
    fn textualize(&self) -> Text {
        let datom: Datom = self.conceive().unwrap();
        datom.to_protoform().textualize()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The kind whose capability checks if a datom is a pure bare chain.
trait VariantChaining {
    fn is_all_bare_chain(&self) -> bool;
    fn rejoin_chain(&self) -> String;
}

impl VariantChaining for Datom {
    fn is_all_bare_chain(&self) -> bool {
        match self {
            Datom::Bare(_) => true,
            Datom::Variant(Head::Bare(_), body) => body.is_all_bare_chain(),
            _ => false,
        }
    }

    fn rejoin_chain(&self) -> String {
        use protos::Glyphing as _;
        match self {
            Datom::Bare(s) => s.clone(),
            Datom::Variant(Head::Bare(head), body) => {
                let sep_char = Separator::Period.glyph();
                let mut result = head.clone();
                result.push(sep_char);
                result.push_str(&body.rejoin_chain());
                result
            }
            _ => String::new(),
        }
    }
}

/// The kind whose capability checks if a string is safe to write bare.
trait BareSafety {
    fn is_bare_safe(&self) -> bool;
}

impl BareSafety for str {
    fn is_bare_safe(&self) -> bool {
        use protos::Identifying as _;
        use protos::Recognizing as _;

        if self.is_empty() {
            return false;
        }
        for c in self.chars() {
            if c.is_whitespace()
                || Enclosure::from_opener(c).is_some()
                || Enclosure::from_closer(c).is_some()
                || Boundary::from_opener(c).is_some()
                || Boundary::from_closer(c).is_some()
                || c == ';'
            {
                return false;
            }
        }
        // No leading/trailing separator
        let first = self.chars().next().unwrap();
        let last = self.chars().next_back().unwrap();
        if Separator::identify(first).is_some() || Separator::identify(last).is_some() {
            return false;
        }
        // No consecutive separators
        let mut prev_sep = false;
        for c in self.chars() {
            let is_sep = Separator::identify(c).is_some();
            if is_sep && prev_sep {
                return false;
            }
            prev_sep = is_sep;
        }
        // Round-trip check: delineate and conceive, verify it comes back as a bare chain
        if let Ok(d) = <Text as protos::Protosizable>::protosize(&self.to_owned()) {
            if d.protoforms.len() == 1 {
                if let Ok(datom) = d.protoforms[0].conceive_at(&[0]) {
                    if datom.is_all_bare_chain() {
                        return datom.rejoin_chain() == self;
                    }
                }
            }
        }
        false
    }
}

/// The kind whose capability parses an integer from text.
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
        } else {
            self
        };
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

/// The kind whose capability parses a decimal from text.
trait DecimalParsing {
    fn parse_decimal(&self) -> Result<Decimal, Fault>;
}

impl DecimalParsing for str {
    fn parse_decimal(&self) -> Result<Decimal, Fault> {
        // Must contain a point
        if !self.contains('.') {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        // Must have digits on both sides of the point
        let parts: Vec<&str> = self.splitn(2, '.').collect();
        if parts.len() != 2 {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        let integer_part = if let Some(rest) = parts[0].strip_prefix('-') {
            rest
        } else {
            parts[0]
        };
        if integer_part.is_empty() || parts[1].is_empty() {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        if !integer_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        if !parts[1].chars().all(|c| c.is_ascii_digit()) {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        // No leading zero except "0." or "-0."
        if integer_part.len() > 1 && integer_part.starts_with('0') {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        // No exponent
        if self.contains('e') || self.contains('E') {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        let value: Decimal = self
            .parse()
            .map_err(|_| Fault::Corporate(vec![], Problem::Value(self.to_owned())))?;
        if !value.is_finite() {
            return Err(Fault::Corporate(vec![], Problem::Value(self.to_owned())));
        }
        Ok(value)
    }
}

/// The kind whose capability prints a decimal in shortest round-trip form.
trait DecimalPrinting {
    fn print_decimal(&self) -> String;
}

impl DecimalPrinting for Decimal {
    fn print_decimal(&self) -> String {
        // Use ryu for shortest round-trip representation
        let s = format!("{self}");
        if s.contains('.') {
            let trimmed = s.trim_end_matches('0');
            if trimmed.ends_with('.') {
                format!("{trimmed}0")
            } else {
                trimmed.to_owned()
            }
        } else {
            format!("{s}.0")
        }
    }
}

// ---------------------------------------------------------------------------
// Datomic implementations: scalars
// ---------------------------------------------------------------------------

/// A macro that implements the three traits for a scalar Datomic type.
macro_rules! impl_datomic_scalar {
    ($type:ty, $conceive:expr, $incorporate:expr) => {
        impl protos::Conceivable<Datom> for $type {
            type Fault = Infallible;

            fn conceive(&self) -> Result<Datom, Infallible> {
                #[allow(clippy::redundant_closure_call)]
                Ok($conceive(self))
            }
        }

        impl Datomic for $type {
            fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
                #[allow(clippy::redundant_closure_call)]
                $incorporate(datom)
            }
        }

        impl protos::Incorporable<$type> for Datom {
            type Fault = Fault;

            fn incorporate(self) -> Result<$type, Fault> {
                <$type as Datomic>::incorporate_from(self)
            }
        }
    };
}

impl_datomic_scalar!(
    Integer,
    |v: &Integer| Datom::Bare(v.to_string()),
    |datom: Datom| {
        match datom {
            Datom::Bare(s) => s.parse_integer(),
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Integer, other),
            )),
        }
    }
);

impl_datomic_scalar!(
    Boolean,
    |v: &Boolean| Datom::Bare(if *v { "True" } else { "False" }.to_owned()),
    |datom: Datom| {
        match datom {
            Datom::Bare(s) if s == "True" => Ok(true),
            Datom::Bare(s) if s == "False" => Ok(false),
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Boolean, other),
            )),
        }
    }
);

impl_datomic_scalar!(
    Decimal,
    |v: &Decimal| Datom::Bare(v.print_decimal()),
    |datom: Datom| {
        if !datom.is_all_bare_chain() {
            return Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Decimal, datom),
            ));
        }
        let s = datom.rejoin_chain();
        s.parse_decimal()
    }
);

impl_datomic_scalar!(
    Text,
    |v: &Text| {
        if v.is_bare_safe() {
            Datom::Bare(v.clone())
        } else {
            Datom::Text(v.clone())
        }
    },
    |datom: Datom| {
        match datom {
            Datom::Text(content) => Ok(content),
            Datom::Bare(symbol) => Ok(symbol),
            ref d if d.is_all_bare_chain() => Ok(d.rejoin_chain()),
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Text, other),
            )),
        }
    }
);

impl_datomic_scalar!(
    Meaning,
    |v: &Meaning| {
        match v {
            Meaning::Plain(c) => Datom::Meaning(c.clone()),
        }
    },
    |datom: Datom| {
        match datom {
            Datom::Meaning(content) => Ok(Meaning::Plain(content)),
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Meaning, other),
            )),
        }
    }
);

// ---------------------------------------------------------------------------
// Datomic implementations: containers
// ---------------------------------------------------------------------------

impl<T: Datomic> protos::Conceivable<Datom> for Vec<T> {
    type Fault = Infallible;

    fn conceive(&self) -> Result<Datom, Infallible> {
        let mut items = Vec::with_capacity(self.len());
        for item in self {
            items.push(item.conceive().unwrap());
        }
        Ok(Datom::Vector(items))
    }
}

impl<T: Datomic> Datomic for Vec<T> {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Vector(items) => {
                let mut result = Vec::with_capacity(items.len());
                for (i, item) in items.into_iter().enumerate() {
                    result.push(T::incorporate_from(item).map_err(|f| f.prepend(i as Integer))?);
                }
                Ok(result)
            }
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Vector, other),
            )),
        }
    }
}

impl<T: Datomic> protos::Incorporable<Vec<T>> for Datom {
    type Fault = Fault;

    fn incorporate(self) -> Result<Vec<T>, Fault> {
        Vec::<T>::incorporate_from(self)
    }
}

impl<T: Datomic> protos::Conceivable<Datom> for Option<T> {
    type Fault = Infallible;

    fn conceive(&self) -> Result<Datom, Infallible> {
        match self {
            None => Ok(Datom::Bare("None".to_owned())),
            Some(val) => Ok(Datom::Variant(
                Head::Bare("Some".to_owned()),
                Box::new(val.conceive().unwrap()),
            )),
        }
    }
}

impl<T: Datomic> Datomic for Option<T> {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) if s == "None" => Ok(None),
            Datom::Variant(Head::Bare(name), _) if name == "Some" => {
                let Datom::Variant(_, body) = datom else {
                    unreachable!()
                };
                T::incorporate_from(*body)
                    .map(Some)
                    .map_err(|f| f.prepend(0))
            }
            _ => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Variant, datom),
            )),
        }
    }
}

impl<T: Datomic> protos::Incorporable<Option<T>> for Datom {
    type Fault = Fault;

    fn incorporate(self) -> Result<Option<T>, Fault> {
        Option::<T>::incorporate_from(self)
    }
}

impl<T: Datomic, E: Datomic> protos::Conceivable<Datom> for Result<T, E> {
    type Fault = Infallible;

    fn conceive(&self) -> Result<Datom, Infallible> {
        match self {
            Ok(val) => Ok(Datom::Variant(
                Head::Bare("Ok".to_owned()),
                Box::new(val.conceive().unwrap()),
            )),
            Err(err) => Ok(Datom::Variant(
                Head::Bare("Err".to_owned()),
                Box::new(err.conceive().unwrap()),
            )),
        }
    }
}

impl<T: Datomic, E: Datomic> Datomic for Result<T, E> {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(Head::Bare(name), body) => match name.as_str() {
                "Ok" => T::incorporate_from(*body).map(Ok).map_err(|f| f.prepend(0)),
                "Err" => E::incorporate_from(*body)
                    .map(Err)
                    .map_err(|f| f.prepend(0)),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(name))),
            },
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Variant, other),
            )),
        }
    }
}

impl<T: Datomic, E: Datomic> protos::Incorporable<Result<T, E>> for Datom {
    type Fault = Fault;

    fn incorporate(self) -> Result<Result<T, E>, Fault> {
        Result::<T, E>::incorporate_from(self)
    }
}

// ---------------------------------------------------------------------------
// Datomic implementations: fault types (self-describing)
// ---------------------------------------------------------------------------

impl_datomic_scalar!(
    Expected,
    |v: &Expected| {
        Datom::Bare(
            match v {
                Expected::Variant => "Variant",
                Expected::Struct => "Struct",
                Expected::Vector => "Vector",
                Expected::Text => "Text",
                Expected::Meaning => "Meaning",
                Expected::Integer => "Integer",
                Expected::Decimal => "Decimal",
                Expected::Boolean => "Boolean",
                Expected::Bare => "Bare",
            }
            .to_owned(),
        )
    },
    |datom: Datom| {
        match datom {
            Datom::Bare(s) => match s.as_str() {
                "Variant" => Ok(Expected::Variant),
                "Struct" => Ok(Expected::Struct),
                "Vector" => Ok(Expected::Vector),
                "Text" => Ok(Expected::Text),
                "Meaning" => Ok(Expected::Meaning),
                "Integer" => Ok(Expected::Integer),
                "Decimal" => Ok(Expected::Decimal),
                "Boolean" => Ok(Expected::Boolean),
                "Bare" => Ok(Expected::Bare),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Bare, other),
            )),
        }
    }
);

impl protos::Conceivable<Datom> for Problem {
    type Fault = Infallible;

    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(match self {
            Problem::Shape(expected, datom) => Datom::Variant(
                Head::Bare("Shape".to_owned()),
                Box::new(Datom::Struct(vec![expected.conceive()?, datom.clone()])),
            ),
            Problem::Arity(expected, actual) => Datom::Variant(
                Head::Bare("Arity".to_owned()),
                Box::new(Datom::Struct(vec![
                    expected.conceive()?,
                    actual.conceive()?,
                ])),
            ),
            Problem::UnknownVariant(s) => Datom::Variant(
                Head::Bare("UnknownVariant".to_owned()),
                Box::new(s.conceive()?),
            ),
            Problem::Separator(sep) => Datom::Variant(
                Head::Bare("Separator".to_owned()),
                Box::new(sep.conceive()?),
            ),
            Problem::Value(v) => {
                Datom::Variant(Head::Bare("Value".to_owned()), Box::new(v.conceive()?))
            }
            Problem::OneValue => Datom::Bare("OneValue".to_owned()),
        })
    }
}

impl Datomic for Problem {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(Head::Bare(name), body) => match (name.as_str(), *body) {
                ("Shape", Datom::Struct(fields)) if fields.len() == 2 => {
                    let mut it = fields.into_iter();
                    let expected =
                        Expected::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(0))?;
                    Ok(Problem::Shape(expected, it.next().unwrap()))
                }
                ("Arity", Datom::Struct(fields)) if fields.len() == 2 => {
                    let mut it = fields.into_iter();
                    let expected =
                        Integer::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(0))?;
                    let actual =
                        Integer::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(1))?;
                    Ok(Problem::Arity(expected, actual))
                }
                ("UnknownVariant", body) => {
                    Text::incorporate_from(body).map(Problem::UnknownVariant)
                }
                ("Value", body) => Text::incorporate_from(body).map(Problem::Value),
                ("Separator", body) => Separator::incorporate_from(body).map(Problem::Separator),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(name))),
            },
            Datom::Bare(s) => match s.as_str() {
                "OneValue" => Ok(Problem::OneValue),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Variant, other),
            )),
        }
    }
}

impl protos::Incorporable<Problem> for Datom {
    type Fault = Fault;

    fn incorporate(self) -> Result<Problem, Fault> {
        Problem::incorporate_from(self)
    }
}

impl protos::Conceivable<Datom> for Fault {
    type Fault = Infallible;

    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(match self {
            Fault::Structural(f) => {
                Datom::Variant(Head::Bare("Structural".to_owned()), Box::new(f.conceive()?))
            }
            Fault::Conceptual(path, problem) => Datom::Variant(
                Head::Bare("Conceptual".to_owned()),
                Box::new(Datom::Struct(vec![path.conceive()?, problem.conceive()?])),
            ),
            Fault::Corporate(path, problem) => Datom::Variant(
                Head::Bare("Corporate".to_owned()),
                Box::new(Datom::Struct(vec![path.conceive()?, problem.conceive()?])),
            ),
        })
    }
}

impl Datomic for Fault {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(Head::Bare(name), body) => match (name.as_str(), *body) {
                ("Structural", body) => {
                    protos::Fault::incorporate_from(body).map(Fault::Structural)
                }
                ("Conceptual", Datom::Struct(fields)) if fields.len() == 2 => {
                    let mut it = fields.into_iter();
                    let path = Vec::<Integer>::incorporate_from(it.next().unwrap())
                        .map_err(|f| f.prepend(0))?;
                    let problem =
                        Problem::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(1))?;
                    Ok(Fault::Conceptual(path, problem))
                }
                ("Corporate", Datom::Struct(fields)) if fields.len() == 2 => {
                    let mut it = fields.into_iter();
                    let path = Vec::<Integer>::incorporate_from(it.next().unwrap())
                        .map_err(|f| f.prepend(0))?;
                    let problem =
                        Problem::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(1))?;
                    Ok(Fault::Corporate(path, problem))
                }
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(name))),
            },
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Variant, other),
            )),
        }
    }
}

impl protos::Incorporable<Fault> for Datom {
    type Fault = Fault;

    fn incorporate(self) -> Result<Fault, Fault> {
        Fault::incorporate_from(self)
    }
}

// ---------------------------------------------------------------------------
// Datomic implementations: protos structural types
// ---------------------------------------------------------------------------

impl_datomic_scalar!(
    Separator,
    |v: &Separator| {
        Datom::Bare(
            match v {
                Separator::Period => "Period",
                Separator::Exclamation => "Exclamation",
                Separator::Colon => "Colon",
            }
            .to_owned(),
        )
    },
    |datom: Datom| {
        match datom {
            Datom::Bare(s) => match s.as_str() {
                "Period" => Ok(Separator::Period),
                "Exclamation" => Ok(Separator::Exclamation),
                "Colon" => Ok(Separator::Colon),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Bare, other),
            )),
        }
    }
);

impl_datomic_scalar!(
    Enclosure,
    |v: &Enclosure| {
        Datom::Bare(
            match v {
                Enclosure::Braced => "Braced",
                Enclosure::Bracketed => "Bracketed",
                Enclosure::Angled => "Angled",
            }
            .to_owned(),
        )
    },
    |datom: Datom| {
        match datom {
            Datom::Bare(s) => match s.as_str() {
                "Braced" => Ok(Enclosure::Braced),
                "Bracketed" => Ok(Enclosure::Bracketed),
                "Angled" => Ok(Enclosure::Angled),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Bare, other),
            )),
        }
    }
);

impl_datomic_scalar!(
    Boundary,
    |v: &Boundary| {
        Datom::Bare(
            match v {
                Boundary::CurlyQuotes => "CurlyQuotes",
                Boundary::Parentheses => "Parentheses",
            }
            .to_owned(),
        )
    },
    |datom: Datom| {
        match datom {
            Datom::Bare(s) => match s.as_str() {
                "CurlyQuotes" => Ok(Boundary::CurlyQuotes),
                "Parentheses" => Ok(Boundary::Parentheses),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Bare, other),
            )),
        }
    }
);

impl_datomic_scalar!(
    Extent,
    |v: &Extent| Datom::Struct(vec![v.0.conceive().unwrap(), v.1.conceive().unwrap()]),
    |datom: Datom| {
        match datom {
            Datom::Struct(fields) => {
                if fields.len() != 2 {
                    return Err(Fault::Corporate(
                        vec![],
                        Problem::Arity(2, fields.len() as Integer),
                    ));
                }
                let mut it = fields.into_iter();
                let start =
                    Integer::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(0))?;
                let end =
                    Integer::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(1))?;
                Ok(Extent(start, end))
            }
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Struct, other),
            )),
        }
    }
);

impl protos::Conceivable<Datom> for protos::Problem {
    type Fault = Infallible;

    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(match self {
            protos::Problem::Unclosed(e) => {
                Datom::Variant(Head::Bare("Unclosed".to_owned()), Box::new(e.conceive()?))
            }
            protos::Problem::UnclosedBoundary(b) => Datom::Variant(
                Head::Bare("UnclosedBoundary".to_owned()),
                Box::new(b.conceive()?),
            ),
            protos::Problem::Unopened => Datom::Bare("Unopened".to_owned()),
        })
    }
}

impl Datomic for protos::Problem {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(Head::Bare(name), body) => match (name.as_str(), *body) {
                ("Unclosed", body) => {
                    Enclosure::incorporate_from(body).map(protos::Problem::Unclosed)
                }
                ("UnclosedBoundary", body) => {
                    Boundary::incorporate_from(body).map(protos::Problem::UnclosedBoundary)
                }
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(name))),
            },
            Datom::Bare(s) => match s.as_str() {
                "Unopened" => Ok(protos::Problem::Unopened),
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(s))),
            },
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Variant, other),
            )),
        }
    }
}

impl protos::Incorporable<protos::Problem> for Datom {
    type Fault = Fault;

    fn incorporate(self) -> Result<protos::Problem, Fault> {
        protos::Problem::incorporate_from(self)
    }
}

impl_datomic_scalar!(
    protos::Fault,
    |v: &protos::Fault| {
        Datom::Struct(vec![
            v.extent.conceive().unwrap(),
            v.problem.conceive().unwrap(),
        ])
    },
    |datom: Datom| {
        match datom {
            Datom::Struct(fields) => {
                if fields.len() != 2 {
                    return Err(Fault::Corporate(
                        vec![],
                        Problem::Arity(2, fields.len() as Integer),
                    ));
                }
                let mut it = fields.into_iter();
                let extent =
                    Extent::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(0))?;
                let problem = protos::Problem::incorporate_from(it.next().unwrap())
                    .map_err(|f| f.prepend(1))?;
                Ok(protos::Fault { extent, problem })
            }
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Struct, other),
            )),
        }
    }
);

// ---------------------------------------------------------------------------
// Datom identity
// ---------------------------------------------------------------------------

impl protos::Conceivable<Datom> for Datom {
    type Fault = Infallible;

    fn conceive(&self) -> Result<Datom, Infallible> {
        Ok(self.clone())
    }
}

impl Datomic for Datom {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        Ok(datom)
    }
}

impl protos::Incorporable<Datom> for Datom {
    type Fault = Fault;

    fn incorporate(self) -> Result<Datom, Fault> {
        Ok(self)
    }
}

// ---------------------------------------------------------------------------
// Box<T>: blanket Incorporable, macro for Conceivable + Datomic
// ---------------------------------------------------------------------------

impl<T: Datomic> protos::Incorporable<Box<T>> for Datom {
    type Fault = Fault;

    fn incorporate(self) -> Result<Box<T>, Fault> {
        T::incorporate_from(self).map(Box::new)
    }
}

/// Implement `Conceivable<Datom>` and `Datomic` for `Box<T>`.
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
                let extent = Option::<Extent>::incorporate_from(it.next().unwrap())
                    .map_err(|f| f.prepend(0))?;
                let fault = F::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(1))?;
                Ok(Situated(extent, fault))
            }
            Datom::Struct(fields) => Err(Fault::Corporate(
                vec![],
                Problem::Arity(2, fields.len() as Integer),
            )),
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Struct, other),
            )),
        }
    }
}

impl<F: Datomic> protos::Incorporable<Situated<F>> for Datom {
    type Fault = Fault;

    fn incorporate(self) -> Result<Situated<F>, Fault> {
        Situated::<F>::incorporate_from(self)
    }
}
