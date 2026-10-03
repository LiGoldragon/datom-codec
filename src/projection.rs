use crate::{Datom, Form};

pub(crate) trait Projecting {
    fn project(&self) -> protos::Protos;
}

/// A projected node has no text of its own yet; protos assigns the extents of
/// the canonical print once the tree is whole.
const UNPLACED: protos::Extent = protos::Extent { start: 0, end: 0 };

enum Build<'a> {
    Visit(&'a Datom),
    Enclosed(protos::Enclosure, usize),
    Headed(protos::Symbol),
}

trait Building<'a> {
    fn build(root: &'a Datom) -> protos::Protos;
}
struct Builder<'a> {
    work: Vec<Build<'a>>,
    values: Vec<protos::Protos>,
}
impl<'a> Building<'a> for Builder<'a> {
    fn build(root: &'a Datom) -> protos::Protos {
        let mut builder = Self {
            work: vec![Build::Visit(root)],
            values: Vec::new(),
        };
        while let Some(job) = builder.work.pop() {
            match job {
                Build::Visit(datom) => match &datom.form {
                    Form::Bare(text) => builder.values.push(protos::Protos::Bare {
                        extent: UNPLACED,
                        text: text.clone(),
                    }),
                    Form::String(content) => builder.values.push(protos::Protos::Opaque {
                        extent: UNPLACED,
                        boundary: protos::Boundary::Guillemets,
                        content: content.clone(),
                    }),
                    Form::Meaning(content) => builder.values.push(protos::Protos::Opaque {
                        extent: UNPLACED,
                        boundary: protos::Boundary::Parentheses,
                        content: content.clone(),
                    }),
                    Form::Variant(head, body) => {
                        builder.work.push(Build::Headed(head.clone()));
                        builder.work.push(Build::Visit(body));
                    }
                    Form::Struct(children) | Form::Vector(children) => {
                        let enclosure = if matches!(&datom.form, Form::Struct(_)) {
                            protos::Enclosure::Braced
                        } else {
                            protos::Enclosure::Bracketed
                        };
                        builder
                            .work
                            .push(Build::Enclosed(enclosure, children.len()));
                        builder.work.extend(children.iter().rev().map(Build::Visit));
                    }
                },
                Build::Enclosed(enclosure, count) => {
                    let children = builder.values.split_off(builder.values.len() - count);
                    builder.values.push(protos::Protos::Enclosed {
                        extent: UNPLACED,
                        enclosure,
                        children,
                    });
                }
                Build::Headed(head) => {
                    let body = Box::new(builder.values.pop().expect("projected headed body"));
                    builder.values.push(protos::Protos::Headed {
                        extent: UNPLACED,
                        head,
                        constraints: None,
                        separator: protos::Separator::Period,
                        body,
                    });
                }
            }
        }
        builder.values.pop().expect("projected root")
    }
}

impl Projecting for Datom {
    fn project(&self) -> protos::Protos {
        use protos::Canonicalizable;
        let mut projected = Builder::build(self);
        projected.canonicalize();
        projected
    }
}
impl protos::Protosizable for Datom {
    type Output = protos::Protos;
    fn protosize(&self) -> Self::Output {
        self.project()
    }
}
