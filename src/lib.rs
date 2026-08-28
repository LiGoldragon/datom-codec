//! Datomic's typed Portion anatomy.
//!
//! Protos owns text delineation and printing. This crate owns only the
//! contextual mapping between one expected Rust type and a Protos `Portion`.

use std::{cmp::Ordering, collections::BTreeMap, fmt, string::String as StdString};

use protos::{
    Bare, Delineatable, DelineatedText, Enclosed, EnclosedAnatomy, Extent, Headed, Layout,
    OpaqueBoundary, Portion, PortionText, Printing, ScalarAnatomy, Separator, StructuralEnclosed,
    StructuralEnclosure, Symbol,
};

pub use protos::Text;

/// A typed failure, always tied to the Portion extent where it was found.
pub struct Fault {
    pub extent: Extent,
    pub problem: FaultProblem,
}

/// Datomic's anatomy-level fault taxonomy.
pub enum FaultProblem {
    Shape,
    Head,
    Value,
    Arity,
    MapPair,
    DuplicateMapKey,
    UnrepresentableString,
    Protos,
}

/// The sole hand-written anatomy pattern, emitted verbatim by future Ethos.
pub trait Datomic: Sized {
    fn embody(portion: &Portion) -> Result<Self, Fault>;

    fn portion(&self) -> Portion;

    fn textualize(&self) -> Text<Self> {
        self.portion().print(Layout::Flat).retag()
    }
}

/// The public incoming edge: prospective Protos text to one expected type.
pub trait TextEdge<T> {
    fn embody(&self) -> Result<T, Fault>;
}

/// Portion questions Datomic needs; they never inspect characters.
pub trait PortionViewing {
    fn bare_symbol(&self) -> Option<&str>;
    fn headed(&self) -> Option<&Headed>;
    fn structural(&self, enclosure: StructuralEnclosure) -> Option<&[Portion]>;
    fn opaque(&self, boundary: OpaqueBoundary) -> Option<&str>;
    fn fault(&self, problem: FaultProblem) -> Fault;
}

impl<T: Datomic> TextEdge<T> for Text<T> {
    fn embody(&self) -> Result<T, Fault> {
        let delineation = self.delineate().map_err(|fault| Fault {
            extent: fault.extent,
            problem: FaultProblem::Protos,
        })?;
        match delineation.portions.as_slice() {
            [portion] => T::embody(portion),
            portions => Err(Fault {
                extent: Extent {
                    start: 0,
                    end: self.as_ref().len(),
                },
                problem: if portions.is_empty() {
                    FaultProblem::Arity
                } else {
                    FaultProblem::Shape
                },
            }),
        }
    }
}

impl PortionViewing for Portion {
    fn bare_symbol(&self) -> Option<&str> {
        match self {
            Portion::Bare(_, bare) => Some(bare.symbol.as_ref()),
            _ => None,
        }
    }

    fn headed(&self) -> Option<&Headed> {
        match self {
            Portion::Headed(_, headed) => Some(headed),
            _ => None,
        }
    }

    fn structural(&self, enclosure: StructuralEnclosure) -> Option<&[Portion]> {
        match self {
            Portion::Enclosed(_, enclosed)
                if enclosed.structural_enclosure() == Some(enclosure) =>
            {
                enclosed.portions()
            }
            _ => None,
        }
    }

    fn opaque(&self, boundary: OpaqueBoundary) -> Option<&str> {
        match self {
            Portion::Enclosed(_, enclosed) if enclosed.opaque_boundary() == Some(boundary) => {
                enclosed.opaque_content()
            }
            _ => None,
        }
    }

    fn fault(&self, problem: FaultProblem) -> Fault {
        Fault {
            extent: Extent {
                start: self.as_ref().start,
                end: self.as_ref().end,
            },
            problem,
        }
    }
}

impl Datomic for bool {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        match portion.bare_symbol() {
            Some("True") => Ok(true),
            Some("False") => Ok(false),
            _ => Err(portion.fault(FaultProblem::Value)),
        }
    }

    fn portion(&self) -> Portion {
        (if *self { "True" } else { "False" }).bare()
    }
}

impl Datomic for i64 {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        portion.signed_i64().map_err(|fault| Fault {
            extent: fault.extent,
            problem: FaultProblem::Protos,
        })
    }

    fn portion(&self) -> Portion {
        Portion::from_signed_i64(*self)
    }
}

/// A finite decimal embodied by Protos's scalar anatomy.
pub struct FiniteDecimal {
    value: f64,
    portion: Portion,
}

pub struct NonFiniteDecimal {
    pub extent: Extent,
}

impl TryFrom<f64> for FiniteDecimal {
    type Error = NonFiniteDecimal;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let portion = Portion::from_decimal_f64(value).map_err(|fault| NonFiniteDecimal {
            extent: fault.extent,
        })?;
        Ok(Self { value, portion })
    }
}

pub trait DecimalViewing {
    fn value(&self) -> f64;
}

impl DecimalViewing for FiniteDecimal {
    fn value(&self) -> f64 {
        self.value
    }
}

impl Datomic for FiniteDecimal {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let value = portion.decimal_f64().map_err(|fault| Fault {
            extent: fault.extent,
            problem: FaultProblem::Protos,
        })?;
        Ok(Self {
            value,
            portion: portion.clone(),
        })
    }

    fn portion(&self) -> Portion {
        self.portion.clone()
    }
}

