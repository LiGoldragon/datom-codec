//! The composition layer's own contracts: budget, arity, variant shape, the
//! positional decimal, error paths, and the bare-string rule.

use datom_codec::{
    Actualizing, Budget, Composable, Compositional, Datom, Datomizable, Error, ErrorKind,
    ErrorLayer, Form, Path, Pathing, Potential,
};
use protos::{Protosizable, ReaderBudget, Textualizable};

fn budget(remaining: i64) -> Budget {
    Budget {
        remaining,
        reader: ReaderBudget { remaining: 4_096 },
        depth: 0,
        maximum_depth: 4_096,
    }
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
enum Shape {
    Unit,
    Empty(),
    Named {},
    Carrying(i64),
}

#[derive(Debug, PartialEq, Compositional, Datomizable)]
struct Triple {
    first: String,
    second: i64,
    third: bool,
}

#[test]
fn bare_variants_spend_the_composition_budget() {
    let datom = Datom {
        path: Path::new(),
        form: Form::Bare("Unit".into()),
    };
    assert_eq!(
        datom.compose::<Shape>(&mut budget(1)).unwrap(),
        Shape::Unit,
        "a spent budget still composes one variant"
    );
    let error = datom.compose::<Shape>(&mut budget(0)).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Budget);
}

#[test]
fn variants_carrying_nothing_round_trip_whatever_their_field_syntax() {
    for value in [Shape::Unit, Shape::Empty(), Shape::Named {}] {
        let text = value.datomize(Path::new()).protosize().textualize();
        let composed: Shape = Potential::from(text.as_str())
            .actualize(&mut budget(4))
            .unwrap_or_else(|error| panic!("{text} composes: {error:?}"));
        assert_eq!(composed, value);
    }
    assert_eq!(
        Shape::Empty()
            .datomize(Path::new())
            .protosize()
            .textualize(),
        "Empty"
    );
}

#[test]
fn a_struct_reports_its_declared_arity_at_its_own_path() {
    for count in [2usize, 4] {
        let error = Datom {
            path: vec![7],
            form: Form::Struct(
                (0..count)
                    .map(|index| Datom {
                        path: vec![7, index as i64],
                        form: Form::Bare("x".into()),
                    })
                    .collect(),
            ),
        }
        .compose::<Triple>(&mut budget(16))
        .unwrap_err();
        assert_eq!(error.path, vec![7]);
        assert_eq!(
            error.kind,
            ErrorKind::Arity {
                expected: 3,
                found: count as i64
            }
        );
    }
}

#[test]
fn integers_refuse_a_leading_plus() {
    assert_eq!(
        Potential::<i64>::from("4")
            .actualize(&mut budget(2))
            .unwrap(),
        4
    );
    let error = Potential::<i64>::from("+4")
        .actualize(&mut budget(2))
        .unwrap_err();
    assert_eq!(
        error.kind,
        ErrorKind::Value {
            expected: "Integer".into(),
            value: "+4".into()
        }
    );
}

#[test]
fn a_decimal_is_read_by_its_position_and_never_by_its_content() {
    assert_eq!(
        Potential::<f64>::from("2.75")
            .actualize(&mut budget(4))
            .unwrap(),
        2.75
    );
    assert_eq!(
        Potential::<f64>::from("-0.5")
            .actualize(&mut budget(4))
            .unwrap(),
        -0.5
    );
    assert_eq!(
        Potential::<String>::from("2.75")
            .actualize(&mut budget(4))
            .unwrap(),
        "2.75",
        "the same text is a string where the position says string"
    );
    for text in ["2.75.15", "1.x", "3.", "007.5"] {
        assert!(
            Potential::<f64>::from(text)
                .actualize(&mut budget(8))
                .is_err(),
            "{text} is not a Decimal"
        );
    }
    let error = Potential::<i64>::from("2.75")
        .actualize(&mut budget(4))
        .unwrap_err();
    assert_eq!(
        error.kind,
        ErrorKind::Form {
            expected: "Bare".into(),
            found: "Variant".into()
        }
    );
}

