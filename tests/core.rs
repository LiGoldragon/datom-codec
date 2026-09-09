use datom_codec::{
    Actualizing, Budget, Composable, Compositional, Datom, Datomizable, ErrorKind, Form, Meaning,
    Path, Potential, Scalar,
};
use protos::{Protosizable, ReaderBudget, Textualizable};

#[derive(Debug, PartialEq, Compositional, Datomizable)]
struct Score {
    name: String,
    value: i64,
    enabled: bool,
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
enum Reply {
    Accepted(i64, String),
    Pending,
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
struct Wrapper<T: Compositional + Datomizable> {
    value: T,
}

#[test]
fn derived_struct_composes_its_typed_positions() {
    let datom = Datom {
        path: Path::new(),
        form: Form::Struct(vec![
            Datom {
                path: vec![0],
                form: Form::Bare("Ada".into()),
            },
            Datom {
                path: vec![1],
                form: Form::Bare("42".into()),
            },
            Datom {
                path: vec![2],
                form: Form::Bare("True".into()),
            },
        ]),
    };
    let value: Score = datom
        .compose(&mut Budget {
            remaining: 10,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap();
    assert_eq!(
        value,
        Score {
            name: "Ada".into(),
            value: 42,
            enabled: true
        }
    );
    assert_eq!(value.datomize(vec![]), datom);
}

#[test]
fn scalar_refusal_keeps_the_datom_path() {
    let datom = Datom {
        path: vec![2],
        form: Form::Bare("01".into()),
    };
    let error = i64::scalar(
        &datom,
        &mut Budget {
            remaining: 1,
            reader: ReaderBudget { remaining: 128 },
        },
    )
    .unwrap_err();
    assert_eq!(error.path, vec![2]);
}

#[test]
fn derived_variants_use_their_rust_names_as_heads() {
    let value = Reply::Accepted(42, "today".into());
    let datom = value.datomize(vec![]);
    assert!(matches!(&datom.form, Form::Variant(head, _) if head.0 == "Accepted"));
    assert_eq!(
        datom
            .compose::<Reply>(&mut Budget {
                remaining: 10,
                reader: ReaderBudget { remaining: 10 }
            })
            .unwrap(),
        value
    );
    let pending = Reply::Pending.datomize(vec![]);
    assert_eq!(
        pending
            .compose::<Reply>(&mut Budget {
                remaining: 10,
                reader: ReaderBudget { remaining: 10 }
            })
            .unwrap(),
        Reply::Pending
    );
}

#[test]
fn protos_conversion_preserves_datoms_and_canonical_text() {
    let datom = Score {
        name: "Ada".into(),
        value: 42,
        enabled: true,
    }
    .datomize(vec![]);
    let text = datom.protosize().unwrap().textualize();
    assert_eq!(text, "{ Ada 42 True }");
    let rebuilt = text.protosize().unwrap().datomize(vec![]);
    assert_eq!(rebuilt, datom);
}

#[test]
fn potential_actualizes_text_through_protos_and_datom() {
    let value: Score = Potential::from("{ Ada 42 True }")
        .actualize(&mut Budget {
            remaining: 10,
            reader: ReaderBudget { remaining: 10 },
        })
        .unwrap();
    assert_eq!(
        value,
        Score {
            name: "Ada".into(),
            value: 42,
            enabled: true
        }
    );
}

#[test]
fn scalar_positions_keep_bare_payloads_and_some_bodies_flat() {
    let three_point_fourteen: f64 = "3.14".parse().unwrap();
    let timestamp: String = Potential::from("2026-09-03T17:46:20")
        .actualize(&mut Budget {
            remaining: 1,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap();
    assert_eq!(timestamp, "2026-09-03T17:46:20");
    let name: String = Potential::from("Ada:one")
        .actualize(&mut Budget {
            remaining: 1,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap();
    assert_eq!(name, "Ada:one");

    let string: Option<String> = Potential::from("Some.42")
        .actualize(&mut Budget {
            remaining: 2,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap();
    assert_eq!(string, Some("42".into()));
    let integer: Option<i64> = Potential::from("Some.42")
        .actualize(&mut Budget {
            remaining: 2,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap();
    assert_eq!(integer, Some(42));
    let decimal: f64 = Potential::from("3.14")
        .actualize(&mut Budget {
            remaining: 1,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap();
    assert_eq!(decimal, three_point_fourteen);
    let optional_decimal: Option<f64> = Potential::from("Some.3.14")
        .actualize(&mut Budget {
            remaining: 2,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap();
    assert_eq!(optional_decimal, Some(three_point_fourteen));

    assert_eq!(
        Some("Ada:one".to_owned())
            .datomize(vec![])
            .protosize()
            .unwrap()
            .textualize(),
        "Some.Ada:one"
    );
    assert_eq!(
        Some(three_point_fourteen)
            .datomize(vec![])
            .protosize()
            .unwrap()
            .textualize(),
        "Some.3.14"
    );
}

#[test]
fn generic_containers_and_meaning_round_trip_their_forms() {
    let value = Wrapper {
        value: Some(Box::new(Meaning("nested (meaning)".into()))),
    };
    let datom = value.datomize(vec![]);
    assert_eq!(
        datom.protosize().unwrap().textualize(),
        "{ Some.(nested (meaning)) }"
    );
    let rebuilt: Wrapper<Option<Box<Meaning>>> = datom
        .compose(&mut Budget {
            remaining: 10,
            reader: ReaderBudget { remaining: 10 },
        })
        .unwrap();
    assert_eq!(rebuilt, value);
    let result: Result<i64, String> = Err("nope".into());
    let datom = result.datomize(vec![]);
    assert_eq!(
        datom
            .compose::<Result<i64, String>>(&mut Budget {
                remaining: 10,
                reader: ReaderBudget { remaining: 10 }
            })
            .unwrap(),
        result
    );
}

#[test]
fn potential_reports_structural_and_budget_errors_at_their_context() {
    let structural = Potential::<Score>::from("{ Ada")
        .actualize(&mut Budget {
            remaining: 10,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap_err();
    assert_eq!(structural.path, Vec::<i64>::new());
    assert!(matches!(structural.kind, ErrorKind::Structural(_)));
    let exhausted = Potential::<Score>::from("{ Ada 42 True }")
        .actualize(&mut Budget {
            remaining: 0,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap_err();
    assert_eq!(exhausted.path, Vec::<i64>::new());
    assert_eq!(exhausted.kind, ErrorKind::Budget);
}

#[test]
fn potential_refuses_before_composition_when_reader_budget_is_exhausted() {
    let error = Potential::<String>::from("Ada")
        .actualize(&mut Budget {
            remaining: 1,
            reader: ReaderBudget { remaining: 0 },
        })
        .unwrap_err();
    assert_eq!(error.path, Vec::<i64>::new());
    assert!(matches!(error.kind, ErrorKind::Structural(_)));
}
