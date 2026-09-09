use std::{marker::PhantomData, str::FromStr};

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
}

pub trait Budgeting {
    fn spend(&mut self, path: &Path) -> Result<(), Error>;
}
impl Budgeting for Budget {
    fn spend(&mut self, path: &Path) -> Result<(), Error> {
        if self.remaining <= 0 {
            return Err(Error {
                path: path.clone(),
                kind: ErrorKind::Budget,
            });
        }
        self.remaining -= 1;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub path: Path,
    pub kind: ErrorKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Budget,
    Structural(protos::Error),
    Form {
        expected: &'static str,
        found: &'static str,
    },
    Arity {
        expected: Integer,
        found: Integer,
    },
    Value {
        expected: &'static str,
        value: String,
    },
    Variant {
        expected: &'static str,
        found: String,
    },
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
    const ARITY: Integer;
    fn from_positions(positions: Positions<'_>, budget: &mut Budget) -> Result<Self, Error>;
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        Self::from_positions(datom.positions("Struct")?, budget)
    }
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
    fn finish(self) -> Result<(), Error>;
}
impl Positioning for Positions<'_> {
    fn position<T: Compositional>(&mut self, budget: &mut Budget) -> Result<T, Error> {
        let child = self.children.get(self.next).ok_or_else(|| Error {
            path: self.path.clone(),
            kind: ErrorKind::Arity {
                expected: self.next as Integer + 1,
                found: self.next as Integer,
            },
        })?;
        self.next += 1;
        child.compose(budget)
    }
    fn finish(self) -> Result<(), Error> {
        if self.next == self.children.len() {
            Ok(())
        } else {
            Err(Error {
                path: self.path.clone(),
                kind: ErrorKind::Arity {
                    expected: self.next as Integer,
                    found: self.children.len() as Integer,
                },
            })
        }
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
    fn positions(&self, expected: &'static str) -> Result<Positions<'_>, Error>;
    fn variant_positions(&self) -> Result<Positions<'_>, Error>;
}
impl DatomPositioning for Datom {
    fn positions(&self, expected: &'static str) -> Result<Positions<'_>, Error> {
        let children = match &self.form {
            Form::Struct(children) => children,
            found => {
                return Err(Error {
                    path: self.path.clone(),
                    kind: ErrorKind::Form {
                        expected,
                        found: found.form_name(),
                    },
                });
            }
        };
        Ok(Positions {
            path: &self.path,
            children,
            next: 0,
        })
    }
    fn variant_positions(&self) -> Result<Positions<'_>, Error> {
        self.positions("Struct")
    }
}
impl Composable for Datom {
    fn compose<T: Compositional>(&self, budget: &mut Budget) -> Result<T, Error> {
        T::compose(self, budget)
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
                path: Path::new(),
                kind: ErrorKind::Structural(error),
            })?;
        self.reader = Some(protos.clone());
        let datom = protos.datomize(Path::new())?;
        datom.compose(budget)
    }
}

