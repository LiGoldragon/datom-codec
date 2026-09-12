//! The finite decimal.
//!
//! A datom decimal is finite and point-mandatory. The reader has always held
//! both halves of that — it refuses `NaN`, it refuses `inf`, and it refuses a
//! run with no point. The writer held neither: `f64::datomize` wrote whatever
//! `f64::to_string` gave it, so a non-finite value produced text that no
//! `Decimal` position would read back, silently.
//!
//! The terminal fix is not a check in the writer. It is a type in which the
//! unwritable value does not exist: [`Decimal`] holds its `f64` privately and
//! admits only a finite one. A writer that cannot be handed `NaN` cannot write
//! `NaN`, and there is no second place to keep the rule in step with the
//! reader.
//!
//! Finiteness buys the rest of the type's shape with it. Without `NaN`,
//! equality is reflexive, so `Decimal` is `Eq`; with `Eq` it is `Hash`, so a
//! contract value carrying one can be a map key; and the order is total, so it
//! is `Ord`. `f64` is none of those, and every generated type that reached one
//! lost them too.

use std::{cmp::Ordering, fmt, hash::Hash, hash::Hasher, str::FromStr};

use crate::*;

/// A finite decimal: no `NaN`, no infinity, and no `-0.0` distinct from `0.0`.
///
/// The inner `f64` is private. [`Decimal::try_from`] is the only way in, and
/// it refuses every value that has no datom text.
#[derive(Clone, Copy, Debug, Default)]
pub struct Decimal(f64);

/// The reason a float is not a decimal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NotFinite;

impl fmt::Display for NotFinite {
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        form.write_str("a decimal is finite: NaN and infinity have no datom text")
    }
}

impl std::error::Error for NotFinite {}

impl TryFrom<f64> for Decimal {
    type Error = NotFinite;

    fn try_from(value: f64) -> Result<Self, NotFinite> {
        if value.is_finite() {
            // `-0.0 == 0.0` but they hash and order apart, so the sign of zero
            // is normalized away here rather than left to break Eq/Hash/Ord
            // agreement further down.
            Ok(Self(if value == 0.0 { 0.0 } else { value }))
        } else {
            Err(NotFinite)
        }
    }
}

/// A decimal yields the finite float it holds.
pub trait Decimating {
    fn float(&self) -> f64;
}

impl Decimating for Decimal {
    fn float(&self) -> f64 {
        self.0
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// Sound because the value is finite: `NaN` is the only `f64` for which
/// equality is not reflexive, and no `Decimal` holds one.
impl Eq for Decimal {}

impl Hash for Decimal {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        // Equal finite values with the sign of zero normalized have equal
        // bits, so this agrees with `PartialEq` as `Hash` requires.
        self.0.to_bits().hash(hasher);
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Total, again because the value is finite: `total_cmp` is a total order on
/// every `f64`, and over the finite ones it is the numeric order.
impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl fmt::Display for Decimal {
    /// Point-mandatory: a whole-valued decimal still carries its point, so the
    /// text reads back as a decimal and not as an integer.
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.0.to_string();
        form.write_str(&text)?;
        if text.contains('.') {
            Ok(())
        } else {
            form.write_str(".0")
        }
    }
}

impl Datomizable for Decimal {
    fn datomize(&self, at: Path) -> Datom {
        Datom {
            path: at,
            form: Form::Bare(self.to_string()),
        }
    }
}

impl Composing for Decimal {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        Self::scalar(datom, budget)
    }
}

impl Scalar for Decimal {
    fn scalar(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        let text = match &datom.form {
            Form::Bare(value) => value.clone(),
            Form::Variant(whole, fraction) => {
                budget.spend(&fraction.path)?;
                match &fraction.form {
                    Form::Bare(fraction) => format!("{}.{}", whole.0, fraction),
                    found => {
                        return Err(Error {
                            layer: ErrorLayer::Composition,
                            path: fraction.path.clone(),
                            kind: ErrorKind::Form {
                                expected: "Bare".into(),
                                found: found.form_name().to_owned(),
                            },
                        });
                    }
                }
            }
            found => {
                return Err(Error {
                    layer: ErrorLayer::Composition,
                    path: datom.path.clone(),
                    kind: ErrorKind::Form {
                        expected: "Bare".into(),
                        found: found.form_name().to_owned(),
                    },
                });
            }
        };
        text.parse::<Decimal>().map_err(|_| Error {
            layer: ErrorLayer::Composition,
            path: datom.path.clone(),
            kind: ErrorKind::Value {
                expected: "Decimal".into(),
                value: text,
            },
        })
    }
}

impl FromStr for Decimal {
    type Err = NotFinite;

    /// Canonical datom decimal text: a mandatory point, ASCII digits on both
    /// sides, no leading plus, and no leading zero in the whole part except a
    /// lone `0`.
    fn from_str(text: &str) -> Result<Self, NotFinite> {
        let (whole, fraction) = text.split_once('.').ok_or(NotFinite)?;
        let digits = whole.strip_prefix('-').unwrap_or(whole);
        let canonical = !digits.is_empty()
            && digits.chars().all(|character| character.is_ascii_digit())
            && (digits.len() == 1 || !digits.starts_with('0'))
            && !fraction.is_empty()
            && fraction.chars().all(|character| character.is_ascii_digit());
        if !canonical {
            return Err(NotFinite);
        }
        text.parse::<f64>().map_err(|_| NotFinite)?.try_into()
    }
}