/// A String whose canonical Datomic Portion is representable without escapes.
pub struct DatomicString {
    value: StdString,
    portion: Portion,
}

pub struct UnrepresentableString {
    pub extent: Extent,
}

impl AsRef<str> for DatomicString {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl PartialEq for DatomicString {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for DatomicString {}

impl PartialOrd for DatomicString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DatomicString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl TryFrom<StdString> for DatomicString {
    type Error = UnrepresentableString;

    fn try_from(value: StdString) -> Result<Self, Self::Error> {
        let portion = Portion::from_expected_string(value.as_str()).map_err(|fault| {
            UnrepresentableString {
                extent: fault.extent,
            }
        })?;
        Ok(Self { value, portion })
    }
}

impl Datomic for DatomicString {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        if let Some(content) = portion.opaque(OpaqueBoundary::CurlyQuote) {
            return Self::try_from(StdString::from(content))
                .map_err(|_| portion.fault(FaultProblem::UnrepresentableString));
        }
        if let Some(content) = portion.opaque(OpaqueBoundary::Dialect(
            protos::DialectBoundary::Parentheses,
        )) {
            return Self::try_from(StdString::from(content))
                .map_err(|_| portion.fault(FaultProblem::UnrepresentableString));
        }
        Self::try_from(StdString::from(portion.canonical_text().as_ref()))
            .map_err(|_| portion.fault(FaultProblem::UnrepresentableString))
    }

    fn portion(&self) -> Portion {
        self.portion.clone()
    }
}

impl<T: Datomic> Datomic for Vec<T> {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(portions) = portion.structural(StructuralEnclosure::Bracketed) else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        portions.iter().map(T::embody).collect()
    }

    fn portion(&self) -> Portion {
        "".structural(
            StructuralEnclosure::Bracketed,
            self.iter().map(T::portion).collect(),
        )
    }
}

impl<K: Datomic + Ord, V: Datomic> Datomic for BTreeMap<K, V> {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(portions) = portion.structural(StructuralEnclosure::Guillemets) else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        if portions.len() % 2 != 0 {
            return Err(portion.fault(FaultProblem::MapPair));
        }
        let mut map = BTreeMap::new();
        for pair in portions.chunks_exact(2) {
            let key = K::embody(&pair[0])?;
            if map.contains_key(&key) {
                return Err(pair[0].fault(FaultProblem::DuplicateMapKey));
            }
            map.insert(key, V::embody(&pair[1])?);
        }
        Ok(map)
    }

    fn portion(&self) -> Portion {
        let mut portions = Vec::with_capacity(self.len() * 2);
        for (key, value) in self {
            portions.push(key.portion());
            portions.push(value.portion());
        }
        "".structural(StructuralEnclosure::Guillemets, portions)
    }
}

impl<T: Datomic> Datomic for Option<T> {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        if portion.bare_symbol() == Some("None") {
            return Ok(None);
        }
        let Some(headed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        if headed.head.as_ref() != "Some" || headed.separator != Separator::Period {
            return Err(portion.fault(FaultProblem::Head));
        }
        T::embody(&headed.body).map(Some)
    }

    fn portion(&self) -> Portion {
        match self {
            None => "None".bare(),
            Some(value) => "Some".headed(Separator::Period, value.portion()),
        }
    }
}

/// Canonical Portion constructors for a hand-declared Datomic anatomy.
pub trait PortionBuilding {
    fn bare(&self) -> Portion;
    fn headed(&self, separator: Separator, body: Portion) -> Portion;
    fn structural(&self, enclosure: StructuralEnclosure, portions: Vec<Portion>) -> Portion;
}

impl PortionBuilding for str {
    fn bare(&self) -> Portion {
        let symbol = Symbol::try_from(self).expect("Datomic bare values are Protos symbols");
        Bare::from(symbol).into()
    }

    fn headed(&self, separator: Separator, body: Portion) -> Portion {
        let symbol = Symbol::try_from(self).expect("Datomic heads are Protos symbols");
        Headed::from((symbol, separator, body)).into()
    }

    fn structural(&self, enclosure: StructuralEnclosure, portions: Vec<Portion>) -> Portion {
        let enclosed = StructuralEnclosed::from((enclosure, portions));
        Enclosed::from(enclosed).into()
    }
}

impl fmt::Debug for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Fault")
            .field("extent", &self.extent)
            .field("problem", &self.problem)
            .finish()
    }
}

impl fmt::Debug for FiniteDecimal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("FiniteDecimal")
            .field(&self.value)
            .finish()
    }
}

impl fmt::Debug for DatomicString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("DatomicString")
            .field(&self.value)
            .finish()
    }
}

impl fmt::Debug for UnrepresentableString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UnrepresentableString")
            .field("extent", &self.extent)
            .finish()
    }
}

impl fmt::Debug for NonFiniteDecimal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NonFiniteDecimal")
            .field("extent", &self.extent)
            .finish()
    }
}

impl fmt::Debug for FaultProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Shape => "Shape",
            Self::Head => "Head",
            Self::Value => "Value",
            Self::Arity => "Arity",
            Self::MapPair => "MapPair",
            Self::DuplicateMapKey => "DuplicateMapKey",
            Self::UnrepresentableString => "UnrepresentableString",
            Self::Protos => "Protos",
        })
    }
}
