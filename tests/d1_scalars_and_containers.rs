use std::collections::BTreeMap;

use datomic::{
    Datomic, DatomicString, DecimalViewing, FaultProblem, FiniteDecimal, Text, TextEdge,
};

#[test]
fn scalars_round_trip_at_the_portion_edge() {
    for (input, value) in [("True", true), ("False", false)] {
        let text = Text::<bool>::from(input);
        assert_eq!(text.embody().expect("boolean embodies"), value);
        assert_eq!(value.textualize().as_ref(), input);
    }

    let integer = Text::<i64>::from("-42").embody().expect("integer embodies");
    assert_eq!(integer, -42);
    assert_eq!(integer.textualize().as_ref(), "-42");

    let decimal = Text::<FiniteDecimal>::from("-1.5")
        .embody()
        .expect("decimal embodies");
    assert_eq!(decimal.value(), -1.5);
    assert_eq!(decimal.textualize().as_ref(), "-1.5");

    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(FiniteDecimal::try_from(value).is_err());
    }
    let programmed = FiniteDecimal::try_from(1.25).expect("finite decimal constructs");
    assert_eq!(programmed.textualize().as_ref(), "1.25");
    let expanded = FiniteDecimal::try_from(1.0e20).expect("large finite decimal constructs");
    assert!(expanded.textualize().as_ref().contains('.'));
    assert!(!expanded.textualize().as_ref().contains(['e', 'E']));

    let string = Text::<DatomicString>::from("“inside } ]”")
        .embody()
        .expect("opaque string embodies");
    assert_eq!(string.as_ref(), "inside } ]");
    assert_eq!(string.textualize().as_ref(), "“inside } ]”");
}

#[test]
fn vectors_maps_and_options_use_headless_positional_portions() {
    let vector = Text::<Vec<i64>>::from("[1 -2]")
        .embody()
        .expect("vector embodies");
    assert_eq!(vector, vec![1, -2]);
    assert_eq!(vector.textualize().as_ref(), "[1 -2]");

    let map = Text::<BTreeMap<DatomicString, i64>>::from("«north 1 south 2»")
        .embody()
        .expect("map embodies");
    assert_eq!(map.len(), 2);
    assert!(
        map.iter()
            .any(|(key, value)| key.as_ref() == "north" && *value == 1)
    );
    assert!(
        map.iter()
            .any(|(key, value)| key.as_ref() == "south" && *value == 2)
    );
    assert_eq!(map.textualize().as_ref(), "«north 1 south 2»");

    let option = Text::<Option<DatomicString>>::from("Some.value")
        .embody()
        .expect("some embodies");
    assert!(matches!(option, Some(ref value) if value.as_ref() == "value"));
    assert_eq!(option.textualize().as_ref(), "Some.value");
    assert_eq!(None::<DatomicString>.textualize().as_ref(), "None");
}

#[test]
fn scalar_and_container_faults_retain_the_protos_extent() {
    let fault = Text::<bool>::from("true")
        .embody()
        .expect_err("lower boolean faults");
    assert!(matches!(fault.problem, FaultProblem::Value));
    assert_eq!(fault.extent.start, 0);
    assert_eq!(fault.extent.end, 4);

    for source in ["+1", "01", "-0", "9223372036854775808"] {
        let fault = Text::<i64>::from(source)
            .embody()
            .expect_err("noncanonical integer faults");
        assert!(matches!(fault.problem, FaultProblem::Protos));
        assert_eq!(fault.extent.start, 0);
        assert_eq!(fault.extent.end, source.len());
    }

    for (source, start, end) in [("1e3", 0, 3), ("1", 0, 1), (".1", 0, 1), ("1.", 2, 2)] {
        let fault = Text::<FiniteDecimal>::from(source)
            .embody()
            .expect_err("noncanonical decimal faults");
        assert!(matches!(fault.problem, FaultProblem::Protos));
        assert_eq!(fault.extent.start, start);
        assert_eq!(fault.extent.end, end);
    }

    let fault = Text::<BTreeMap<DatomicString, i64>>::from("«north»")
        .embody()
        .expect_err("unpaired map faults");
    assert!(matches!(fault.problem, FaultProblem::MapPair));
    assert_eq!(fault.extent.start, 0);
    assert_eq!(fault.extent.end, 9);

    let fault = Text::<Vec<i64>>::from("[1")
        .embody()
        .expect_err("unclosed vector faults");
    assert!(matches!(fault.problem, FaultProblem::Protos));
    assert_eq!(fault.extent.start, 0);
    assert_eq!(fault.extent.end, 2);

    let fault = Text::<BTreeMap<DatomicString, i64>>::from("«north 1 north 2»")
        .embody()
        .expect_err("duplicate map key faults");
    assert!(matches!(fault.problem, FaultProblem::DuplicateMapKey));
    assert_eq!(fault.extent.start, 10);
    assert_eq!(fault.extent.end, 15);
}
