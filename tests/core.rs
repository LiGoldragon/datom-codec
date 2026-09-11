use datom_codec::{
    Actualizing, Budget, Composable, Compositional, Datom, Datomizable, Error, ErrorKind,
    ErrorLayer, ErrorRaising, Form, Meaning, Path, Potential, PotentialExtenting, ProtosExtenting,
    Scalar,
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
enum Message {
    Reply(Reply),
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
struct PunctuatedStrings {
    model: String,
    url: String,
    qualified_name: String,
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
            depth: 0,
            maximum_depth: 4_096,
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
fn protos_symbol_and_reader_budget_are_datom_intrinsics() {
    let mut budget = Budget {
        remaining: 20,
        reader: ReaderBudget { remaining: 128 },
        depth: 0,
        maximum_depth: 128,
    };
    let symbol = protos::Symbol("Reviewer".into());
    let datom = symbol.datomize(vec![]);
    let rebuilt: protos::Symbol = datom.compose(&mut budget.clone()).unwrap();
    assert_eq!(rebuilt, symbol);

    let reader = ReaderBudget { remaining: 4_096 };
    let datom = reader.datomize(vec![]);
    let rebuilt: ReaderBudget = datom.compose(&mut budget).unwrap();
    assert_eq!(rebuilt, reader);
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
            depth: 0,
            maximum_depth: 4_096,
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
                reader: ReaderBudget { remaining: 10 },
                depth: 0,
                maximum_depth: 4_096
            })
            .unwrap(),
        value
    );
    let pending = Reply::Pending.datomize(vec![]);
    assert_eq!(
        pending
            .compose::<Reply>(&mut Budget {
                remaining: 10,
                reader: ReaderBudget { remaining: 10 },
                depth: 0,
                maximum_depth: 4_096
            })
            .unwrap(),
        Reply::Pending
    );
    assert_eq!(pending.protosize().textualize(), "Pending");
    assert_eq!(
        Potential::<Reply>::from("Pending")
            .actualize(&mut Budget {
                remaining: 1,
                reader: ReaderBudget { remaining: 128 },
                depth: 0,
                maximum_depth: 4_096
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
                reader: ReaderBudget { remaining: 1 },
                depth: 0,
                maximum_depth: 4_096
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
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap_err();
    assert_eq!(error.path, Vec::<i64>::new());
    assert!(matches!(
        error.kind,
        ErrorKind::Form { ref expected, ref found }
            if expected == "unqualified Variant" && found == "qualified head"
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
            depth: 0,
            maximum_depth: 4_096,
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
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap();
    assert_eq!(timestamp, "2026-09-03T17:46:20");
    let name: String = Potential::from("Ada:one")
        .actualize(&mut Budget {
            remaining: 1,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap();
    assert_eq!(name, "Ada:one");

    let string: Option<String> = Potential::from("Some.42")
        .actualize(&mut Budget {
            remaining: 2,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap();
    assert_eq!(string, Some("42".into()));
    let integer: Option<i64> = Potential::from("Some.42")
        .actualize(&mut Budget {
            remaining: 2,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap();
    assert_eq!(integer, Some(42));
    let decimal: f64 = Potential::from("3.14")
        .actualize(&mut Budget {
            remaining: 1,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap();
    assert_eq!(decimal, three_point_fourteen);
    let optional_decimal: Option<f64> = Potential::from("Some.3.14")
        .actualize(&mut Budget {
            remaining: 2,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 4_096,
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
fn option_string_with_protos_separators_round_trips() {
    let value = Some("5::7/128".to_owned());
    let rendered = value.datomize(vec![]).protosize().textualize();
    assert_eq!(rendered, "Some.«5::7/128»");
    let decoded: Option<String> = Potential::from(rendered.as_str())
        .actualize(&mut Budget {
            remaining: 2,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 4_096,
        })
        .expect("quoted separator string decodes");
    assert_eq!(decoded, value);
}

#[test]
fn typed_string_positions_resolve_bare_syntax_characters() {
    let text = "{ gpt-5.6-luna https://example.org/a Upper.Case }";
    let expected = PunctuatedStrings {
        model: "gpt-5.6-luna".into(),
        url: "https://example.org/a".into(),
        qualified_name: "Upper.Case".into(),
    };
    let value: PunctuatedStrings = Potential::from(text)
        .actualize(&mut Budget {
            remaining: 16,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 16,
        })
        .unwrap();
    assert_eq!(value, expected);
    assert_eq!(value.datomize(vec![]).protosize().textualize(), text);
}

#[test]
fn string_rule_round_trips_bare_punctuation_and_delimited_content() {
    for (value, canonical) in [
        (".a", ".a"),
        ("a.", "a."),
        ("a..b", "a..b"),
        (".codex/agents/worker.toml", ".codex/agents/worker.toml"),
        ("lower.case", "lower.case"),
        ("Upper.Case", "Upper.Case"),
        ("a:b:c", "a:b:c"),
        ("a!b!c", "a!b!c"),
        ("https://example.org/a", "https://example.org/a"),
        ("猫.龍", "猫.龍"),
        ("path\\segment", "path\\segment"),
        ("", "«»"),
        ("two words", "«two words»"),
        (";comment", "«;comment»"),
        ("a;b", "«a;b»"),
        ("{x}", "«{x}»"),
        ("(x)", "«(x)»"),
        ("«x»", "««x\\»»"),
    ] {
        let text = value.to_owned().datomize(vec![]).protosize().textualize();
        assert_eq!(text, canonical, "canonical String for {value:?}");
        let rebuilt: String = Potential::from(text)
            .actualize(&mut Budget {
                remaining: 64,
                reader: ReaderBudget { remaining: 256 },
                depth: 0,
                maximum_depth: 64,
            })
            .unwrap();
        assert_eq!(rebuilt, value, "String round trip for {canonical:?}");
    }
}

#[test]
fn typed_enums_keep_bare_carried_and_daisy_variants() {
    for (text, expected) in [
        (
            "Reply.Accepted.{ 42 today }",
            Message::Reply(Reply::Accepted(42, "today".into())),
        ),
        ("Reply.Pending", Message::Reply(Reply::Pending)),
    ] {
        let value: Message = Potential::from(text)
            .actualize(&mut Budget {
                remaining: 16,
                reader: ReaderBudget { remaining: 128 },
                depth: 0,
                maximum_depth: 16,
            })
            .unwrap();
        assert_eq!(value, expected);
        assert_eq!(value.datomize(vec![]).protosize().textualize(), text);
    }
}

#[test]
fn delimited_variant_shapes_do_not_compose_as_strings() {
    let error = Potential::<String>::from("Head.{ x }")
        .actualize(&mut Budget {
            remaining: 8,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 8,
        })
        .unwrap_err();
    assert!(matches!(
        error.kind,
        ErrorKind::Form { ref expected, .. } if expected == "String"
    ));
}

#[test]
fn dotted_string_composition_obeys_depth_and_work_budgets() {
    let dotted = Datom {
        path: vec![],
        form: Form::Variant(
            protos::Symbol("One".into()),
            Box::new(Datom {
                path: vec![1],
                form: Form::Variant(
                    protos::Symbol("Two".into()),
                    Box::new(Datom {
                        path: vec![1, 1],
                        form: Form::Variant(
                            protos::Symbol("Three".into()),
                            Box::new(Datom {
                                path: vec![1, 1, 1],
                                form: Form::Bare("Four".into()),
                            }),
                        ),
                    }),
                ),
            }),
        ),
    };
    let mut depth_budget = Budget {
        remaining: 16,
        reader: ReaderBudget { remaining: 16 },
        depth: 0,
        maximum_depth: 3,
    };
    let error = dotted.compose::<String>(&mut depth_budget).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Budget);
    assert_eq!(depth_budget.depth, 0);

    let mut work_budget = Budget {
        remaining: 2,
        reader: ReaderBudget { remaining: 16 },
        depth: 0,
        maximum_depth: 16,
    };
    let error = dotted.compose::<String>(&mut work_budget).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Budget);
    assert_eq!(work_budget.depth, 0);
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
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap();
    assert_eq!(rebuilt, value);
    let result: Result<i64, String> = Err("nope".into());
    let datom = result.datomize(vec![]);
    assert_eq!(
        datom
            .compose::<Result<i64, String>>(&mut Budget {
                remaining: 10,
                reader: ReaderBudget { remaining: 10 },
                depth: 0,
                maximum_depth: 4_096
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
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap_err();
    assert_eq!(structural.path, Vec::<i64>::new());
    assert!(matches!(structural.kind, ErrorKind::Structural(_)));
    let exhausted = Potential::<Score>::from("{ Ada 42 True }")
        .actualize(&mut Budget {
            remaining: 0,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 4_096,
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
            depth: 0,
            maximum_depth: 4_096,
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
                reader: ReaderBudget { remaining: 128 },
                depth: 0,
                maximum_depth: 4_096
            })
            .unwrap(),
        None
    );
    assert_eq!(
        Potential::<Reply>::from("Pending")
            .actualize(&mut Budget {
                remaining: 1,
                reader: ReaderBudget { remaining: 128 },
                depth: 0,
                maximum_depth: 4_096
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
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap_err();
    assert!(matches!(
        angled.kind,
        ErrorKind::Form { ref expected, ref found }
            if expected == "Datom enclosure" && found == "Angled"
    ));

    let positions = Potential::<Pair>::from("Values.[ 1 2 ]")
        .actualize(&mut Budget {
            remaining: 10,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap_err();
    assert!(matches!(
        positions.kind,
        ErrorKind::Form { ref expected, ref found }
            if expected == "Struct" && found == "Vector"
    ));
}

#[test]
fn strings_quote_delimiters_keep_bare_separators_and_accept_temporary_meaning_text() {
    for value in ["a{b", "a;b", "a(b"] {
        let text = value.to_owned().datomize(vec![]).protosize().textualize();
        assert!(text.starts_with('«'), "{value}: {text}");
        assert_eq!(
            Potential::<String>::from(text)
                .actualize(&mut Budget {
                    remaining: 1,
                    reader: ReaderBudget { remaining: 128 },
                    depth: 0,
                    maximum_depth: 4_096
                })
                .unwrap(),
            value
        );
    }
    for value in ["a.b", "a!b", "a:b"] {
        let text = value.to_owned().datomize(vec![]).protosize().textualize();
        assert_eq!(text, value);
        assert_eq!(
            Potential::<String>::from(text)
                .actualize(&mut Budget {
                    remaining: 8,
                    reader: ReaderBudget { remaining: 128 },
                    depth: 0,
                    maximum_depth: 8,
                })
                .unwrap(),
            value
        );
    }
    assert_eq!(
        Potential::<String>::from("(temporary meaning)")
            .actualize(&mut Budget {
                remaining: 1,
                reader: ReaderBudget { remaining: 128 },
                depth: 0,
                maximum_depth: 4_096
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
            depth: 0,
            maximum_depth: 4_096,
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
            depth: 0,
            maximum_depth: 4_096,
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
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap();
    assert_eq!(rebuilt, error);
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
enum GenericReply<T> {
    Pending,
    One(T),
    Pair(T, i64),
    Named { value: T, label: String },
}

#[test]
fn generic_enum_derives_bound_every_payload_shape() {
    let budget = || Budget {
        remaining: 50,
        reader: ReaderBudget { remaining: 128 },
        depth: 0,
        maximum_depth: 4_096,
    };
    for value in [
        GenericReply::Pending,
        GenericReply::One(7_i64),
        GenericReply::Pair(7_i64, 8),
        GenericReply::Named {
            value: 7_i64,
            label: "Ada".into(),
        },
    ] {
        let datom = value.datomize(vec![]);
        let rebuilt: GenericReply<i64> = datom.compose(&mut budget()).unwrap();
        assert_eq!(rebuilt, value);
    }
}

#[test]
fn datom_errors_round_trip_with_their_raising_layer() {
    let error = Error {
        layer: ErrorLayer::Datom,
        path: vec![1, 0],
        kind: ErrorKind::Form {
            expected: "Datom enclosure".into(),
            found: "Angled".into(),
        },
    };
    let text = error.datomize(vec![]).protosize().textualize();
    let rebuilt: Error = Potential::from(text)
        .actualize(&mut Budget {
            remaining: 30,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 4_096,
        })
        .unwrap();
    assert_eq!(rebuilt, error);
}

#[test]
fn manually_built_hundred_thousand_deep_datom_projects_and_drops_iteratively() {
    let mut datom = Datom {
        path: vec![],
        form: Form::Bare("x".into()),
    };
    for _ in 0..100_000 {
        datom = Datom {
            path: vec![],
            form: Form::Variant(protos::Symbol("V".into()), Box::new(datom)),
        };
    }
    let protos = datom.protosize();
    assert_eq!(protos.textualize().len(), 200_001);
    drop(protos);
    drop(datom);
}

#[test]
fn composition_depth_is_bounded_independently_from_node_budget() {
    let mut datom = Datom {
        path: vec![],
        form: Form::Bare("Pending".into()),
    };
    for _ in 0..128 {
        datom = Datom {
            path: vec![],
            form: Form::Variant(protos::Symbol("Next".into()), Box::new(datom)),
        };
    }
    let error = datom
        .compose::<Recursive>(&mut Budget {
            remaining: 1_000_000,
            reader: ReaderBudget {
                remaining: 1_000_000,
            },
            depth: 0,
            maximum_depth: 32,
        })
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Budget);
}

#[derive(Debug, PartialEq)]
struct Recursive(usize);
impl Compositional for Recursive {
    const ARITY: i64 = 0;
    fn from_positions(_: datom_codec::Positions<'_>, _: &mut Budget) -> Result<Self, Error> {
        unreachable!()
    }
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        use datom_codec::{Composable, Variantizing};
        if matches!(&datom.form, Form::Bare(name) if name == "Pending") {
            return Ok(Self(0));
        }
        let (head, body) = datom.variant(budget, "Recursive")?;
        if head != "Next" {
            return Err(datom_codec::Error::composition(
                datom.path.clone(),
                ErrorKind::Variant {
                    expected: "Recursive".into(),
                    found: head.into(),
                },
            ));
        }
        Ok(Self(body.compose::<Recursive>(budget)?.0 + 1))
    }
}

#[test]
fn wide_projection_keeps_canonical_extents() {
    let datom = Datom {
        path: vec![],
        form: Form::Struct(
            (0..10_000)
                .map(|index| Datom {
                    path: vec![index],
                    form: Form::Bare("x".into()),
                })
                .collect(),
        ),
    };
    let protos = datom.protosize();
    assert_eq!(protos.textualize().len(), 20_003);
    assert_eq!(
        protos.extent(),
        protos::Extent {
            start: 0,
            end: 20_003
        }
    );
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
enum RecursiveNode<T> {
    Leaf(T),
    Next(Box<RecursiveNode<T>>),
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
struct Chain {
    string: String,
    chain_option: Option<Box<Chain>>,
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
struct GenericChain<T> {
    value: T,
    next: Option<Box<GenericChain<T>>>,
}

#[test]
fn recursive_struct_derives_without_a_self_bound_cycle() {
    let value = Chain {
        string: "root".into(),
        chain_option: Some(Box::new(Chain {
            string: "tail".into(),
            chain_option: None,
        })),
    };
    let datom = value.datomize(vec![]);
    let rebuilt: Chain = datom
        .compose(&mut Budget {
            remaining: 100,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 128,
        })
        .unwrap();
    assert_eq!(rebuilt, value);

    let generic = GenericChain {
        value: 7_i64,
        next: Some(Box::new(GenericChain {
            value: 8,
            next: None,
        })),
    };
    let datom = generic.datomize(vec![]);
    let rebuilt: GenericChain<i64> = datom
        .compose(&mut Budget {
            remaining: 100,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 128,
        })
        .unwrap();
    assert_eq!(rebuilt, generic);
}

#[test]
fn recursive_generic_enum_derives_without_a_self_bound_cycle() {
    let value = RecursiveNode::Next(Box::new(RecursiveNode::Leaf(7_i64)));
    let datom = value.datomize(vec![]);
    let rebuilt: RecursiveNode<i64> = datom
        .compose(&mut Budget {
            remaining: 100,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 128,
        })
        .unwrap();
    assert_eq!(rebuilt, value);
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
struct NodeData<T>(T);
#[derive(Debug, PartialEq, Compositional, Datomizable)]
enum Node<T> {
    Data(NodeData<T>),
    Next(Box<Node<T>>),
}
#[derive(Debug, PartialEq, Compositional, Datomizable)]
enum MutualA<T> {
    End,
    Next(Box<MutualB<T>>),
}
#[derive(Debug, PartialEq, Compositional, Datomizable)]
enum MutualB<T> {
    Next(Box<MutualA<T>>),
    Data(T),
}

#[test]
fn exact_self_detection_bounds_similarly_named_wrappers_without_cycles() {
    let value = Node::Data(NodeData(7_i64));
    let datom = value.datomize(vec![]);
    let rebuilt: Node<i64> = datom
        .compose(&mut Budget {
            remaining: 100,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 128,
        })
        .unwrap();
    assert_eq!(rebuilt, value);
    let mutual = MutualA::Next(Box::new(MutualB::Data(8_i64)));
    let datom = mutual.datomize(vec![]);
    let rebuilt: MutualA<i64> = datom
        .compose(&mut Budget {
            remaining: 100,
            reader: ReaderBudget { remaining: 128 },
            depth: 0,
            maximum_depth: 128,
        })
        .unwrap();
    assert_eq!(rebuilt, mutual);
}

#[test]
fn manually_built_hundred_thousand_deep_protos_refuses_or_forms_without_recursion() {
    let mut protos = protos::Protos::Bare {
        extent: protos::Extent { start: 0, end: 1 },
        text: "x".into(),
    };
    for index in 0..100_000 {
        protos = protos::Protos::Headed {
            extent: protos::Extent {
                start: 0,
                end: index + 3,
            },
            head: protos::Symbol("V".into()),
            constraints: None,
            separator: protos::Separator::Period,
            body: Box::new(protos),
        };
    }
    let error = protos
        .datomize(vec![])
        .expect_err("manual headed tree reaches the typed depth refusal");
    assert_eq!(error.kind, ErrorKind::Budget);
    drop(protos);
}
