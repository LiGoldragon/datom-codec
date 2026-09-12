use std::marker::PhantomData;

use crate::composition::Variantizing;

use crate::{Integer, Opaque, Path};
use protos::Symbol;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Datom {
    pub path: Path,
    pub form: Form,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Form {
    Struct(Vec<Datom>),
    Vector(Vec<Datom>),
    Variant(Symbol, Box<Datom>),
    Bare(String),
    String(String),
    Meaning(Opaque),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Budget {
    pub remaining: Integer,
    pub reader: protos::ReaderBudget,
    pub depth: Integer,
    pub maximum_depth: Integer,
}

pub trait Budgeting {
    fn spend(&mut self, path: &Path) -> Result<(), Error>;
}
pub trait CompositionDepthing {
    fn enter_composition(&mut self, path: &Path) -> Result<(), Error>;
    fn leave_composition(&mut self);
}
impl Budgeting for Budget {
    fn spend(&mut self, path: &Path) -> Result<(), Error> {
        if self.remaining <= 0 {
            return Err(Error {
                layer: ErrorLayer::Composition,
                path: path.clone(),
                kind: ErrorKind::Budget,
            });
        }
        self.remaining -= 1;
        Ok(())
    }
}

impl CompositionDepthing for Budget {
    fn enter_composition(&mut self, path: &Path) -> Result<(), Error> {
        if self.depth >= self.maximum_depth {
            return Err(Error::composition(path.clone(), ErrorKind::Budget));
        }
        self.depth += 1;
        Ok(())
    }
    fn leave_composition(&mut self) {
        self.depth -= 1;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ErrorLayer {
    Protos,
    Datom,
    Composition,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Error {
    pub layer: ErrorLayer,
    pub path: Path,
    pub kind: ErrorKind,
}
pub trait ErrorRaising {
    fn composition(path: Path, kind: ErrorKind) -> Self;
}
impl ErrorRaising for Error {
    fn composition(path: Path, kind: ErrorKind) -> Self {
        Self {
            layer: ErrorLayer::Composition,
            path,
            kind,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Budget,
    Structural(protos::Error),
    Form { expected: String, found: String },
    Arity { expected: Integer, found: Integer },
    Value { expected: String, value: String },
    Variant { expected: String, found: String },
}
pub trait ProtosExtenting {
    fn extent(&self) -> protos::Extent;
    fn at(&self, path: &[Integer]) -> Option<&protos::Protos>;
}
impl ProtosExtenting for protos::Protos {
    fn extent(&self) -> protos::Extent {
        match self {
            Self::Headed { extent, .. }
            | Self::Enclosed { extent, .. }
            | Self::Opaque { extent, .. }
            | Self::Bare { extent, .. } => *extent,
        }
    }
    fn at(&self, path: &[Integer]) -> Option<&protos::Protos> {
        let Some((&index, tail)) = path.split_first() else {
            return Some(self);
        };
        match self {
            Self::Enclosed { children, .. } => children
                .get(usize::try_from(index).ok()?)
                .and_then(|child| child.at(tail)),
            Self::Headed { body, .. } if index == 1 => body.at(tail),
            _ => None,
        }
    }
}

pub trait Composable {
    fn compose<T: Composing>(&self, budget: &mut Budget) -> Result<T, Error>;
    fn compose_positions<T: Compositional>(&self, budget: &mut Budget) -> Result<T, Error>;
}
/// A type a datom composes into. Every composable type bears this: the
/// scalars, the containers, the enums, and — through [`Compositional`] — the
/// positional types.
pub trait Composing: Sized {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error>;
}
/// A type whose datom form is a struct of positions. It states its arity and
/// builds itself from its positions; it never reads the tree. The reading —
/// budget, arity, positions, locus — is [`Composable::compose_positions`].
pub trait Compositional: Composing {
    const ARITY: Integer;
    fn from_positions(positions: Positions<'_>) -> Result<Self, Error>;
}
/// A composition becomes a datom. Infallible, and the only conversion
/// this kind names: the descent `Protos -> Datom` is [`crate::DatomForming`],
/// which may fail and therefore is not this.
pub trait Datomizable {
    fn datomize(&self, at: Path) -> Datom;
}
pub trait Pathing {
    fn child(&self, index: Integer) -> Path;
}
impl Pathing for Path {
    fn child(&self, index: Integer) -> Path {
        let mut child = self.clone();
        child.push(index);
        child
    }
}

/// The positions of one struct form, with the budget they are read against.
pub struct Positions<'a> {
    children: &'a [Datom],
    next: usize,
    budget: &'a mut Budget,
}
pub trait Positioning {
    fn position<T: Composing>(&mut self) -> Result<T, Error>;
}
impl Positioning for Positions<'_> {
    /// `positions` validated the count before handing these out, so every
    /// position it promised is there; asking past that arity is a mistake in
    /// the caller, not a refusal the datum earned.
    fn position<T: Composing>(&mut self) -> Result<T, Error> {
        let child = &self.children[self.next];
        self.next += 1;
        child.compose(self.budget)
    }
}

pub trait Naming {
    fn form_name(&self) -> &'static str;
}
impl Naming for Form {
    fn form_name(&self) -> &'static str {
        match self {
            Form::Struct(_) => "Struct",
            Form::Vector(_) => "Vector",
            Form::Variant(_, _) => "Variant",
            Form::Bare(_) => "Bare",
            Form::String(_) => "String",
            Form::Meaning(_) => "Meaning",
        }
    }
}
pub trait DatomPositioning {
    fn positions<'a>(
        &'a self,
        arity: Integer,
        budget: &'a mut Budget,
    ) -> Result<Positions<'a>, Error>;
}
impl DatomPositioning for Datom {
    fn positions<'a>(
        &'a self,
        arity: Integer,
        budget: &'a mut Budget,
    ) -> Result<Positions<'a>, Error> {
        let children = match &self.form {
            Form::Struct(children) => children,
            found => {
                return Err(Error {
                    layer: ErrorLayer::Composition,
                    path: self.path.clone(),
                    kind: ErrorKind::Form {
                        expected: "Struct".to_owned(),
                        found: found.form_name().to_owned(),
                    },
                });
            }
        };
        if children.len() as Integer != arity {
            return Err(Error {
                layer: ErrorLayer::Composition,
                path: self.path.clone(),
                kind: ErrorKind::Arity {
                    expected: arity,
                    found: children.len() as Integer,
                },
            });
        }
        Ok(Positions {
            children,
            next: 0,
            budget,
        })
    }
}
impl Composable for Datom {
    fn compose<T: Composing>(&self, budget: &mut Budget) -> Result<T, Error> {
        budget.enter_composition(&self.path)?;
        let result = T::compose(self, budget);
        budget.leave_composition();
        result
    }
    /// The one reading of a positional tree: the budget it spends, the arity it
    /// demands, the positions it hands out, and the path it refuses at.
    fn compose_positions<T: Compositional>(&self, budget: &mut Budget) -> Result<T, Error> {
        budget.spend(&self.path)?;
        T::from_positions(self.positions(T::ARITY, budget)?)
    }
}

impl Datomizable for ErrorLayer {
    fn datomize(&self, at: Path) -> Datom {
        let name = match self {
            Self::Protos => "Protos",
            Self::Datom => "Datom",
            Self::Composition => "Composition",
        };
        Datom {
            path: at,
            form: Form::Bare(name.to_owned()),
        }
    }
}
impl Composing for ErrorLayer {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        match &datom.form {
            Form::Bare(name) => match name.as_str() {
                "Protos" => Ok(Self::Protos),
                "Datom" => Ok(Self::Datom),
                "Composition" => Ok(Self::Composition),
                found => Err(Error::composition(
                    datom.path.clone(),
                    ErrorKind::Variant {
                        expected: "ErrorLayer".into(),
                        found: found.to_owned(),
                    },
                )),
            },
            found => Err(Error::composition(
                datom.path.clone(),
                ErrorKind::Form {
                    expected: "Bare".into(),
                    found: found.form_name().to_owned(),
                },
            )),
        }
    }
}
impl Datomizable for ErrorKind {
    fn datomize(&self, at: Path) -> Datom {
        match self {
            Self::Budget => Datom {
                path: at,
                form: Form::Bare("Budget".into()),
            },
            Self::Structural(error) => error.datomize(at.child(1)).named_variant(at, "Structural"),
            Self::Form { expected, found } => Datom {
                path: at.child(1),
                form: Form::Struct(vec![
                    expected.datomize(at.child(1).child(0)),
                    found.datomize(at.child(1).child(1)),
                ]),
            }
            .named_variant(at, "Form"),
            Self::Arity { expected, found } => Datom {
                path: at.child(1),
                form: Form::Struct(vec![
                    expected.datomize(at.child(1).child(0)),
                    found.datomize(at.child(1).child(1)),
                ]),
            }
            .named_variant(at, "Arity"),
            Self::Value { expected, value } => Datom {
                path: at.child(1),
                form: Form::Struct(vec![
                    expected.datomize(at.child(1).child(0)),
                    value.datomize(at.child(1).child(1)),
                ]),
            }
            .named_variant(at, "Value"),
            Self::Variant { expected, found } => Datom {
                path: at.child(1),
                form: Form::Struct(vec![
                    expected.datomize(at.child(1).child(0)),
                    found.datomize(at.child(1).child(1)),
                ]),
            }
            .named_variant(at, "Variant"),
        }
    }
}
impl Composing for ErrorKind {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        if matches!(&datom.form, Form::Bare(name) if name == "Budget") {
            budget.spend(&datom.path)?;
            return Ok(Self::Budget);
        }
        let (head, body) = datom.variant(budget, "ErrorKind")?;
        match head {
            "Structural" => Ok(Self::Structural(body.compose(budget)?)),
            "Arity" => {
                let (expected, found) = body.compose_positions(budget)?;
                Ok(Self::Arity { expected, found })
            }
            "Form" => {
                let (expected, found) = body.compose_positions(budget)?;
                Ok(Self::Form { expected, found })
            }
            "Value" => {
                let (expected, value) = body.compose_positions(budget)?;
                Ok(Self::Value { expected, value })
            }
            "Variant" => {
                let (expected, found) = body.compose_positions(budget)?;
                Ok(Self::Variant { expected, found })
            }
            found => Err(Error::composition(
                datom.path.clone(),
                ErrorKind::Variant {
                    expected: "ErrorKind".into(),
                    found: found.to_owned(),
                },
            )),
        }
    }
}
impl Datomizable for Error {
    fn datomize(&self, at: Path) -> Datom {
        Datom {
            path: at.child(1),
            form: Form::Struct(vec![
                self.layer.datomize(at.child(1).child(0)),
                self.path.datomize(at.child(1).child(1)),
                self.kind.datomize(at.child(1).child(2)),
            ]),
        }
        .named_variant(at, "Error")
    }
}
impl Compositional for Error {
    const ARITY: Integer = 3;
    fn from_positions(mut positions: Positions<'_>) -> Result<Self, Error> {
        Ok(Self {
            layer: positions.position()?,
            path: positions.position()?,
            kind: positions.position()?,
        })
    }
}
impl Composing for Error {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        let (head, body) = datom.variant(budget, "Error")?;
        if head != "Error" {
            return Err(Error::composition(
                datom.path.clone(),
                ErrorKind::Variant {
                    expected: "Error".into(),
                    found: head.to_owned(),
                },
            ));
        }
        body.compose_positions(budget)
    }
}

