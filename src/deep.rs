//! Iterative ordinary operations for the recursive datom concept.

use std::fmt;

use crate::anatomy::Datom;

enum CopyJob<'a> {
    Datom(&'a Datom),
    Variant(protos::Symbol),
    Struct(usize),
    Vector(usize),
}
struct Copier<'a> {
    work: Vec<CopyJob<'a>>,
    values: Vec<Datom>,
}
trait Copying {
    fn copy(&mut self);
}
impl Copying for Copier<'_> {
    fn copy(&mut self) {
        while let Some(job) = self.work.pop() {
            match job {
                CopyJob::Datom(Datom::Variant(symbol, body)) => {
                    self.work.push(CopyJob::Variant(symbol.clone()));
                    self.work.push(CopyJob::Datom(body));
                }
                CopyJob::Datom(Datom::Struct(children)) => {
                    self.work.push(CopyJob::Struct(children.len()));
                    self.work.extend(children.iter().rev().map(CopyJob::Datom));
                }
                CopyJob::Datom(Datom::Vector(children)) => {
                    self.work.push(CopyJob::Vector(children.len()));
                    self.work.extend(children.iter().rev().map(CopyJob::Datom));
                }
                CopyJob::Datom(Datom::Text(text)) => self.values.push(Datom::Text(text.clone())),
                CopyJob::Datom(Datom::Meaning(meaning)) => {
                    self.values.push(Datom::Meaning(meaning.clone()))
                }
                CopyJob::Datom(Datom::Word(word)) => self.values.push(Datom::Word(word.clone())),
                CopyJob::Variant(symbol) => {
                    let body = self.values.pop().expect("variant body");
                    self.values.push(Datom::Variant(symbol, Box::new(body)));
                }
                CopyJob::Struct(count) => {
                    let children = self.values.split_off(self.values.len() - count);
                    self.values.push(Datom::Struct(children));
                }
                CopyJob::Vector(count) => {
                    let children = self.values.split_off(self.values.len() - count);
                    self.values.push(Datom::Vector(children));
                }
            }
        }
    }
}
impl Clone for Datom {
    fn clone(&self) -> Self {
        let mut copier = Copier {
            work: vec![CopyJob::Datom(self)],
            values: Vec::new(),
        };
        copier.copy();
        copier.values.pop().expect("one copied datom")
    }
}

struct ComparedDatoms<'a> {
    left: &'a Datom,
    right: &'a Datom,
}
struct Comparer<'a> {
    work: Vec<ComparedDatoms<'a>>,
}
trait Comparing {
    fn same(&mut self) -> bool;
}
impl Comparing for Comparer<'_> {
    fn same(&mut self) -> bool {
        while let Some(ComparedDatoms { left, right }) = self.work.pop() {
            match (left, right) {
                (
                    Datom::Variant(left_symbol, left_body),
                    Datom::Variant(right_symbol, right_body),
                ) => {
                    if left_symbol != right_symbol {
                        return false;
                    }
                    self.work.push(ComparedDatoms {
                        left: left_body,
                        right: right_body,
                    });
                }
                (Datom::Struct(left), Datom::Struct(right))
                | (Datom::Vector(left), Datom::Vector(right)) => {
                    if left.len() != right.len() {
                        return false;
                    }
                    self.work.extend(
                        left.iter()
                            .zip(right)
                            .map(|(left, right)| ComparedDatoms { left, right }),
                    );
                }
                (Datom::Text(left), Datom::Text(right)) => {
                    if left != right {
                        return false;
                    }
                }
                (Datom::Meaning(left), Datom::Meaning(right)) => {
                    if left != right {
                        return false;
                    }
                }
                (Datom::Word(left), Datom::Word(right)) => {
                    if left != right {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        true
    }
}
impl PartialEq for Datom {
    fn eq(&self, other: &Self) -> bool {
        let mut comparer = Comparer {
            work: vec![ComparedDatoms {
                left: self,
                right: other,
            }],
        };
        comparer.same()
    }
}
impl Eq for Datom {}

enum Show<'a> {
    Datom(&'a Datom),
    Text(&'static str),
    Symbol(&'a protos::Symbol),
    Word(&'a protos::Word),
    Plain(&'a protos::Text),
    Meaning(&'a protos::Opaque),
}
struct Displaying<'a> {
    work: Vec<Show<'a>>,
}
trait Showing<'a> {
    fn show(&mut self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    fn datoms(&mut self, values: &'a [Datom]);
}
impl<'a> Showing<'a> for Displaying<'a> {
    fn show(&mut self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        while let Some(job) = self.work.pop() {
            match job {
                Show::Text(text) => f.write_str(text)?,
                Show::Symbol(value) => write!(f, "{value:?}")?,
                Show::Word(value) => write!(f, "{value:?}")?,
                Show::Plain(value) => write!(f, "{value:?}")?,
                Show::Meaning(value) => write!(f, "{value:?}")?,
                Show::Datom(Datom::Variant(symbol, body)) => self.work.extend([
                    Show::Text(")"),
                    Show::Datom(body),
                    Show::Text(", "),
                    Show::Symbol(symbol),
                    Show::Text("Variant("),
                ]),
                Show::Datom(Datom::Struct(values)) => {
                    self.work.push(Show::Text(")"));
                    self.datoms(values);
                    self.work.push(Show::Text("Struct("));
                }
                Show::Datom(Datom::Vector(values)) => {
                    self.work.push(Show::Text(")"));
                    self.datoms(values);
                    self.work.push(Show::Text("Vector("));
                }
                Show::Datom(Datom::Text(value)) => {
                    self.work
                        .extend([Show::Text(")"), Show::Plain(value), Show::Text("Text(")])
                }
                Show::Datom(Datom::Meaning(value)) => self.work.extend([
                    Show::Text(")"),
                    Show::Meaning(value),
                    Show::Text("Meaning("),
                ]),
                Show::Datom(Datom::Word(value)) => {
                    self.work
                        .extend([Show::Text(")"), Show::Word(value), Show::Text("Word(")])
                }
            }
        }
        Ok(())
    }
    fn datoms(&mut self, values: &'a [Datom]) {
        self.work.push(Show::Text("]"));
        for (index, value) in values.iter().enumerate().rev() {
            self.work.push(Show::Datom(value));
            if index != 0 {
                self.work.push(Show::Text(", "));
            }
        }
        self.work.push(Show::Text("["));
    }
}
impl fmt::Debug for Datom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut displaying = Displaying {
            work: vec![Show::Datom(self)],
        };
        displaying.show(f)
    }
}
