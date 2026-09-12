use std::str::FromStr;

use crate::*;

pub trait DatomForming {
    fn datom_form(&self, path: Path) -> Result<Datom, Error>;
}

const MAXIMUM_FORMING_DEPTH: Integer = 4_096;

enum Forming<'a> {
    Visit(&'a protos::Protos, Path, Integer),
    Enclosed(Path, bool, usize),
    Headed(Path, protos::Symbol),
}
struct Former<'a> {
    work: Vec<Forming<'a>>,
    values: Vec<Datom>,
}
trait FormingDatoms<'a> {
    fn form(root: &'a protos::Protos, path: Path) -> Result<Datom, Error>;
}
impl<'a> FormingDatoms<'a> for Former<'a> {
    fn form(root: &'a protos::Protos, path: Path) -> Result<Datom, Error> {
        let mut former = Self {
            work: vec![Forming::Visit(root, path, 0)],
            values: Vec::new(),
        };
        while let Some(work) = former.work.pop() {
            match work {
                Forming::Visit(protos, path, depth) => {
                    if depth >= MAXIMUM_FORMING_DEPTH {
                        return Err(Error {
                            layer: ErrorLayer::Datom,
                            path,
                            kind: ErrorKind::Budget,
                        });
                    }
                    match protos {
                        protos::Protos::Bare { text, .. } => former.values.push(Datom {
                            path,
                            form: Form::Bare(text.clone()),
                        }),
                        protos::Protos::Opaque {
                            boundary: protos::Boundary::Guillemets,
                            content,
                            ..
                        } => former.values.push(Datom {
                            path,
                            form: Form::String(content.clone()),
                        }),
                        protos::Protos::Opaque {
                            boundary: protos::Boundary::Parentheses,
                            content,
                            ..
                        } => former.values.push(Datom {
                            path,
                            form: Form::Meaning(content.clone()),
                        }),
                        protos::Protos::Enclosed {
                            enclosure: protos::Enclosure::Braced,
                            children,
                            ..
                        }
                        | protos::Protos::Enclosed {
                            enclosure: protos::Enclosure::Bracketed,
                            children,
                            ..
                        } => {
                            let structure = matches!(
                                protos,
                                protos::Protos::Enclosed {
                                    enclosure: protos::Enclosure::Braced,
                                    ..
                                }
                            );
                            former.work.push(Forming::Enclosed(
                                path.clone(),
                                structure,
                                children.len(),
                            ));
                            for (index, child) in children.iter().enumerate().rev() {
                                former.work.push(Forming::Visit(
                                    child,
                                    path.child(index as Integer),
                                    depth + 1,
                                ));
                            }
                        }
                        protos::Protos::Enclosed {
                            enclosure: protos::Enclosure::Angled,
                            ..
                        } => {
                            return Err(Error {
                                layer: ErrorLayer::Datom,
                                path,
                                kind: ErrorKind::Form {
                                    expected: "Datom enclosure".into(),
                                    found: "Angled".into(),
                                },
                            });
                        }
                        protos::Protos::Headed {
                            constraints: Some(_),
                            ..
                        } => {
                            return Err(Error {
                                layer: ErrorLayer::Datom,
                                path,
                                kind: ErrorKind::Form {
                                    expected: "unqualified Variant".into(),
                                    found: "qualified head".into(),
                                },
                            });
                        }
                        protos::Protos::Headed {
                            head,
                            constraints: None,
                            separator: protos::Separator::Period,
                            body,
                            ..
                        } => {
                            former
                                .work
                                .push(Forming::Headed(path.clone(), head.clone()));
                            former
                                .work
                                .push(Forming::Visit(body, path.child(1), depth + 1));
                        }
                        protos::Protos::Headed { .. } => former.values.push(Datom {
                            path,
                            form: Form::Bare(protos::Textualizable::textualize(protos)),
                        }),
                    }
                }
                Forming::Enclosed(path, structure, count) => {
                    let children = former.values.split_off(former.values.len() - count);
                    former.values.push(Datom {
                        path,
                        form: if structure {
                            Form::Struct(children)
                        } else {
                            Form::Vector(children)
                        },
                    });
                }
                Forming::Headed(path, head) => {
                    let body = former.values.pop().expect("formed headed body");
                    former.values.push(Datom {
                        path,
                        form: Form::Variant(head, Box::new(body)),
                    });
                }
            }
        }
        Ok(former.values.pop().expect("formed root"))
    }
}
impl DatomForming for protos::Protos {
    fn datom_form(&self, path: Path) -> Result<Datom, Error> {
        Former::form(self, path)
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
        let separator = |character: char| matches!(character, '.' | '!' | ':');
        let delimited = self.is_empty()
            || self.chars().any(|character| {
                character.is_whitespace()
                    || matches!(
                        character,
                        '{' | '}' | '[' | ']' | '<' | '>' | '«' | '»' | '(' | ')' | ';'
                    )
            })
            || self.starts_with(separator)
            || self.ends_with(separator)
            || self
                .chars()
                .zip(self.chars().skip(1))
                .any(|(left, right)| separator(left) && separator(right));
        let form = if delimited {
            Form::String(self.clone())
        } else {
            Form::Bare(self.clone())
        };
        Datom { path: at, form }
    }
}
impl Composing for String {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        datom.compose_bare_string(budget)
    }
}