trait Emptying {
    fn empty() -> Self;
}
impl Emptying for protos::Extent {
    fn empty() -> Self {
        Self { start: 0, end: 0 }
    }
}
pub trait ProtosForming {
    fn protos_form(&self) -> protos::Protos;
}
impl ProtosForming for Datom {
    fn protos_form(&self) -> protos::Protos {
        match &self.form {
            Form::Struct(children) => protos::Protos::Enclosed {
                extent: protos::Extent::empty(),
                enclosure: protos::Enclosure::Braced,
                children: children.iter().map(ProtosForming::protos_form).collect(),
            },
            Form::Vector(children) => protos::Protos::Enclosed {
                extent: protos::Extent::empty(),
                enclosure: protos::Enclosure::Bracketed,
                children: children.iter().map(ProtosForming::protos_form).collect(),
            },
            Form::Variant(head, body) => protos::Protos::Headed {
                extent: protos::Extent::empty(),
                head: head.clone(),
                constraints: None,
                separator: protos::Separator::Period,
                body: Box::new(body.protos_form()),
            },
            Form::Bare(text) => protos::Protos::Bare {
                extent: protos::Extent::empty(),
                text: text.clone(),
            },
            Form::String(content) => protos::Protos::Opaque {
                extent: protos::Extent::empty(),
                boundary: protos::Boundary::Guillemets,
                content: content.clone(),
            },
            Form::Meaning(content) => protos::Protos::Opaque {
                extent: protos::Extent::empty(),
                boundary: protos::Boundary::Parentheses,
                content: content.clone(),
            },
        }
    }
}
impl protos::Protosizable for Datom {
    type Output = protos::Protos;
    fn protosize(&self) -> Self::Output {
        self.protos_with_extent(0).0
    }
}
pub trait ProtosExtending {
    fn protos_with_extent(&self, start: usize) -> (protos::Protos, usize);
}
impl ProtosExtending for Datom {
    fn protos_with_extent(&self, start: usize) -> (protos::Protos, usize) {
        use protos::Textualizable;
        match &self.form {
            Form::Bare(text) => {
                let end = start + text.len();
                (
                    protos::Protos::Bare {
                        extent: protos::Extent { start, end },
                        text: text.clone(),
                    },
                    end,
                )
            }
            Form::String(content) | Form::Meaning(content) => {
                let boundary = if matches!(&self.form, Form::String(_)) {
                    protos::Boundary::Guillemets
                } else {
                    protos::Boundary::Parentheses
                };
                let form = protos::Protos::Opaque {
                    extent: protos::Extent { start: 0, end: 0 },
                    boundary,
                    content: content.clone(),
                };
                let end = start + form.textualize().len();
                (
                    protos::Protos::Opaque {
                        extent: protos::Extent { start, end },
                        boundary,
                        content: content.clone(),
                    },
                    end,
                )
            }
            Form::Struct(children) | Form::Vector(children) => {
                let enclosure = if matches!(&self.form, Form::Struct(_)) {
                    protos::Enclosure::Braced
                } else {
                    protos::Enclosure::Bracketed
                };
                let mut offset = start + 2;
                let forms: Vec<_> = children
                    .iter()
                    .enumerate()
                    .map(|(index, child)| {
                        if index > 0 {
                            offset += 1;
                        }
                        let (form, end) = child.protos_with_extent(offset);
                        offset = end;
                        form
                    })
                    .collect();
                let end = if children.is_empty() {
                    start + 2
                } else {
                    offset + 2
                };
                (
                    protos::Protos::Enclosed {
                        extent: protos::Extent { start, end },
                        enclosure,
                        children: forms,
                    },
                    end,
                )
            }
            Form::Variant(head, body) => {
                let prefix = head.0.len() + 1;
                let (body, end) = body.protos_with_extent(start + prefix);
                (
                    protos::Protos::Headed {
                        extent: protos::Extent { start, end },
                        head: head.clone(),
                        constraints: None,
                        separator: protos::Separator::Period,
                        body: Box::new(body),
                    },
                    end,
                )
            }
        }
    }
}

pub trait DatomForming {
    fn datom_form(&self, path: Path) -> Result<Datom, Error>;
}
impl DatomForming for protos::Protos {
    fn datom_form(&self, path: Path) -> Result<Datom, Error> {
        let form = match self {
            protos::Protos::Bare { text, .. } => Form::Bare(text.clone()),
            protos::Protos::Opaque {
                boundary: protos::Boundary::Guillemets,
                content,
                ..
            } => Form::String(content.clone()),
            protos::Protos::Opaque {
                boundary: protos::Boundary::Parentheses,
                content,
                ..
            } => Form::Meaning(content.clone()),
            protos::Protos::Enclosed {
                enclosure: protos::Enclosure::Braced,
                children,
                ..
            } => Form::Struct(
                children
                    .iter()
                    .enumerate()
                    .map(|(index, child)| child.datom_form(path.child(index as Integer)))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            protos::Protos::Enclosed {
                enclosure: protos::Enclosure::Bracketed,
                children,
                ..
            } => Form::Vector(
                children
                    .iter()
                    .enumerate()
                    .map(|(index, child)| child.datom_form(path.child(index as Integer)))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            protos::Protos::Enclosed {
                enclosure: protos::Enclosure::Angled,
                ..
            } => {
                return Err(Error {
                    path,
                    kind: ErrorKind::Form {
                        expected: "Datom enclosure",
                        found: "Angled",
                    },
                });
            }
            protos::Protos::Headed {
                constraints: Some(_),
                ..
            } => {
                return Err(Error {
                    path,
                    kind: ErrorKind::Form {
                        expected: "unqualified Variant",
                        found: "qualified head",
                    },
                });
            }
            protos::Protos::Headed {
                head,
                constraints: None,
                separator: protos::Separator::Period,
                body,
                ..
            } if head.0.parse::<i64>().is_ok()
                && matches!(body.as_ref(), protos::Protos::Bare { text, .. } if text.chars().all(|character| character.is_ascii_digit())) =>
            {
                let protos::Protos::Bare { text, .. } = body.as_ref() else {
                    unreachable!()
                };
                Form::Bare(format!("{}.{}", head.0, text))
            }
            protos::Protos::Headed {
                head,
                constraints: None,
                separator: protos::Separator::Period,
                body,
                ..
            } => Form::Variant(head.clone(), Box::new(body.datom_form(path.child(1))?)),
            protos::Protos::Headed { .. } => Form::Bare(protos::Textualizable::textualize(self)),
        };
        Ok(Datom { path, form })
    }
}
impl Datomizable for protos::Protos {
    type Output = Result<Datom, Error>;
    fn datomize(&self, at: Path) -> Self::Output {
        self.datom_form(at)
    }
}

impl Datomizable for String {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        let form = if self.is_empty()
            || self.chars().any(|character| {
                character.is_whitespace()
                    || matches!(
                        character,
                        '{' | '}' | '[' | ']' | '<' | '>' | '«' | '»' | '(' | ')' | ';' | '.' | '!'
                    )
            }) {
            Form::String(self.clone())
        } else {
            Form::Bare(self.clone())
        };
        Datom { path: at, form }
    }
}
impl Compositional for String {
    const ARITY: Integer = 0;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!("strings compose from a scalar form")
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        Self::scalar(datom, budget)
    }
}
impl Datomizable for i64 {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        Datom {
            path: at,
            form: Form::Bare(self.to_string()),
        }
    }
}
impl Compositional for i64 {
    const ARITY: Integer = 0;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!("integers compose from a bare form")
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        Self::scalar(datom, budget)
    }
}
impl Datomizable for f64 {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        assert!(self.is_finite(), "Datom decimals are finite");
        let mut text = self.to_string();
        if !text.contains('.') {
            text.push_str(".0");
        }
        Datom {
            path: at,
            form: Form::Bare(text),
        }
    }
}
impl Compositional for f64 {
    const ARITY: Integer = 0;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!("decimals compose from a bare form")
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        Self::scalar(datom, budget)
    }
}
impl Datomizable for bool {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        Datom {
            path: at,
            form: Form::Bare(if *self { "True".into() } else { "False".into() }),
        }
    }
}
impl Compositional for bool {
    const ARITY: Integer = 0;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!("booleans compose from a bare form")
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        Self::scalar(datom, budget)
    }
}