#[test]
fn the_decimal_ascent_is_total_and_its_text_is_refused_on_the_way_back() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let text = value.datomize(Path::new()).protosize().textualize();
        let error = Potential::<f64>::from(text.as_str())
            .actualize(&mut budget(4))
            .unwrap_err();
        assert!(
            matches!(error.kind, ErrorKind::Value { ref expected, .. } if expected == "Decimal"),
            "{text} refuses as a Decimal, got {error:?}"
        );
    }
    assert_eq!(
        3.0f64.datomize(Path::new()).protosize().textualize(),
        "3.0",
        "a finite decimal keeps its mandatory point"
    );
}

#[test]
fn every_datomized_node_carries_the_path_it_sits_at() {
    let error = Error {
        layer: ErrorLayer::Protos,
        path: vec![1, 2],
        kind: ErrorKind::Structural(protos::Error {
            extent: protos::Extent { start: 4, end: 5 },
            problem: protos::Problem::Unclosed('«'),
        }),
    };
    let tree = error.datomize(Path::new());
    let mut mismatches: Vec<(Path, Path)> = Vec::new();
    let mut work = vec![&tree];
    while let Some(datom) = work.pop() {
        let at = &datom.path;
        let children: Vec<(Path, &Datom)> = match &datom.form {
            Form::Struct(children) | Form::Vector(children) => children
                .iter()
                .enumerate()
                .map(|(index, child)| (at.child(index as i64), child))
                .collect(),
            Form::Variant(_, body) => vec![(at.child(1), body.as_ref())],
            _ => Vec::new(),
        };
        for (expected, child) in children {
            if child.path != expected {
                mismatches.push((expected, child.path.clone()));
            }
            work.push(child);
        }
    }
    assert_eq!(mismatches, Vec::new());
}

#[test]
fn a_bare_string_keeps_its_separators_unless_the_run_would_swallow_its_context() {
    for (value, canonical) in [
        ("a:b:c", "a:b:c"),
        ("gpt-5.6-luna", "gpt-5.6-luna"),
        ("https://example.org/a", "https://example.org/a"),
        ("a!b", "a!b"),
        ("Some.Ada", "Some.Ada"),
        ("5::7/128", "«5::7/128»"),
        (".a", "«.a»"),
        ("a.", "«a.»"),
        ("a..b", "«a..b»"),
        ("a:.b", "«a:.b»"),
    ] {
        let value = value.to_owned();
        let text = value.datomize(Path::new()).protosize().textualize();
        assert_eq!(text, canonical, "canonical text of {value}");
        let composed: String = Potential::from(text.as_str())
            .actualize(&mut budget(16))
            .unwrap_or_else(|error| panic!("{text} composes: {error:?}"));
        assert_eq!(composed, value, "round trip of {text} at the root");

        let carried = Some(value.clone())
            .datomize(Path::new())
            .protosize()
            .textualize();
        assert_eq!(carried, format!("Some.{canonical}"));
        let composed: Option<String> = Potential::from(carried.as_str())
            .actualize(&mut budget(16))
            .unwrap_or_else(|error| panic!("{carried} composes: {error:?}"));
        assert_eq!(
            composed,
            Some(value),
            "round trip of {carried} inside a variant"
        );
    }
}

proptest::proptest! {
    /// The build → print → parse direction, the one the ascent actually uses.
    /// The backslash is in the alphabet now that the protos writer escapes it
    /// (protos 0.30.0).
    #[test]
    fn built_strings_round_trip_at_a_root_and_inside_a_variant(
        value in "[a-zA-Z0-9 .!:;/«»(){}\\[\\]<>_\\\\-]{0,24}"
    ) {
        let text = value.datomize(Path::new()).protosize().textualize();
        let composed: String = Potential::from(text.as_str())
            .actualize(&mut budget(256))
            .unwrap_or_else(|error| panic!("{text} composes: {error:?}"));
        proptest::prop_assert_eq!(&composed, &value);

        let carried = Some(value.clone())
            .datomize(Path::new())
            .protosize()
            .textualize();
        let composed: Option<String> = Potential::from(carried.as_str())
            .actualize(&mut budget(256))
            .unwrap_or_else(|error| panic!("{carried} composes: {error:?}"));
        proptest::prop_assert_eq!(composed, Some(value));
    }
}
