use std::collections::HashMap;

use crate::{Datom, Form};

pub(crate) trait Projecting {
    fn project(&self) -> protos::Protos;
}

enum Measure<'a> {
    Visit(&'a Datom),
    Finish(&'a Datom),
}
enum Build<'a> {
    Visit(&'a Datom, usize),
    Enclosed(protos::Enclosure, usize, usize, usize),
    Headed(protos::Symbol, usize, usize),
}

trait Measuring<'a> {
    fn measure(root: &'a Datom) -> HashMap<usize, usize>;
}
struct Meter<'a> {
    work: Vec<Measure<'a>>,
    lengths: HashMap<usize, usize>,
}
impl<'a> Measuring<'a> for Meter<'a> {
    fn measure(root: &'a Datom) -> HashMap<usize, usize> {
        let mut meter = Self {
            work: vec![Measure::Visit(root)],
            lengths: HashMap::new(),
        };
        while let Some(job) = meter.work.pop() {
            match job {
                Measure::Visit(datom) => match &datom.form {
                    Form::Bare(text) => {
                        meter.lengths.insert(datom as *const _ as usize, text.len());
                    }
                    Form::String(content) => {
                        meter.lengths.insert(
                            datom as *const _ as usize,
                            protos::Boundary::Guillemets.opaque_length(content),
                        );
                    }
                    Form::Meaning(content) => {
                        meter.lengths.insert(
                            datom as *const _ as usize,
                            protos::Boundary::Parentheses.opaque_length(content),
                        );
                    }
                    Form::Variant(_, body) => {
                        meter.work.push(Measure::Finish(datom));
                        meter.work.push(Measure::Visit(body));
                    }
                    Form::Struct(children) | Form::Vector(children) => {
                        meter.work.push(Measure::Finish(datom));
                        meter.work.extend(children.iter().map(Measure::Visit));
                    }
                },
                Measure::Finish(datom) => {
                    let length = match &datom.form {
                        Form::Variant(head, body) => {
                            head.0.len() + 1 + meter.lengths[&(body.as_ref() as *const _ as usize)]
                        }
                        Form::Struct(children) | Form::Vector(children) => {
                            if children.is_empty() {
                                2
                            } else {
                                children
                                    .iter()
                                    .map(|child| meter.lengths[&(child as *const _ as usize)])
                                    .sum::<usize>()
                                    + children.len()
                                    + 3
                            }
                        }
                        _ => unreachable!(),
                    };
                    meter.lengths.insert(datom as *const _ as usize, length);
                }
            }
        }
        meter.lengths
    }
}

trait Building<'a> {
    fn build(root: &'a Datom, lengths: &HashMap<usize, usize>) -> protos::Protos;
}
struct Builder<'a> {
    work: Vec<Build<'a>>,
    values: Vec<protos::Protos>,
}
impl<'a> Building<'a> for Builder<'a> {
    fn build(root: &'a Datom, lengths: &HashMap<usize, usize>) -> protos::Protos {
        let mut builder = Self {
            work: vec![Build::Visit(root, 0)],
            values: Vec::new(),
        };
        while let Some(job) = builder.work.pop() {
            match job {
                Build::Visit(datom, start) => match &datom.form {
                    Form::Bare(text) => builder.values.push(protos::Protos::Bare {
                        extent: protos::Extent {
                            start,
                            end: start + text.len(),
                        },
                        text: text.clone(),
                    }),
                    Form::String(content) => builder
                        .values
                        .push(protos::Boundary::Guillemets.project_opaque(content, start)),
                    Form::Meaning(content) => builder
                        .values
                        .push(protos::Boundary::Parentheses.project_opaque(content, start)),
                    Form::Variant(head, body) => {
                        let end = start + lengths[&(datom as *const _ as usize)];
                        builder.work.push(Build::Headed(head.clone(), start, end));
                        builder
                            .work
                            .push(Build::Visit(body, start + head.0.len() + 1));
                    }
                    Form::Struct(children) | Form::Vector(children) => {
                        let enclosure = if matches!(&datom.form, Form::Struct(_)) {
                            protos::Enclosure::Braced
                        } else {
                            protos::Enclosure::Bracketed
                        };
                        let end = start + lengths[&(datom as *const _ as usize)];
                        builder
                            .work
                            .push(Build::Enclosed(enclosure, start, end, children.len()));
                        let mut offset = start + 2;
                        let mut starts = Vec::with_capacity(children.len());
                        for child in children {
                            starts.push(offset);
                            offset += lengths[&(child as *const _ as usize)] + 1;
                        }
                        for (child, child_start) in children.iter().zip(starts).rev() {
                            builder.work.push(Build::Visit(child, child_start));
                        }
                    }
                },
                Build::Enclosed(enclosure, start, end, count) => {
                    let children = builder.values.split_off(builder.values.len() - count);
                    builder.values.push(protos::Protos::Enclosed {
                        extent: protos::Extent { start, end },
                        enclosure,
                        children,
                    });
                }
                Build::Headed(head, start, end) => {
                    let body = Box::new(builder.values.pop().expect("projected headed body"));
                    builder.values.push(protos::Protos::Headed {
                        extent: protos::Extent { start, end },
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

trait OpaqueProjecting {
    fn project_opaque(self, content: &str, start: usize) -> protos::Protos;
    fn opaque_length(self, content: &str) -> usize;
}
impl OpaqueProjecting for protos::Boundary {
    fn project_opaque(self, content: &str, start: usize) -> protos::Protos {
        let end = start + self.opaque_length(content);
        protos::Protos::Opaque {
            extent: protos::Extent { start, end },
            boundary: self,
            content: content.to_owned(),
        }
    }
    fn opaque_length(self, content: &str) -> usize {
        use protos::Textualizable;
        protos::Protos::Opaque {
            extent: protos::Extent { start: 0, end: 0 },
            boundary: self,
            content: content.to_owned(),
        }
        .textualize()
        .len()
    }
}
impl Projecting for Datom {
    fn project(&self) -> protos::Protos {
        let lengths = Meter::measure(self);
        Builder::build(self, &lengths)
    }
}
impl protos::Protosizable for Datom {
    type Output = protos::Protos;
    fn protosize(&self) -> Self::Output {
        self.project()
    }
}