impl<T: Datomizable<Output = Datom>> Datomizable for Vec<T> {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        Datom {
            path: at.clone(),
            form: Form::Vector(
                self.iter()
                    .enumerate()
                    .map(|(index, value)| value.datomize(at.child(index as Integer)))
                    .collect(),
            ),
        }
    }
}
impl<T: Compositional> Compositional for Vec<T> {
    const ARITY: Integer = -1;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!("vectors compose from a vector form")
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        let children = match &datom.form {
            Form::Vector(children) => children,
            found => {
                return Err(Error {
                    path: datom.path.clone(),
                    kind: ErrorKind::Form {
                        expected: "Vector",
                        found: found.form_name(),
                    },
                });
            }
        };
        children.iter().map(|child| child.compose(budget)).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Meaning(pub Opaque);
impl Datomizable for Meaning {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        Datom {
            path: at,
            form: Form::Meaning(self.0.clone()),
        }
    }
}
impl Compositional for Meaning {
    const ARITY: Integer = 0;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!("meaning composes from a meaning form")
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        match &datom.form {
            Form::Meaning(value) => Ok(Self(value.clone())),
            found => Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Form {
                    expected: "Meaning",
                    found: found.form_name(),
                },
            }),
        }
    }
}

impl<T: Datomizable<Output = Datom>> Datomizable for Box<T> {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        self.as_ref().datomize(at)
    }
}
impl<T: Compositional> Compositional for Box<T> {
    const ARITY: Integer = T::ARITY;
    fn from_positions(positions: Positions<'_>, budget: &mut Budget) -> Result<Self, Error> {
        Ok(Box::new(T::from_positions(positions, budget)?))
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        Ok(Box::new(datom.compose(budget)?))
    }
}

