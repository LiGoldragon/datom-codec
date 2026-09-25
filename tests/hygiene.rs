//! The derives name every standard item they emit by its full path, so a
//! crate that declares its own `Box`, `Result`, `Ok`, `Err` or `vec` still
//! derives and round-trips.

#![allow(dead_code)]

use datom_codec::{Actualizing, Budget, Composing, Datomizable, Path, Potential};
use protos::{Protosizable, ReaderBudget, Textualizable};

pub struct Box {
    pub integer: i64,
}
pub enum Result {
    Fine,
}
pub struct Ok {
    pub integer: i64,
}
pub struct Err {
    pub integer: i64,
}
#[allow(unused_macros)]
macro_rules! vec {
    ($($anything:tt)*) => {
        compile_error!("the derive reached a local vec!")
    };
}

#[derive(Debug, PartialEq, Composing, Datomizable)]
enum Shadowed {
    Unit,
    Carrying(i64),
    Pair(i64, String),
}

#[derive(Debug, PartialEq, Composing, Datomizable)]
struct Holder {
    shadowed: Shadowed,
    integer: i64,
}

fn budget() -> Budget {
    Budget {
        remaining: 4_096,
        reader: ReaderBudget { remaining: 4_096 },
        depth: 0,
        maximum_depth: 4_096,
    }
}

#[test]
fn derives_round_trip_beside_local_standard_names() {
    for value in [
        Holder {
            shadowed: Shadowed::Unit,
            integer: 1,
        },
        Holder {
            shadowed: Shadowed::Carrying(2),
            integer: 3,
        },
        Holder {
            shadowed: Shadowed::Pair(4, "five".to_owned()),
            integer: 6,
        },
    ] {
        let text = value.datomize(Path::new()).protosize().textualize();
        let back: Holder = Potential::<Holder>::from(text.clone())
            .actualize(&mut budget())
            .unwrap_or_else(|error| panic!("{text}: {error:?}"));
        assert_eq!(back, value);
    }
}
