//! Faults at every layer, each with its exact path and extent through actualize.

mod common;

use common::*;
use datom_codec::{
    Actualizable, Boundary, Datomic, Enclosure, Expected, Extent, Fault, Found, Locus, Pathed,
    Potential, Problem, Text,
};

fn fault<T: Datomic + std::fmt::Debug>(text: &str) -> Fault {
    Potential::<T>::from(text).actualize().unwrap_err()
}
fn corporate(path: &[i64], extent: Extent, problem: Problem) -> Fault {
    Fault::Corporate(
        Locus {
            path: path.to_vec(),
            extent,
        },
        problem,
    )
}
fn conceptual(path: &[i64], extent: Extent, problem: Problem) -> Fault {
    Fault::Conceptual(
        Locus {
            path: path.to_vec(),
            extent,
        },
        problem,
    )
}
fn at(text: &str, needle: &str) -> Extent {
    let start = text.rfind(needle).unwrap() as i64;
    Extent(start, start + needle.len() as i64)
}

#[test]
fn a_bad_element_is_reported_at_its_own_extent() {
    let text = "[ 1 x ]";
    assert_eq!(
        fault::<Vec<i64>>(text),
        corporate(&[1], at(text, "x"), Problem::Value("x".to_owned()))
    );
}

#[test]
fn a_bad_field_deep_in_the_person() {
    let text = "{ Ada 1990 { “12 Rue de la Paix” Paris 75002 } [ Author Reviewer.{ 2024 x } ] }";
    let f = fault::<Person>(text);
    assert_eq!(
        f,
        corporate(&[3, 1, 1, 1], at(text, "x"), Problem::Value("x".to_owned()))
    );
    assert_eq!(f.path(), &[3, 1, 1, 1]);
}

#[test]
fn three_deep_through_every_container() {
    // Deep(Vec<Option<Result<Box<Role>, Text>>>)
    let text = "{ [ None Some.Ok.Author Some.Ok.Reviewer.{ 1 x } ] }";
    assert_eq!(
        fault::<Deep>(text),
        corporate(
            &[0, 2, 1, 1, 1, 1],
            at(text, "x"),
            Problem::Value("x".to_owned())
        )
    );
    let text = "{ [ Some.Err.{ 1 } ] }";
    assert_eq!(
        fault::<Deep>(text),
        corporate(
            &[0, 0, 1, 1],
            at(text, "{ 1 }"),
            Problem::Shape(Expected::Text, Found::Struct)
        )
    );
    let text = "{ [ Some.Ok.Nope ] }";
    assert_eq!(
        fault::<Deep>(text),
        corporate(
            &[0, 0, 1, 1],
            at(text, "Nope"),
            Problem::UnknownVariant("Nope".to_owned())
        )
    );
    let text = "{ [ Maybe.1 ] }";
    assert_eq!(
        fault::<Deep>(text),
        corporate(
            &[0, 0],
            at(text, "Maybe.1"),
            Problem::UnknownVariant("Maybe".to_owned())
        )
    );
}

#[test]
fn a_variant_body_is_child_one() {
    let text = "Some.x";
    assert_eq!(
        fault::<Option<i64>>(text),
        corporate(&[1], at(text, "x"), Problem::Value("x".to_owned()))
    );
    let text = "Some.Some.x";
    assert_eq!(
        fault::<Option<Option<i64>>>(text),
        corporate(&[1, 1], at(text, "x"), Problem::Value("x".to_owned()))
    );
    let text = "Ok.{ 1 }";
    assert_eq!(
        fault::<Result<i64, i64>>(text),
        corporate(
            &[1],
            at(text, "{ 1 }"),
            Problem::Shape(Expected::Integer, Found::Struct)
        )
    );
}

#[test]
fn variant_forms() {
    let text = "Pending.42";
    assert_eq!(
        fault::<Reply>(text),
        corporate(
            &[],
            Extent(0, 10),
            Problem::Shape(Expected::Word, Found::Variant)
        )
    );
    let text = "Accepted";
    assert_eq!(
        fault::<Reply>(text),
        corporate(
            &[],
            Extent(0, 8),
            Problem::Shape(Expected::Variant, Found::Word)
        )
    );
    let text = "Accepted.42";
    assert_eq!(
        fault::<Reply>(text),
        corporate(
            &[1],
            at(text, "42"),
            Problem::Shape(Expected::Struct, Found::Word)
        )
    );
    let text = "{ 1 }";
    assert_eq!(
        fault::<Reply>(text),
        corporate(
            &[],
            Extent(0, 5),
            Problem::Shape(Expected::Variant, Found::Struct)
        )
    );
}

