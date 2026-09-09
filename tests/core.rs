use datom_codec::{
    Actualizing, Budget, Composable, Compositional, Datom, Datomizable, ErrorKind, Form, Meaning,
    Path, Potential, PotentialExtenting, Scalar,
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
enum Pair {
    Values(i64, i64),
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
struct Wrapper<T> {
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
    assert_eq!(pending.protosize().textualize(), "Pending");
    assert_eq!(
        Potential::<Reply>::from("Pending")
            .actualize(&mut Budget {
                remaining: 1,
                reader: ReaderBudget { remaining: 128 }
            })
            .unwrap(),
        Reply::Pending
    );
}

#[test]
fn scalar_writers_preserve_their_textual_kind() {
    assert_eq!(3.0_f64.datomize(vec![]).protosize().textualize(), "3.0");
    assert_eq!((-0.0_f64).datomize(vec![]).protosize().textualize(), "-0.0");
    assert_eq!(
        "a{b".to_owned().datomize(vec![]).protosize().textualize(),
        "«a{b»"
    );
    assert!(
        i64::scalar(
            &Datom {
                path: vec![],
                form: Form::Bare("-01".into())
            },
            &mut Budget {
                remaining: 1,
                reader: ReaderBudget { remaining: 1 }
            }
        )
        .is_err()
    );
}

#[test]
fn qualified_heads_refuse_at_the_variant_path() {
    let mut potential = Potential::<Reply>::from("Accepted<String>.42");
    let error = potential
        .actualize(&mut Budget {
            remaining: 10,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap_err();
    assert_eq!(error.path, Vec::<i64>::new());
    assert!(matches!(
        error.kind,
        ErrorKind::Form {
            expected: "unqualified Variant",
            found: "qualified head"
        }
    ));
    assert_eq!(
        potential.reader_extent(&error.path),
        Some(protos::Extent { start: 0, end: 19 })
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
    let text = datom.protosize().textualize();
    assert_eq!(text, "{ Ada 42 True }");
    assert_eq!(datom.protosize(), text.protosize().unwrap());
    let rebuilt = text.protosize().unwrap().datomize(vec![]);
    assert_eq!(rebuilt.unwrap(), datom);
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
            .textualize(),
        "Some.Ada:one"
    );
    assert_eq!(
        Some(three_point_fourteen)
            .datomize(vec![])
            .protosize()
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
        datom.protosize().textualize(),
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

#[test]
fn unit_options_and_unit_enums_round_trip_as_bare_text() {
    let value: Option<i64> = None;
    assert_eq!(value.datomize(vec![]).protosize().textualize(), "None");
    assert_eq!(
        Potential::<Option<i64>>::from("None")
            .actualize(&mut Budget {
                remaining: 1,
                reader: ReaderBudget { remaining: 128 }
            })
            .unwrap(),
        None
    );
    assert_eq!(
        Potential::<Reply>::from("Pending")
            .actualize(&mut Budget {
                remaining: 1,
                reader: ReaderBudget { remaining: 128 }
            })
            .unwrap(),
        Reply::Pending
    );
}

#[test]
fn datom_refuses_non_datom_structural_forms() {
    let angled = Potential::<Vec<i64>>::from("< 1 2 >")
        .actualize(&mut Budget {
            remaining: 10,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap_err();
    assert!(matches!(
        angled.kind,
        ErrorKind::Form {
            expected: "Datom enclosure",
            found: "Angled"
        }
    ));

    let positions = Potential::<Pair>::from("Values.[ 1 2 ]")
        .actualize(&mut Budget {
            remaining: 10,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap_err();
    assert!(matches!(
        positions.kind,
        ErrorKind::Form {
            expected: "Struct",
            found: "Vector"
        }
    ));
}

#[test]
fn strings_quote_syntax_and_accept_temporary_meaning_text() {
    for value in ["a{b", "a;b", "a.b", "a(b", "a!b"] {
        let text = value.to_owned().datomize(vec![]).protosize().textualize();
        assert!(text.starts_with('«'), "{value}: {text}");
        assert_eq!(
            Potential::<String>::from(text)
                .actualize(&mut Budget {
                    remaining: 1,
                    reader: ReaderBudget { remaining: 128 }
                })
                .unwrap(),
            value
        );
    }
    assert_eq!(
        Potential::<String>::from("(temporary meaning)")
            .actualize(&mut Budget {
                remaining: 1,
                reader: ReaderBudget { remaining: 128 }
            })
            .unwrap(),
        "temporary meaning"
    );
}

#[test]
fn retained_reader_finds_nested_composition_fault_extents() {
    let mut vector = Potential::<Vec<i64>>::from("[ 1 x ]");
    let error = vector
        .actualize(&mut Budget {
            remaining: 10,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap_err();
    assert_eq!(error.path, vec![1]);
    assert_eq!(
        vector.reader_extent(&error.path),
        Some(protos::Extent { start: 4, end: 5 })
    );

    let mut variant = Potential::<Pair>::from("Values.{ 1 x }");
    let error = variant
        .actualize(&mut Budget {
            remaining: 10,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap_err();
    assert_eq!(error.path, vec![1, 1]);
    assert_eq!(
        variant.reader_extent(&error.path),
        Some(protos::Extent { start: 11, end: 12 })
    );
}

#[test]
fn typed_protos_errors_round_trip_without_a_reader_tree() {
    let error = protos::Error {
        extent: protos::Extent { start: 3, end: 4 },
        problem: protos::Problem::Unexpected('@'),
    };
    let datom = error.datomize(vec![]);
    let text = datom.protosize().textualize();
    let rebuilt: protos::Error = Potential::from(text)
        .actualize(&mut Budget {
            remaining: 20,
            reader: ReaderBudget { remaining: 128 },
        })
        .unwrap();
    assert_eq!(rebuilt, error);
}