trait BareStringComposing {
    fn compose_bare_string(&self, budget: &mut Budget) -> Result<String, Error>;
}

impl BareStringComposing for Datom {
    fn compose_bare_string(&self, budget: &mut Budget) -> Result<String, Error> {
        let mut current = self;
        let mut depth = budget.depth;
        let mut value = String::new();
        loop {
            budget.spend(&current.path)?;
            let text = match &current.form {
                Form::Bare(text) | Form::String(text) if value.is_empty() => {
                    return Ok(text.clone());
                }
                Form::Bare(text) => text,
                Form::Variant(head, body) => {
                    if depth >= budget.maximum_depth {
                        return Err(Error::composition(body.path.clone(), ErrorKind::Budget));
                    }
                    depth += 1;
                    if head.0.is_empty()
                        || head.0.chars().any(|character| {
                            character.is_whitespace()
                                || matches!(
                                    character,
                                    '{' | '}' | '[' | ']' | '<' | '>' | '«' | '»' | '(' | ')' | ';'
                                )
                        })
                    {
                        return Err(Error::composition(
                            current.path.clone(),
                            ErrorKind::Form {
                                expected: "String".into(),
                                found: "Variant".into(),
                            },
                        ));
                    }
                    value.push_str(&head.0);
                    value.push('.');
                    current = body;
                    continue;
                }
                found => {
                    return Err(Error::composition(
                        current.path.clone(),
                        ErrorKind::Form {
                            expected: "String".into(),
                            found: found.form_name().to_owned(),
                        },
                    ));
                }
            };
            if text.is_empty()
                || text.chars().any(|character| {
                    character.is_whitespace()
                        || matches!(
                            character,
                            '{' | '}' | '[' | ']' | '<' | '>' | '«' | '»' | '(' | ')' | ';'
                        )
                })
            {
                return Err(Error::composition(
                    current.path.clone(),
                    ErrorKind::Form {
                        expected: "bare String".into(),
                        found: current.form.form_name().to_owned(),
                    },
                ));
            }
            value.push_str(text);
            return Ok(value);
        }
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
impl Composing for i64 {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        Self::scalar(datom, budget)
    }
}
impl Datomizable for f64 {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        let mut text = self.to_string();
        if self.is_finite() && !text.contains('.') {
            text.push_str(".0");
        }
        Datom {
            path: at,
            form: Form::Bare(text),
        }
    }
}
impl Composing for f64 {
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
impl Composing for bool {
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
impl<T: Composing> Composing for Vec<T> {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        let children = match &datom.form {
            Form::Vector(children) => children,
            found => {
                return Err(Error {
                    layer: ErrorLayer::Composition,
                    path: datom.path.clone(),
                    kind: ErrorKind::Form {
                        expected: "Vector".into(),
                        found: found.form_name().to_owned(),
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
impl Composing for Meaning {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        match &datom.form {
            Form::Meaning(value) => Ok(Self(value.clone())),
            found => Err(Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Form {
                    expected: "Meaning".into(),
                    found: found.form_name().to_owned(),
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
impl<T: Composing> Composing for Box<T> {
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
                layer: ErrorLayer::Composition,
                path: self.path.clone(),
                kind: ErrorKind::Form {
                    expected: expected.to_owned(),
                    found: found.form_name().to_owned(),
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
impl<T: Composing> Composing for Option<T> {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        if matches!(&datom.form, Form::Bare(head) if head == "None") {
            budget.spend(&datom.path)?;
            return Ok(None);
        }
        let (head, body) = datom.variant(budget, "Variant")?;
        match head {
            "Some" => Ok(Some(body.compose(budget)?)),
            found => Err(Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "Option".into(),
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
impl<T: Composing, E: Composing> Composing for Result<T, E> {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        let (head, body) = datom.variant(budget, "Variant")?;
        match head {
            "Ok" => Ok(Ok(body.compose(budget)?)),
            "Err" => Ok(Err(body.compose(budget)?)),
            found => Err(Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "Result".into(),
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
            path: at.child(1),
            form: Form::Struct(vec![
                (self.start as i64).datomize(at.child(1).child(0)),
                (self.end as i64).datomize(at.child(1).child(1)),
            ]),
        }
        .named_variant(at, "Extent")
    }
}
impl Composing for protos::Extent {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        let (head, body) = datom.variant(budget, "Extent")?;
        if head != "Extent" {
            return Err(Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "Extent".into(),
                    found: head.to_owned(),
                },
            });
        }
        let (start, end): (i64, i64) = body.compose_positions(budget)?;
        Ok(Self {
            start: start.try_into().map_err(|_| Error {
                layer: ErrorLayer::Composition,
                path: Path::new(),
                kind: ErrorKind::Value {
                    expected: "non-negative extent".into(),
                    value: start.to_string(),
                },
            })?,
            end: end.try_into().map_err(|_| Error {
                layer: ErrorLayer::Composition,
                path: Path::new(),
                kind: ErrorKind::Value {
                    expected: "non-negative extent".into(),
                    value: end.to_string(),
                },
            })?,
        })
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
impl Composing for protos::Separator {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;
        match &datom.form {
            Form::Bare(name) if name == "Period" => Ok(Self::Period),
            Form::Bare(name) if name == "Exclamation" => Ok(Self::Exclamation),
            Form::Bare(name) if name == "Colon" => Ok(Self::Colon),
            Form::Bare(name) => Err(Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "Separator".into(),
                    found: name.clone(),
                },
            }),
            found => Err(Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Form {
                    expected: "Separator".into(),
                    found: found.form_name().to_owned(),
                },
            }),
        }
    }
}

impl Datomizable for protos::Symbol {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        self.0.datomize(at)
    }
}
impl Composing for protos::Symbol {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        Ok(Self(String::compose(datom, budget)?))
    }
}

impl Datomizable for protos::ReaderBudget {
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        (self.remaining as Integer).datomize(at)
    }
}
impl Composing for protos::ReaderBudget {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        let remaining: Integer = datom.compose(budget)?;
        Ok(Self {
            remaining: remaining.try_into().map_err(|_| Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Value {
                    expected: "non-negative reader budget".into(),
                    value: remaining.to_string(),
                },
            })?,
        })
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
impl Composing for protos::Problem {
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
                    layer: ErrorLayer::Composition,
                    path: datom.path.clone(),
                    kind: ErrorKind::Variant {
                        expected: "Problem".into(),
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
                layer: ErrorLayer::Composition,
                path: body.path.clone(),
                kind: ErrorKind::Value {
                    expected: "one character".into(),
                    value: character,
                },
            });
        };
        match head {
            "Unclosed" => Ok(Self::Unclosed(character)),
            "Unexpected" => Ok(Self::Unexpected(character)),
            found => Err(Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "Problem".into(),
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
            path: at.child(1),
            form: Form::Struct(vec![
                self.extent.datomize(at.child(1).child(0)),
                self.problem.datomize(at.child(1).child(1)),
            ]),
        }
        .named_variant(at, "ProtosError")
    }
}
impl Composing for protos::Error {
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        let (head, body) = datom.variant(budget, "ProtosError")?;
        if head != "ProtosError" {
            return Err(Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Variant {
                    expected: "ProtosError".into(),
                    found: head.to_owned(),
                },
            });
        }
        body.compose_positions(budget)
    }
}
impl Compositional for protos::Error {
    const ARITY: Integer = 2;
    fn from_positions(mut positions: Positions<'_>) -> Result<Self, Error> {
        Ok(Self {
            extent: positions.position()?,
            problem: positions.position()?,
        })
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
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Form {
                    expected: "String".into(),
                    found: found.form_name().to_owned(),
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
                    layer: ErrorLayer::Composition,
                    path: datom.path.clone(),
                    kind: ErrorKind::Form {
                        expected: "Bare".into(),
                        found: found.form_name().to_owned(),
                    },
                });
            }
        };
        if value.starts_with('+')
            || value.starts_with("-0")
            || (value.len() > 1 && value.starts_with('0'))
        {
            return Err(Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Value {
                    expected: "Integer".into(),
                    value: value.clone(),
                },
            });
        }
        i64::from_str(value).map_err(|_| Error {
            layer: ErrorLayer::Composition,
            path: datom.path.clone(),
            kind: ErrorKind::Value {
                expected: "Integer".into(),
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
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Value {
                    expected: "Boolean".into(),
                    value: value.clone(),
                },
            }),
            found => Err(Error {
                layer: ErrorLayer::Composition,
                path: datom.path.clone(),
                kind: ErrorKind::Form {
                    expected: "Bare".into(),
                    found: found.form_name().to_owned(),
                },
            }),
        }
    }
}
impl Scalar for f64 {
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
        let refusal = || Error {
            layer: ErrorLayer::Composition,
            path: datom.path.clone(),
            kind: ErrorKind::Value {
                expected: "Decimal".into(),
                value: text.clone(),
            },
        };
        let (whole, fraction) = text.split_once('.').ok_or_else(refusal)?;
        let digits = whole.strip_prefix('-').unwrap_or(whole);
        let canonical = !digits.is_empty()
            && digits.chars().all(|character| character.is_ascii_digit())
            && (digits.len() == 1 || !digits.starts_with('0'))
            && !fraction.is_empty()
            && fraction.chars().all(|character| character.is_ascii_digit());
        if !canonical {
            return Err(refusal());
        }
        text.parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(refusal)
    }
}
