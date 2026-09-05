//! Protosization: the datom becomes a protoform in one iterative walk; the
//! writer then computes the text and, when asked, the situation of every node.

use std::convert::Infallible;

use protos::{
    Bare, Classifying, Conceivable, Delineation, Enclosure, Glyph, Head, Protoform, Protosizable,
    Separator, Situated, Situating, Situation, Symbol, Word,
};

use crate::anatomy::Datom;

/// What a finished node becomes, once its children are built.
enum Node<'a> {
    Variant(&'a Symbol),
    Struct,
    Vector,
}

/// One step of the build.
enum Step<'a> {
    /// Visit a datom.
    Visit(&'a Datom),
    /// Finish a node from its last `arity` built children.
    Finish(Node<'a>, usize),
}

/// The build's state: the steps to take, the forms built so far.
struct Build<'a> {
    steps: Vec<Step<'a>>,
    forms: Vec<Protoform>,
}

/// The kind whose capability projects one bare run into the structural chain
/// the Protos reader gives that exact run.
trait BareForming {
    fn bare_form(&self) -> Protoform;
}

impl BareForming for Word {
    fn bare_form(&self) -> Protoform {
        let text = self.as_ref();
        let mut pieces = Vec::new();
        let mut start = 0;
        for (offset, glyph) in text.char_indices() {
            if let Glyph::Separate(separator) = glyph.classify() {
                pieces.push((&text[start..offset], separator));
                start = offset + glyph.len_utf8();
            }
        }
        if pieces.is_empty()
            || pieces.iter().any(|(piece, _)| piece.is_empty())
            || text[start..].is_empty()
        {
            return Protoform::Bare(
                Bare::try_from(text).expect("a non-chain word is structurally bare"),
            );
        }
        let mut form = Protoform::Bare(Bare::try_from(&text[start..]).unwrap());
        for (piece, separator) in pieces.into_iter().rev() {
            form = Protoform::Headed(
                Head::Symbol(Symbol::try_from(piece).unwrap()),
                separator,
                Box::new(form),
            );
        }
        form
    }
}

/// The kind whose capabilities take the build's steps.
trait Building<'a> {
    fn visit(&mut self, datom: &'a Datom);
    fn finish(&mut self, node: Node<'a>, arity: usize);
    fn build(self) -> Protoform;
}

impl<'a> Building<'a> for Build<'a> {
    fn visit(&mut self, datom: &'a Datom) {
        match datom {
            Datom::Variant(head, body) => {
                self.steps.push(Step::Finish(Node::Variant(head), 1));
                self.steps.push(Step::Visit(body));
            }
            Datom::Struct(fields) => {
                self.steps.push(Step::Finish(Node::Struct, fields.len()));
                self.steps.extend(fields.iter().rev().map(Step::Visit));
            }
            Datom::Vector(elements) => {
                self.steps.push(Step::Finish(Node::Vector, elements.len()));
                self.steps.extend(elements.iter().rev().map(Step::Visit));
            }
            Datom::Text(text) => self.forms.push(Protoform::Quoted(text.clone())),
            Datom::Meaning(text) => self.forms.push(Protoform::Parenthesized(text.clone())),
            Datom::Word(word) => self.forms.push(word.bare_form()),
        }
    }

    fn finish(&mut self, node: Node<'a>, arity: usize) {
        let children = self.forms.split_off(self.forms.len() - arity);
        let form = match node {
            Node::Variant(head) => {
                let body = children
                    .into_iter()
                    .next()
                    .unwrap_or(Protoform::Bare(Bare::try_from("_").unwrap()));
                Protoform::Headed(
                    Head::Symbol(head.clone()),
                    Separator::Period,
                    Box::new(body),
                )
            }
            Node::Struct => Protoform::Enclosed(Enclosure::Braced, children),
            Node::Vector => Protoform::Enclosed(Enclosure::Bracketed, children),
        };
        self.forms.push(form);
    }

    fn build(mut self) -> Protoform {
        while let Some(step) = self.steps.pop() {
            match step {
                Step::Visit(datom) => self.visit(datom),
                Step::Finish(node, arity) => self.finish(node, arity),
            }
        }
        self.forms
            .pop()
            .unwrap_or(Protoform::Bare(Bare::try_from("_").unwrap()))
    }
}

/// The kind whose capability builds the protoform of a datom.
pub(crate) trait Forming {
    /// The protoform.
    fn form(&self) -> Protoform;
}

impl Forming for Datom {
    fn form(&self) -> Protoform {
        Build {
            steps: vec![Step::Visit(self)],
            forms: vec![],
        }
        .build()
    }
}

impl Protosizable for Datom {
    type Fault = Infallible;

    fn protosize(&self) -> Result<Delineation, Infallible> {
        let form = self.form();
        let Situated(situation, _) = form.situate();
        Ok(Delineation(vec![Situated(situation, form)]))
    }
}

impl Conceivable<Datom> for Datom {
    type Fault = Infallible;

    fn conceive(&self) -> Result<Situated<Datom>, Self::Fault> {
        Ok(Situated(
            Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            self.clone(),
        ))
    }
}