pub trait Variantizing {
    fn named_variant(self, at: Path, name: &str) -> Datom;
    fn variant(&self, budget: &mut Budget, expected: &'static str)
    -> Result<(&str, &Datom), Error>;
}
impl Variantizing for Datom {
    fn named_variant(self, at: Path, name: &str) -> Datom {
        Datom {
            path: at,
            form: Form::Variant(Symbol(name.to_owned()), Box::new(self)),
        }
    }
    fn variant(
        &self,
        budget: &mut Budget,
        expected: &'static str,
    ) -> Result<(&str, &Datom), Error> {
        budget.spend(&self.path)?;
        match &self.form {
            Form::Variant(head, body) => Ok((&head.0, body)),
            found => Err(Error {
                path: self.path.clone(),
                kind: ErrorKind::Form {
                    expected,
                    found: found.form_name(),
                },
            }),
        }
    }
}
impl<T: Datomizable<Output = Datom>> Datomizable for Option<T> {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        match self {
            Some(value) => value
                .datomize(at.child(1))
                .named_variant(at.clone(), "Some"),
            None => Datom {
                path: at,
                form: Form::Bare("None".to_owned()),
            },
        }
    }
}
impl<T: Compositional> Compositional for Option<T> {
    const ARITY: Integer = 1;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!("options compose from a variant form")
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        if matches!(&datom.form, Form::Bare(head) if head == "None") {
            budget.spend(&datom.path)?;
            return Ok(None);
        }
        let (head, body) = datom.variant(budget, "Variant")?;
        match head {
            "Some" => Ok(Some(body.compose(budget)?)),
            "None" if matches!(body.form, Form::Bare(ref value) if value.is_empty()) => Ok(None),
            found => Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "Option",
                    found: found.to_owned(),
                },
            }),
        }
    }
}
impl<T: Datomizable<Output = Datom>, E: Datomizable<Output = Datom>> Datomizable for Result<T, E> {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        match self {
            Ok(value) => value.datomize(at.child(1)).named_variant(at.clone(), "Ok"),
            Err(value) => value.datomize(at.child(1)).named_variant(at.clone(), "Err"),
        }
    }
}
impl<T: Compositional, E: Compositional> Compositional for Result<T, E> {
    const ARITY: Integer = 1;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!("results compose from a variant form")
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        let (head, body) = datom.variant(budget, "Variant")?;
        match head {
            "Ok" => Ok(Ok(body.compose(budget)?)),
            "Err" => Ok(Err(body.compose(budget)?)),
            found => Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "Result",
                    found: found.to_owned(),
                },
            }),
        }
    }
}

impl Datomizable for protos::Extent {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        Datom {
            path: at.clone(),
            form: Form::Struct(vec![
                (self.start as i64).datomize(at.child(0)),
                (self.end as i64).datomize(at.child(1)),
            ]),
        }
        .named_variant(at, "Extent")
    }
}
impl Compositional for protos::Extent {
    const ARITY: Integer = 2;
    fn from_positions(mut positions: Positions<'_>, budget: &mut Budget) -> Result<Self, Error> {
        let start: i64 = positions.position(budget)?;
        let end: i64 = positions.position(budget)?;
        positions.finish()?;
        Ok(Self {
            start: start.try_into().map_err(|_| Error {
                path: Path::new(),
                kind: ErrorKind::Value {
                    expected: "non-negative extent",
                    value: start.to_string(),
                },
            })?,
            end: end.try_into().map_err(|_| Error {
                path: Path::new(),
                kind: ErrorKind::Value {
                    expected: "non-negative extent",
                    value: end.to_string(),
                },
            })?,
        })
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        let (head, body) = datom.variant(budget, "Extent")?;
        if head != "Extent" {
            return Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "Extent",
                    found: head.to_owned(),
                },
            });
        }
        Self::from_positions(body.positions("Struct")?, budget)
    }
}

impl Datomizable for protos::Separator {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        let name = match self {
            Self::Period => "Period",
            Self::Exclamation => "Exclamation",
            Self::Colon => "Colon",
        };
        Datom {
            path: at,
            form: Form::Bare(name.to_owned()),
        }
    }
}
impl Compositional for protos::Separator {
    const ARITY: Integer = 0;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!()
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        match &datom.form {
            Form::Bare(name) if name == "Period" => Ok(Self::Period),
            Form::Bare(name) if name == "Exclamation" => Ok(Self::Exclamation),
            Form::Bare(name) if name == "Colon" => Ok(Self::Colon),
            Form::Bare(name) => Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "Separator",
                    found: name.clone(),
                },
            }),
            found => Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Form {
                    expected: "Separator",
                    found: found.form_name(),
                },
            }),
        }
    }
}

