//! Conception: the situated protoform tree becomes the situated datom tree, in
//! one iterative walk that carries the path down and the built nodes up.

use protos::{
    Boundary, Conceivable, Delineation, Enclosure, Extent, Head, Integer, Locating, Path,
    Protoform, Separator, Situated, Situation, Textualizable,
};

use crate::anatomy::{Datom, Fault, Found, Locus, Problem};

/// What a finished node becomes, once its children are built.
enum Node<'a> {
    Variant(&'a str),
    Struct,
    Vector,
}

/// One step of the walk.
enum Step<'a> {
    /// Visit a structure at its situation, as the child at `index` of the node above.
    Visit {
        form: &'a Protoform,
        at: &'a Situation,
        index: Option<Integer>,
    },
    /// Finish a node from its last `arity` built children.
    Finish {
        node: Node<'a>,
        arity: usize,
        at: &'a Situation,
    },
}

/// The walk's state: the steps to take, the path so far, the nodes built so far.
struct Walk<'a> {
    steps: Vec<Step<'a>>,
    path: Path,
    datoms: Vec<Datom>,
    situations: Vec<Situation>,
}

/// The kind whose capability says whether a headed structure is a chain of words.
trait Wordy {
    fn is_word_chain(&self) -> bool;
}

impl Wordy for Protoform {
    fn is_word_chain(&self) -> bool {
        let mut here = self;
        loop {
            match here {
                Protoform::Bare(Head::Symbol(_)) => return true,
                Protoform::Headed(Head::Symbol(_), _, body) => here = body,
                _ => return false,
            }
        }
    }
}

/// The kind whose capabilities take the walk's steps.
trait Walking<'a> {
    fn leaf(&mut self, datom: Datom, at: &'a Situation);
    fn visit(&mut self, form: &'a Protoform, at: &'a Situation) -> Result<(), Fault>;
    fn finish(&mut self, node: Node<'a>, arity: usize, at: &'a Situation);
    fn formless(&self, found: Found, at: &'a Situation) -> Fault;
    fn walk(self) -> Result<Situated<Datom>, Fault>;
}

impl<'a> Walking<'a> for Walk<'a> {
    fn leaf(&mut self, datom: Datom, at: &'a Situation) {
        self.datoms.push(datom);
        self.situations.push(Situation {
            extent: at.extent,
            children: vec![],
        });
        self.path.pop();
    }

    fn visit(&mut self, form: &'a Protoform, at: &'a Situation) -> Result<(), Fault> {
        match form {
            Protoform::Headed(Head::Symbol(head), Separator::Period, body) => {
                self.steps.push(Step::Finish {
                    node: Node::Variant(head),
                    arity: 1,
                    at,
                });
                self.steps.push(Step::Visit {
                    form: body,
                    at: at.part(1),
                    index: Some(1),
                });
            }
            Protoform::Headed(Head::Symbol(_), _, _) if form.is_word_chain() => {
                self.leaf(Datom::Word(form.textualize()), at);
            }
            Protoform::Headed(Head::Symbol(_), _, _) => return Err(self.formless(Found::Chain, at)),
            Protoform::Headed(Head::Qualified(..), _, _) | Protoform::Bare(Head::Qualified(..)) => {
                return Err(self.formless(Found::Qualified, at));
            }
            Protoform::Enclosed(Enclosure::Angled, _) => {
                return Err(self.formless(Found::Angled, at));
            }
            Protoform::Enclosed(enclosure, children) => {
                let node = match enclosure {
                    Enclosure::Braced => Node::Struct,
                    _ => Node::Vector,
                };
                self.steps.push(Step::Finish {
                    node,
                    arity: children.len(),
                    at,
                });
                for (index, child) in children.iter().enumerate().rev() {
                    let index = index as Integer;
                    self.steps.push(Step::Visit {
                        form: child,
                        at: at.part(index),
                        index: Some(index),
                    });
                }
            }
            Protoform::Opaque(Boundary::CurlyQuotes, text) => {
                self.leaf(Datom::Text(text.clone()), at)
            }
            Protoform::Opaque(Boundary::Parentheses, text) => {
                self.leaf(Datom::Meaning(text.clone()), at)
            }
            Protoform::Bare(Head::Symbol(symbol)) => self.leaf(Datom::Word(symbol.clone()), at),
        }
        Ok(())
    }

    fn finish(&mut self, node: Node<'a>, arity: usize, at: &'a Situation) {
        let datoms = self.datoms.split_off(self.datoms.len() - arity);
        let mut children = self.situations.split_off(self.situations.len() - arity);
        let datom = match node {
            Node::Variant(head) => {
                let body = datoms
                    .into_iter()
                    .next()
                    .unwrap_or(Datom::Word(String::new()));
                children.insert(
                    0,
                    Situation {
                        extent: at.part(0).extent,
                        children: vec![],
                    },
                );
                Datom::Variant(head.to_owned(), Box::new(body))
            }
            Node::Struct => Datom::Struct(datoms),
            Node::Vector => Datom::Vector(datoms),
        };
        self.datoms.push(datom);
        self.situations.push(Situation {
            extent: at.extent,
            children,
        });
        self.path.pop();
    }

    fn formless(&self, found: Found, at: &'a Situation) -> Fault {
        Fault::Conceptual(
            Locus {
                path: self.path.clone(),
                extent: at.extent,
            },
            Problem::Formless(found),
        )
    }

    fn walk(mut self) -> Result<Situated<Datom>, Fault> {
        while let Some(step) = self.steps.pop() {
            match step {
                Step::Visit { form, at, index } => {
                    if let Some(index) = index {
                        self.path.push(index);
                    }
                    self.visit(form, at)?;
                }
                Step::Finish { node, arity, at } => self.finish(node, arity, at),
            }
        }
        let datom = self.datoms.pop().unwrap_or(Datom::Word(String::new()));
        let situation = self.situations.pop().unwrap_or(Situation {
            extent: Extent(0, 0),
            children: vec![],
        });
        Ok(Situated(situation, datom))
    }
}

impl Conceivable<Datom> for Situated<Protoform> {
    type Fault = Fault;

    fn conceive(&self) -> Result<Situated<Datom>, Fault> {
        let Situated(at, form) = self;
        Walk {
            steps: vec![Step::Visit {
                form,
                at,
                index: None,
            }],
            path: vec![],
            datoms: vec![],
            situations: vec![],
        }
        .walk()
    }
}

impl Conceivable<Datom> for Delineation {
    type Fault = Fault;

    fn conceive(&self) -> Result<Situated<Datom>, Fault> {
        match self.0.as_slice() {
            [structure] => structure.conceive(),
            [] => Err(Fault::Conceptual(
                Locus {
                    path: vec![],
                    extent: Extent(0, 0),
                },
                Problem::OneValue(0),
            )),
            [first, .., last] => Err(Fault::Conceptual(
                Locus {
                    path: vec![],
                    extent: Extent(first.0.extent.0, last.0.extent.1),
                },
                Problem::OneValue(self.0.len() as Integer),
            )),
        }
    }
}
