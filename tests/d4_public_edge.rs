use datomic::{Datomic, FaultProblem, Text, TextEdge};

#[test]
fn public_edge_embodies_prospective_text_or_returns_an_extent_fault() {
    let value = Text::<Option<i64>>::from(";; input trivia\nSome.-42")
        .embody()
        .expect("text enters through the sole public edge");
    assert_eq!(value, Some(-42));

    let fault = Text::<Option<i64>>::from("Some.01")
        .embody()
        .expect_err("invalid typed payload faults");
    assert!(matches!(fault.problem, FaultProblem::Value));
    assert_eq!(fault.extent.start, 5);
    assert_eq!(fault.extent.end, 7);
}

#[test]
fn public_edge_textualizes_typed_values_as_canonical_text() {
    let value = Some(-42_i64);
    let text = value.textualize();
    assert_eq!(text.as_ref(), "Some.-42");
    assert_eq!(
        text.embody().expect("canonical text returns to type"),
        value
    );
}
