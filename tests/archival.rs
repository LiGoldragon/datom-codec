//! Under the `rkyv` feature, datom-codec's error, the type an ethos contract
//! holds where a datom reading fails, archives and restores equal, its
//! structural kind carrying a protos error included, so a generated
//! contract holding it can cross a wire.

use datom_codec::{Error, ErrorKind, ErrorLayer};

trait Crossing: Sized {
    fn crossed(&self) -> Self;
}

impl Crossing for Error {
    fn crossed(&self) -> Self {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self).expect("archives");
        rkyv::from_bytes::<Error, rkyv::rancor::Error>(&bytes).expect("restores")
    }
}

#[test]
fn a_datom_error_crosses_the_wire() {
    let errors = [
        Error {
            layer: ErrorLayer::Protos,
            path: vec![0, 2],
            kind: ErrorKind::Structural(protos::Error {
                extent: protos::Extent { start: 1, end: 4 },
                problem: protos::Problem::Unclosed('['),
            }),
        },
        Error {
            layer: ErrorLayer::Composition,
            path: vec![],
            kind: ErrorKind::Arity {
                expected: 2,
                found: 3,
            },
        },
        Error {
            layer: ErrorLayer::Datom,
            path: vec![1],
            kind: ErrorKind::Form {
                expected: "Struct".to_owned(),
                found: "Bare".to_owned(),
            },
        },
    ];
    for error in errors {
        assert_eq!(error.crossed(), error);
    }
}
