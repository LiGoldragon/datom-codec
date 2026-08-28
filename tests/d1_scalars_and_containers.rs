use std::collections::BTreeMap;

use datomic::{Datomic, FaultProblem, Text, TextEdge};

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

    let decimal = Text::<f64>::from("-1.5")
        .embody()
        .expect("decimal embodies");
    assert_eq!(decimal, -1.5);
    assert_eq!(decimal.textualize().as_ref(), "-1.5");

    let string = Text::<String>::from("“inside } ]”")
        .embody()
        .expect("opaque string embodies");
    assert_eq!(string, "inside } ]");
    assert_eq!(string.textualize().as_ref(), "“inside } ]”");
}

#[test]
fn vectors_maps_and_options_use_headless_positional_portions() {
    let vector = Text::<Vec<i64>>::from("[1 -2]")
        .embody()
        .expect("vector embodies");
    assert_eq!(vector, vec![1, -2]);
    assert_eq!(vector.textualize().as_ref(), "[1 -2]");

    let map = Text::<BTreeMap<String, i64>>::from("«north 1 south 2»")
        .embody()
        .expect("map embodies");
    assert_eq!(map.get("north"), Some(&1));
    assert_eq!(map.get("south"), Some(&2));
    assert_eq!(map.textualize().as_ref(), "«north 1 south 2»");

    let option = Text::<Option<String>>::from("Some.value")
        .embody()
        .expect("some embodies");
    assert_eq!(option, Some("value".into()));
    assert_eq!(option.textualize().as_ref(), "Some.value");
    assert_eq!(None::<String>.textualize().as_ref(), "None");
}

#[test]
fn scalar_and_container_faults_retain_the_protos_extent() {
    let fault = Text::<bool>::from("true")
        .embody()
        .expect_err("lower boolean faults");
    assert!(matches!(fault.problem, FaultProblem::Value));
    assert_eq!(fault.extent.start, 0);
    assert_eq!(fault.extent.end, 4);

    let fault = Text::<f64>::from("1e3")
        .embody()
        .expect_err("exponent faults");
    assert!(matches!(fault.problem, FaultProblem::Value));

    let fault = Text::<BTreeMap<String, i64>>::from("«north»")
        .embody()
        .expect_err("unpaired map faults");
    assert!(matches!(fault.problem, FaultProblem::MapPair));
}