/// Text that may actualize into `T` after structural reading and datomic composition.
pub struct Potential<T> {
    text: String,
    reader: Option<protos::Protos>,
    marker: PhantomData<fn() -> T>,
}
impl<T> From<&str> for Potential<T> {
    fn from(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            reader: None,
            marker: PhantomData,
        }
    }
}
impl<T> From<String> for Potential<T> {
    fn from(text: String) -> Self {
        Self {
            text,
            reader: None,
            marker: PhantomData,
        }
    }
}
pub trait Actualizing<T> {
    fn actualize(&mut self, budget: &mut Budget) -> Result<T, Error>;
}
pub trait PotentialExtenting {
    fn reader_extent(&self, path: &Path) -> Option<protos::Extent>;
}
impl<T> PotentialExtenting for Potential<T> {
    fn reader_extent(&self, path: &Path) -> Option<protos::Extent> {
        self.reader.as_ref()?.at(path).map(ProtosExtenting::extent)
    }
}
impl<T: Composing> Actualizing<T> for Potential<T> {
    fn actualize(&mut self, budget: &mut Budget) -> Result<T, Error> {
        use protos::BoundedProtosizable;
        let protos = self
            .text
            .protosize_with(&mut budget.reader)
            .map_err(|error| Error {
                layer: ErrorLayer::Protos,
                path: Path::new(),
                kind: ErrorKind::Structural(error),
            })?;
        self.reader = Some(protos.clone());
        let datom = crate::DatomForming::datom_form(&protos, Path::new())?;
        datom.compose(budget)
    }
}