#[test]
fn arity_and_shape() {
    let text = "{ Ada }";
    assert_eq!(
        fault::<Scores>(text),
        corporate(&[], Extent(0, 7), Problem::Arity(2, 1))
    );
    let text = "{ Ada [ 1 ] 3 }";
    assert_eq!(
        fault::<Scores>(text),
        corporate(&[], Extent(0, 15), Problem::Arity(2, 3))
    );
    let text = "[ 1 ]";
    assert_eq!(
        fault::<i64>(text),
        corporate(
            &[],
            Extent(0, 5),
            Problem::Shape(Expected::Integer, Found::Vector)
        )
    );
    let text = "{ Ada 1 }";
    assert_eq!(
        fault::<Scores>(text),
        corporate(
            &[1],
            at(text, "1"),
            Problem::Shape(Expected::Vector, Found::Word)
        )
    );
    let text = "{ Ada (x) }";
    assert_eq!(
        fault::<Remark>(text),
        corporate(
            &[1],
            at(text, "(x)"),
            Problem::Shape(Expected::Text, Found::Meaning)
        )
    );
    let text = "{ Ada “x” }";
    assert_eq!(
        fault::<Note>(text),
        corporate(
            &[1],
            at(text, "“x”"),
            Problem::Shape(Expected::Meaning, Found::Text)
        )
    );
    let text = "Maybe";
    assert_eq!(
        fault::<bool>(text),
        corporate(&[], Extent(0, 5), Problem::Value("Maybe".to_owned()))
    );
}

#[test]
fn conceptual_faults_name_the_form_found() {
    let text = "{ Ada <a> }";
    assert_eq!(
        fault::<Scores>(text),
        conceptual(&[1], at(text, "<a>"), Problem::Formless(Found::Angled))
    );
    let text = "[ 1 Vector<Text> ]";
    assert_eq!(
        fault::<Vec<i64>>(text),
        conceptual(
            &[1],
            at(text, "Vector<Text>"),
            Problem::Formless(Found::Qualified)
        )
    );
    let text = "Some.{ [ A<b>.{ 1 } ] }";
    assert_eq!(
        fault::<Option<i64>>(text),
        conceptual(
            &[1, 0, 0],
            at(text, "A<b>.{ 1 }"),
            Problem::Formless(Found::Qualified)
        )
    );
}

#[test]
fn one_value() {
    assert_eq!(
        fault::<i64>("1 2"),
        conceptual(&[], Extent(0, 3), Problem::OneValue(2))
    );
    assert_eq!(
        fault::<i64>(""),
        conceptual(&[], Extent(0, 0), Problem::OneValue(0))
    );
    assert_eq!(
        fault::<i64>(" ; only a comment"),
        conceptual(&[], Extent(0, 0), Problem::OneValue(0))
    );
}

#[test]
fn structural_faults_pass_through() {
    assert_eq!(
        fault::<Scores>("{ Ada [ 1 }"),
        Fault::Structural(protos::Fault {
            extent: Extent(10, 11),
            problem: protos::Problem::Unopened(Enclosure::Braced)
        })
    );
    assert_eq!(
        fault::<Text>("“open"),
        Fault::Structural(protos::Fault {
            extent: Extent(0, 7),
            problem: protos::Problem::Unterminated(Boundary::CurlyQuotes)
        })
    );
    assert_eq!(
        Fault::Structural(protos::Fault {
            extent: Extent(0, 1),
            problem: protos::Problem::Unopened(Enclosure::Braced)
        })
        .path(),
        &[] as &[i64]
    );
}

#[test]
fn faults_are_themselves_datomic() {
    let f = fault::<Vec<i64>>("[ 1 x ]");
    let text = f.textualize();
    assert_eq!(text, "Corporate.{ { [ 1 ] { 4 5 } } Value.x }");
    let back: Fault = Potential::<Fault>::from(text.as_str()).actualize().unwrap();
    assert_eq!(back, f);
    let s = fault::<Scores>("{ Ada [ 1 }");
    assert_eq!(s.textualize(), "Structural.{ { 10 11 } Unopened.Braced }");
    let back: Fault = Potential::<Fault>::from(s.textualize().as_str())
        .actualize()
        .unwrap();
    assert_eq!(back, s);
    let c = fault::<Scores>("{ Ada <a> }");
    assert_eq!(
        c.textualize(),
        "Conceptual.{ { [ 1 ] { 6 9 } } Formless.Angled }"
    );
}
