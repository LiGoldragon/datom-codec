//! Iterative drop for the datom tree: a deep datom never recurses on its way out.

use protos::Word;

use crate::anatomy::Datom;

/// The kind whose capability moves a node's children out onto a worklist, leaving the node a leaf.
trait Shedding {
    fn shed(&mut self, work: &mut Vec<Datom>);
}

impl Shedding for Datom {
    fn shed(&mut self, work: &mut Vec<Datom>) {
        match self {
            Datom::Variant(_, body) => {
                work.push(std::mem::replace(
                    body.as_mut(),
                    Datom::Word(Word::try_from("_").unwrap()),
                ));
            }
            Datom::Struct(children) | Datom::Vector(children) => work.append(children),
            Datom::Text(_) | Datom::Meaning(_) | Datom::Word(_) => {}
        }
    }
}

impl Drop for Datom {
    fn drop(&mut self) {
        let mut work = Vec::new();
        self.shed(&mut work);
        while let Some(mut datom) = work.pop() {
            datom.shed(&mut work);
        }
    }
}