impl Datomizable for protos::Problem {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        match self {
            Self::Empty
            | Self::Multiple
            | Self::MissingHead
            | Self::MissingBody
            | Self::Budget
            | Self::Depth => {
                let name = match self {
                    Self::Empty => "Empty",
                    Self::Multiple => "Multiple",
                    Self::MissingHead => "MissingHead",
                    Self::MissingBody => "MissingBody",
                    Self::Budget => "Budget",
                    Self::Depth => "Depth",
                    _ => unreachable!(),
                };
                Datom {
                    path: at,
                    form: Form::Bare(name.to_owned()),
                }
            }
            Self::Unclosed(character) => character
                .to_string()
                .datomize(at.child(1))
                .named_variant(at, "Unclosed"),
            Self::Unexpected(character) => character
                .to_string()
                .datomize(at.child(1))
                .named_variant(at, "Unexpected"),
        }
    }
}
impl Compositional for protos::Problem {
    const ARITY: Integer = 1;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!()
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        if let Form::Bare(name) = &datom.form {
            budget.spend(&datom.path)?;
            return match name.as_str() {
                "Empty" => Ok(Self::Empty),
                "Multiple" => Ok(Self::Multiple),
                "MissingHead" => Ok(Self::MissingHead),
                "MissingBody" => Ok(Self::MissingBody),
                "Budget" => Ok(Self::Budget),
                "Depth" => Ok(Self::Depth),
                _ => Err(Error {
                    path: datom.path.clone(),
                    kind: ErrorKind::Variant {
                        expected: "Problem",
                        found: name.clone(),
                    },
                }),
            };
        }
        let (head, body) = datom.variant(budget, "Problem")?;
        let character: String = body.compose(budget)?;
        let mut characters = character.chars();
        let Some(character) = characters.next().filter(|_| characters.next().is_none()) else {
            return Err(Error {
                path: body.path.clone(),
                kind: ErrorKind::Value {
                    expected: "one character",
                    value: character,
                },
            });
        };
        match head {
            "Unclosed" => Ok(Self::Unclosed(character)),
            "Unexpected" => Ok(Self::Unexpected(character)),
            found => Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "Problem",
                    found: found.to_owned(),
                },
            }),
        }
    }
}

impl Datomizable for protos::Error {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        Datom {
            path: at.clone(),
            form: Form::Struct(vec![
                self.extent.datomize(at.child(1).child(0)),
                self.problem.datomize(at.child(1).child(1)),
            ]),
        }
        .named_variant(at, "ProtosError")
    }
}
impl Compositional for protos::Error {
    const ARITY: Integer = 2;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!()
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        let (head, body) = datom.variant(budget, "ProtosError")?;
        if head != "ProtosError" {
            return Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "ProtosError",
                    found: head.to_owned(),
                },
            });
        }
        let mut positions = body.positions("Struct")?;
        let extent = positions.position(budget)?;
        let problem = positions.position(budget)?;
        positions.finish()?;
        Ok(Self { extent, problem })
    }
}

pub trait Scalar: Sized {
    fn scalar(datom: &Datom, budget: &mut Budget) -> Result<Self, Error>;
}
impl Scalar for String {
    fn scalar(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        match &datom.form {
            Form::Bare(value) | Form::String(value) | Form::Meaning(value) => Ok(value.clone()),
            found => Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Form {
                    expected: "String",
                    found: found.form_name(),
                },
            }),
        }
    }
}
impl Scalar for i64 {
    fn scalar(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        let value = match &datom.form {
            Form::Bare(value) => value,
            found => {
                return Err(Error {
                    path: datom.path.clone(),
                    kind: ErrorKind::Form {
                        expected: "Bare",
                        found: found.form_name(),
                    },
                });
            }
        };
        if value.starts_with("-0") || (value.len() > 1 && value.starts_with('0')) {
            return Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Value {
                    expected: "Integer",
                    value: value.clone(),
                },
            });
        }
        i64::from_str(value).map_err(|_| Error {
            path: datom.path.clone(),
            kind: ErrorKind::Value {
                expected: "Integer",
                value: value.clone(),
            },
        })
    }
}
impl Scalar for bool {
    fn scalar(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        match &datom.form {
            Form::Bare(value) if value == "True" => Ok(true),
            Form::Bare(value) if value == "False" => Ok(false),
            Form::Bare(value) => Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Value {
                    expected: "Boolean",
                    value: value.clone(),
                },
            }),
            found => Err(Error {
                path: datom.path.clone(),
                kind: ErrorKind::Form {
                    expected: "Bare",
                    found: found.form_name(),
                },
            }),
        }
    }
}
impl Scalar for f64 {
    fn scalar(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        let value = match &datom.form {
            Form::Bare(value) if value.contains('.') => value,
            Form::Bare(value) => {
                return Err(Error {
                    path: datom.path.clone(),
                    kind: ErrorKind::Value {
                        expected: "Decimal",
                        value: value.clone(),
                    },
                });
            }
            found => {
                return Err(Error {
                    path: datom.path.clone(),
                    kind: ErrorKind::Form {
                        expected: "Bare",
                        found: found.form_name(),
                    },
                });
            }
        };
        value
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(|| Error {
                path: datom.path.clone(),
                kind: ErrorKind::Value {
                    expected: "Decimal",
                    value: value.clone(),
                },
            })
    }
}
