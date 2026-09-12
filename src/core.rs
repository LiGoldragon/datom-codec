use std::marker::PhantomData;

use crate::composition::Variantizing;

use crate::{Integer, Opaque, Path};
use protos::Symbol;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Datom {
    pub path: Path,
    pub form: Form,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Form {
    Struct(Vec<Datom>),
    Vector(Vec<Datom>),
    Variant(Symbol, Box<Datom>),
    Bare(String),
    String(String),
    Meaning(Opaque),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorLayer {
    Protos,
    Datom,
    Composition,
}
#[derive(Clone, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
    fn compose<T: Compositional>(&self, budget: &mut Budget) -> Result<T, Error>;
}
pub trait Compositional: Sized {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error>;
}
pub trait Datomizable {
    type Output;
    fn datomize(&self, at: Path) -> Self::Output;
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

pub struct Positions<'a> {
    path: &'a Path,
    children: &'a [Datom],
    next: usize,
}
pub trait Positioning {
    fn position<T: Compositional>(&mut self, budget: &mut Budget) -> Result<T, Error>;
}
impl Positioning for Positions<'_> {
    fn position<T: Compositional>(&mut self, budget: &mut Budget) -> Result<T, Error> {
        let child = self.children.get(self.next).ok_or_else(|| Error {
            layer: ErrorLayer::Composition,
            path: self.path.clone(),
            kind: ErrorKind::Arity {
                expected: self.next as Integer + 1,
                found: self.children.len() as Integer,
            },
        })?;
        self.next += 1;
        child.compose(budget)
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
    fn positions(&self, arity: Integer) -> Result<Positions<'_>, Error>;
}
impl DatomPositioning for Datom {
    fn positions(&self, arity: Integer) -> Result<Positions<'_>, Error> {
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
            path: &self.path,
            children,
            next: 0,
        })
    }
}
impl Composable for Datom {
    fn compose<T: Compositional>(&self, budget: &mut Budget) -> Result<T, Error> {
        budget.enter_composition(&self.path)?;
        let result = T::compose(self, budget);
        budget.leave_composition();
        result
    }
}

impl Datomizable for ErrorLayer {
    type Output = Datom;
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
impl Compositional for ErrorLayer {
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
    type Output = Datom;
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
impl Compositional for ErrorKind {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        if matches!(&datom.form, Form::Bare(name) if name == "Budget") {
            budget.spend(&datom.path)?;
            return Ok(Self::Budget);
        }
        let (head, body) = datom.variant(budget, "ErrorKind")?;
        if head == "Structural" {
            return Ok(Self::Structural(body.compose(budget)?));
        }
        let mut positions = body.positions(2)?;
        match head {
            "Form" => Ok(Self::Form {
                expected: positions.position(budget)?,
                found: positions.position(budget)?,
            }),
            "Arity" => Ok(Self::Arity {
                expected: positions.position(budget)?,
                found: positions.position(budget)?,
            }),
            "Value" => Ok(Self::Value {
                expected: positions.position(budget)?,
                value: positions.position(budget)?,
            }),
            "Variant" => Ok(Self::Variant {
                expected: positions.position(budget)?,
                found: positions.position(budget)?,
            }),
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
    type Output = Datom;
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
        let mut positions = body.positions(3)?;
        Ok(Self {
            layer: positions.position(budget)?,
            path: positions.position(budget)?,
            kind: positions.position(budget)?,
        })
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
impl<T: Compositional> Actualizing<T> for Potential<T> {
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
        let datom = protos.datomize(Path::new())?;
        datom.compose(budget)
    }
}
