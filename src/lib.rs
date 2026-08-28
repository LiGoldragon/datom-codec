//! Datomic's typed Portion anatomy.
//!
//! Protos owns text delineation and printing. This crate owns only the
//! contextual mapping between one expected Rust type and a Protos `Portion`.

use std::{collections::BTreeMap, fmt};

use protos::{
    Bare, BareExpectation, BareSafe, Delineatable, Enclosed, EnclosedAnatomy, Extent, Headed,
    Layout, OpaqueBoundary, OpaqueEnclosed, Portion, PortionText, Printing, Separator,
    StructuralEnclosed, StructuralEnclosure, Symbol,
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
    Protos,
}

/// The sole hand-written anatomy pattern, emitted verbatim by future Ethos.
pub trait Datomic: Sized {
    fn embody(portion: &Portion) -> Result<Self, Fault>;

    fn portion(&self) -> Portion;

    fn textualize(&self) -> Text<Self> {
        let printed = self.portion().print(Layout::Flat);
        Text::from(printed.as_ref())
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
        let Some(symbol) = portion.bare_symbol() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        symbol
            .canonical_integer()
            .then(|| symbol.parse().ok())
            .flatten()
            .ok_or_else(|| portion.fault(FaultProblem::Value))
    }

    fn portion(&self) -> Portion {
        self.to_string().as_str().bare()
    }
}

impl Datomic for f64 {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let representation = match portion {
            Portion::Headed(_, headed)
                if headed.separator == Separator::Period && headed.body.bare_symbol().is_some() =>
            {
                format!(
                    "{}.{}",
                    headed.head.as_ref(),
                    headed.body.bare_symbol().expect("checked above")
                )
            }
            Portion::Bare(_, bare) => bare.symbol.as_ref().into(),
            _ => return Err(portion.fault(FaultProblem::Shape)),
        };
        if !representation.as_str().canonical_decimal() {
            return Err(portion.fault(FaultProblem::Value));
        }
        representation
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(|| portion.fault(FaultProblem::Value))
    }

    fn portion(&self) -> Portion {
        let decimal = self.decimal();
        let (whole, fraction) = decimal
            .split_once('.')
            .expect("Datomic f64 textualization always includes a point");
        whole.headed(Separator::Period, fraction.bare())
    }
}

impl Datomic for String {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        if let Some(content) = portion.opaque(OpaqueBoundary::CurlyQuote) {
            return Ok(content.into());
        }
        if let Some(content) = portion.opaque(OpaqueBoundary::Dialect(
            protos::DialectBoundary::Parentheses,
        )) {
            return Ok(content.into());
        }
        Ok(portion.canonical_text().as_ref().into())
    }

    fn portion(&self) -> Portion {
        let prospective = Text::<()>::from(self.as_str());
        if prospective.is_bare_safe_for(BareExpectation::String) {
            prospective
                .delineate()
                .expect("a bare-safe String has a Protos delineation")
                .portions
                .into_iter()
                .next()
                .expect("a bare-safe String has one Portion")
        } else {
            self.as_str().opaque(OpaqueBoundary::CurlyQuote, self)
        }
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
            map.insert(K::embody(&pair[0])?, V::embody(&pair[1])?);
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

trait ScalarAnatomy {
    fn canonical_integer(&self) -> bool;
    fn canonical_decimal(&self) -> bool;
    fn decimal(&self) -> String;
}

impl ScalarAnatomy for str {
    fn canonical_integer(&self) -> bool {
        match self.as_bytes() {
            [b'0'] => true,
            [b'1'..=b'9', rest @ ..] => rest.iter().all(u8::is_ascii_digit),
            [b'-', b'1'..=b'9', rest @ ..] => rest.iter().all(u8::is_ascii_digit),
            _ => false,
        }
    }

    fn canonical_decimal(&self) -> bool {
        let Some((whole, fraction)) = self.split_once('.') else {
            return false;
        };
        let digits = if let Some(whole) = whole.strip_prefix('-') {
            whole
        } else {
            whole
        };
        !digits.is_empty()
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && !fraction.is_empty()
            && fraction.bytes().all(|byte| byte.is_ascii_digit())
    }

    fn decimal(&self) -> String {
        self.into()
    }
}

impl ScalarAnatomy for f64 {
    fn canonical_integer(&self) -> bool {
        false
    }

    fn canonical_decimal(&self) -> bool {
        false
    }

    fn decimal(&self) -> String {
        let rendered = self.to_string();
        let plain = match rendered.split_once(['e', 'E']) {
            Some((mantissa, exponent)) => {
                let exponent = exponent
                    .parse::<i32>()
                    .expect("Rust f64 exponent is decimal");
                let negative = mantissa.starts_with('-');
                let digits = mantissa.trim_start_matches('-').replace('.', "");
                let point = mantissa.find('.').unwrap_or(mantissa.len()) - usize::from(negative);
                let target = point as i32 + exponent;
                let mut expanded = if target <= 0 {
                    format!("0.{}{}", "0".repeat((-target) as usize), digits)
                } else if target as usize >= digits.len() {
                    format!("{}{}", digits, "0".repeat(target as usize - digits.len()))
                } else {
                    format!(
                        "{}.{}",
                        &digits[..target as usize],
                        &digits[target as usize..]
                    )
                };
                if negative {
                    expanded.insert(0, '-');
                }
                expanded
            }
            None => rendered,
        };
        if plain.contains('.') {
            plain
        } else {
            format!("{plain}.0")
        }
    }
}

/// Canonical Portion constructors for a hand-declared Datomic anatomy.
pub trait PortionBuilding {
    fn bare(&self) -> Portion;
    fn headed(&self, separator: Separator, body: Portion) -> Portion;
    fn structural(&self, enclosure: StructuralEnclosure, portions: Vec<Portion>) -> Portion;
    fn opaque(&self, boundary: OpaqueBoundary, content: &str) -> Portion;
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

    fn opaque(&self, boundary: OpaqueBoundary, content: &str) -> Portion {
        let enclosed = OpaqueEnclosed::try_from((boundary, content.into()))
            .expect("Datomic strings have a representable Protos opaque boundary");
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

impl fmt::Debug for FaultProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Shape => "Shape",
            Self::Head => "Head",
            Self::Value => "Value",
            Self::Arity => "Arity",
            Self::MapPair => "MapPair",
            Self::Protos => "Protos",
        })
    }
}
