use std::mem;

use crate::{Datom, Form};

trait TreeDropping {
    fn empty_tree(&mut self) -> Form;
}
impl TreeDropping for Datom {
    fn empty_tree(&mut self) -> Form {
        mem::replace(&mut self.form, Form::Bare(String::new()))
    }
}

impl Drop for Datom {
    fn drop(&mut self) {
        let mut forms = vec![self.empty_tree()];
        while let Some(form) = forms.pop() {
            match form {
                Form::Variant(_, body) => {
                    let mut body = *body;
                    forms.push(body.empty_tree());
                }
                Form::Struct(children) | Form::Vector(children) => {
                    for mut child in children {
                        forms.push(child.empty_tree());
                    }
                }
                Form::Bare(_) | Form::String(_) | Form::Meaning(_) => {}
            }
        }
    }
}
