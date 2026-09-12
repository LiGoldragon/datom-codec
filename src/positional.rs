//! Tuples are the positional types Rust already has. A struct declared in
//! Ethos states its positions through the derive; a multi-field variant's
//! payload has no name of its own, and its positions are a tuple's.

use crate::*;

impl<T: Datomizable> Datomizable for &T {
    fn datomize(&self, at: Path) -> Datom {
        (*self).datomize(at)
    }
}

macro_rules! positional_tuples {
    ($arity:literal, $($name:ident $index:tt),+) => {
        impl<$($name: Datomizable),+> Datomizable for ($($name,)+) {
            fn datomize(&self, at: Path) -> Datom {
                Datom {
                    path: at.clone(),
                    form: Form::Struct(vec![
                        $(self.$index.datomize(at.child($index as Integer)),)+
                    ]),
                }
            }
        }
        impl<$($name: Composing),+> Compositional for ($($name,)+) {
            const ARITY: Integer = $arity;
            fn from_positions(mut positions: Positions<'_>) -> Result<Self, Error> {
                Ok(($(positions.position::<$name>()?,)+))
            }
        }
        impl<$($name: Composing),+> Composing for ($($name,)+) {
            fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
                datom.compose_positions(budget)
            }
        }
    };
}

positional_tuples!(2, A 0, B 1);
positional_tuples!(3, A 0, B 1, C 2);
positional_tuples!(4, A 0, B 1, C 2, D 3);
positional_tuples!(5, A 0, B 1, C 2, D 3, E 4);
positional_tuples!(6, A 0, B 1, C 2, D 3, E 4, F 5);
positional_tuples!(7, A 0, B 1, C 2, D 3, E 4, F 5, G 6);
positional_tuples!(8, A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7);
positional_tuples!(9, A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8);
positional_tuples!(10, A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9);
positional_tuples!(11, A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10);
positional_tuples!(12, A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11);
