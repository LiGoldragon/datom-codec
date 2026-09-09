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
            Form::Struct(children) | Form::Vector(children) => children,
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
pub struct Potential<T>(pub String, PhantomData<fn() -> T>);
impl<T> From<&str> for Potential<T> {
    fn from(text: &str) -> Self {
        Self(text.to_owned(), PhantomData)
    }
}
impl<T> From<String> for Potential<T> {
    fn from(text: String) -> Self {
        Self(text, PhantomData)
    }
}
pub trait Actualizing<T> {
    fn actualize(&self, budget: &mut Budget) -> Result<T, Error>;
}
impl<T: Compositional> Actualizing<T> for Potential<T> {
    fn actualize(&self, budget: &mut Budget) -> Result<T, Error> {
        use protos::BoundedProtosizable;
        let protos = self
            .0
            .protosize_with(&mut budget.reader)
            .map_err(|error| Error {
                path: Path::new(),
                kind: ErrorKind::Structural(error),
            })?;
        protos.datomize(Path::new()).compose(budget)
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
    fn protosize(&self) -> Result<protos::Protos, protos::Error> {
        Ok(self.protos_form())
    }
}

pub trait DatomForming {
    fn datom_form(&self, path: Path) -> Datom;
}
impl DatomForming for protos::Protos {
    fn datom_form(&self, path: Path) -> Datom {
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
                    .collect(),
            ),
            protos::Protos::Enclosed { children, .. } => Form::Vector(
                children
                    .iter()
                    .enumerate()
                    .map(|(index, child)| child.datom_form(path.child(index as Integer)))
                    .collect(),
            ),
            protos::Protos::Headed {
                head,
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
                separator: protos::Separator::Period,
                body,
                ..
            } => Form::Variant(head.clone(), Box::new(body.datom_form(path.child(1)))),
            protos::Protos::Headed { .. } => Form::Bare(protos::Textualizable::textualize(self)),
        };
        Datom { path, form }
    }
}
impl Datomizable for protos::Protos {
    fn datomize(&self, at: Path) -> Datom {
        self.datom_form(at)
    }
}

impl Datomizable for String {
    fn datomize(&self, at: Path) -> Datom {
        let form = if self.is_empty() || self.chars().any(char::is_whitespace) {
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
    fn datomize(&self, at: Path) -> Datom {
        assert!(self.is_finite(), "Datom decimals are finite");
        Datom {
            path: at,
            form: Form::Bare(self.to_string()),
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

impl<T: Datomizable> Datomizable for Vec<T> {
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

impl<T: Datomizable> Datomizable for Box<T> {
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
impl<T: Datomizable> Datomizable for Option<T> {
    fn datomize(&self, at: Path) -> Datom {
        match self {
            Some(value) => value
                .datomize(at.child(1))
                .named_variant(at.clone(), "Some"),
            None => Datom {
                path: at.child(1),
                form: Form::Bare(String::new()),
            }
            .named_variant(at.clone(), "None"),
        }
    }
}
impl<T: Compositional> Compositional for Option<T> {
    const ARITY: Integer = 1;
    fn from_positions(_: Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!("options compose from a variant form")
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
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
impl<T: Datomizable, E: Datomizable> Datomizable for Result<T, E> {
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

pub trait Scalar: Sized {
    fn scalar(datom: &Datom, budget: &mut Budget) -> Result<Self, Error>;
}
impl Scalar for String {
    fn scalar(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        match &datom.form {
            Form::Bare(value) | Form::String(value) => Ok(value.clone()),
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
        if value == "-0" || (value.len() > 1 && value.starts_with('0')) {
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
