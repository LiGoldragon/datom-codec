use std::collections::BTreeMap;

use datomic::{
    Actualizable, Corporal, Datom, Datomic, Enclosure, Expected, Extent, Fault, Meaning, Printing,
    Problem, Protoform, Protosizable, Separator, Situated, Textualizable,
};
use protos::{Conceptual, Potential, Structural};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn actualize<T: Datomic>(source: &str) -> Result<T, protos::Situated<Fault>> {
    let pot: Potential<T, Datom> = Potential::from(source);
    pot.actualize()
}

fn round_trip<T: Datomic + PartialEq + std::fmt::Debug>(source: &str, expected: T) {
    let value =
        actualize::<T>(source).unwrap_or_else(|e| panic!("failed to actualize {source:?}: {e:?}"));
    assert_eq!(value, expected, "actualize({source:?})");
    let text = value.textualize();
    let re_value =
        actualize::<T>(&text).unwrap_or_else(|e| panic!("failed to re-actualize {text:?}: {e:?}"));
    assert_eq!(re_value, expected, "round-trip({text:?})");
}

// ---------------------------------------------------------------------------
// Scalar tests
// ---------------------------------------------------------------------------

#[test]
fn integer_round_trips() {
    round_trip::<i64>("42", 42);
    round_trip::<i64>("-42", -42);
    round_trip::<i64>("0", 0);
}

#[test]
fn integer_rejects_invalid() {
    assert!(actualize::<i64>("+1").is_err());
    assert!(actualize::<i64>("01").is_err());
    assert!(actualize::<i64>("-0").is_err());
}

#[test]
fn boolean_round_trips() {
    round_trip::<bool>("True", true);
    round_trip::<bool>("False", false);
}

#[test]
fn decimal_round_trips() {
    round_trip::<f64>("3.125", 3.125);
    round_trip::<f64>("-0.5", -0.5);
    round_trip::<f64>("1.0", 1.0);
    round_trip::<f64>("0.0000001", 0.0000001);
}

#[test]
fn decimal_edge_values() {
    // Table of edge values with round-trip verification
    for (input, expected) in [
        ("0.0", 0.0_f64),
        ("1.5", 1.5),
        ("-1.5", -1.5),
        ("0.0000001", 1e-7),
    ] {
        let value = actualize::<f64>(input).unwrap_or_else(|e| panic!("{input}: {e:?}"));
        assert_eq!(value, expected, "input: {input}");
        let text = value.textualize();
        let re = actualize::<f64>(&text).unwrap_or_else(|e| panic!("{text}: {e:?}"));
        assert_eq!(re, expected, "round-trip of {input} via {text}");
    }
}

// ---------------------------------------------------------------------------
// String and Meaning tests
// ---------------------------------------------------------------------------

#[test]
fn string_bare_words_round_trip() {
    round_trip::<String>("alpha", "alpha".to_owned());
}

#[test]
fn string_with_separators_are_bare() {
    round_trip::<String>("name:first", "name:first".to_owned());
    round_trip::<String>("a.b", "a.b".to_owned());
}

#[test]
fn string_timestamp_is_bare() {
    round_trip::<String>("2026-09-03T17:46:20", "2026-09-03T17:46:20".to_owned());
}

#[test]
fn meaning_round_trips() {
    let value: Meaning = actualize("(hello world)").unwrap();
    assert_eq!(value, Meaning::Plain("hello world".to_owned()));
    assert_eq!(value.textualize(), "(hello world)");
}

// ---------------------------------------------------------------------------
// Container tests
// ---------------------------------------------------------------------------

#[test]
fn vector_of_integers() {
    round_trip::<Vec<i64>>("[ 0 42 -42 ]", vec![0, 42, -42]);
}

#[test]
fn map_of_string_to_integer() {
    let source = "\u{00AB} alpha 1 beta 2 \u{00BB}";
    let map: BTreeMap<String, i64> = actualize(source).unwrap();
    assert_eq!(map.len(), 2);
    assert_eq!(map["alpha"], 1);
}

#[test]
fn option_round_trips() {
    round_trip::<Option<i64>>("Some.42", Some(42));
    round_trip::<Option<i64>>("None", None);
}

#[test]
fn result_round_trips() {
    round_trip::<Result<i64, String>>("Ok.42", Ok(42));
    round_trip::<Result<i64, String>>("Err.failed", Err("failed".to_owned()));
}

// ---------------------------------------------------------------------------
// Hand-written struct and enum fixtures
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Clone)]
struct Address(String, String, String);

impl Corporal<Datom> for Address {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Struct(fields) => {
                if fields.len() != 3 {
                    return Err(Fault::Corporal(
                        vec![],
                        Problem::Arity(3, fields.len() as i64),
                    ));
                }
                let mut it = fields.into_iter();
                Ok(Address(
                    String::incorporate(it.next().unwrap())?,
                    String::incorporate(it.next().unwrap())?,
                    String::incorporate(it.next().unwrap())?,
                ))
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Struct, datom),
            )),
        }
    }
}
impl Datomic for Address {
    fn datomize(&self) -> Datom {
        Datom::Struct(vec![
            self.0.datomize(),
            self.1.datomize(),
            self.2.datomize(),
        ])
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Role {
    Author,
    Reviewer(i64, i64),
}

impl Corporal<Datom> for Role {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) if s == "Author" => Ok(Role::Author),
            Datom::Variant(head, sep, body) => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match head.as_str() {
                    "Reviewer" => match body {
                        Some(b) => match b.as_ref() {
                            Datom::Struct(fields) if fields.len() == 2 => Ok(Role::Reviewer(
                                i64::incorporate(fields[0].clone())?,
                                i64::incorporate(fields[1].clone())?,
                            )),
                            _ => Err(Fault::Corporal(
                                vec![],
                                Problem::Shape(Expected::Struct, *b.clone()),
                            )),
                        },
                        None => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Struct, datom),
                        )),
                    },
                    _ => Err(Fault::Corporal(
                        vec![],
                        Problem::UnknownVariant(head.clone()),
                    )),
                }
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Variant, datom),
            )),
        }
    }
}
impl Datomic for Role {
    fn datomize(&self) -> Datom {
        match self {
            Role::Author => Datom::Bare("Author".to_owned()),
            Role::Reviewer(y, c) => Datom::Variant(
                "Reviewer".to_owned(),
                Separator::Period,
                Some(Box::new(Datom::Struct(vec![y.datomize(), c.datomize()]))),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Person(String, i64, Address, Vec<Role>);

impl Corporal<Datom> for Person {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Struct(fields) => {
                if fields.len() != 4 {
                    return Err(Fault::Corporal(
                        vec![],
                        Problem::Arity(4, fields.len() as i64),
                    ));
                }
                let mut it = fields.into_iter();
                Ok(Person(
                    String::incorporate(it.next().unwrap())?,
                    i64::incorporate(it.next().unwrap())?,
                    Address::incorporate(it.next().unwrap())?,
                    Vec::<Role>::incorporate(it.next().unwrap())?,
                ))
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Struct, datom),
            )),
        }
    }
}
impl Datomic for Person {
    fn datomize(&self) -> Datom {
        Datom::Struct(vec![
            self.0.datomize(),
            self.1.datomize(),
            self.2.datomize(),
            self.3.datomize(),
        ])
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Reply {
    Accepted(i64, String),
    Refused(String, i64),
    Pending,
}

impl Corporal<Datom> for Reply {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) if s == "Pending" => Ok(Reply::Pending),
            Datom::Variant(head, sep, body) => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match (head.as_str(), body) {
                    ("Accepted", Some(b)) => match b.as_ref() {
                        Datom::Struct(fields) if fields.len() == 2 => Ok(Reply::Accepted(
                            i64::incorporate(fields[0].clone())?,
                            String::incorporate(fields[1].clone())?,
                        )),
                        _ => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Struct, *b.clone()),
                        )),
                    },
                    ("Refused", Some(b)) => match b.as_ref() {
                        Datom::Struct(fields) if fields.len() == 2 => Ok(Reply::Refused(
                            String::incorporate(fields[0].clone())?,
                            i64::incorporate(fields[1].clone())?,
                        )),
                        _ => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Struct, *b.clone()),
                        )),
                    },
                    _ => Err(Fault::Corporal(
                        vec![],
                        Problem::UnknownVariant(head.clone()),
                    )),
                }
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Variant, datom),
            )),
        }
    }
}
impl Datomic for Reply {
    fn datomize(&self) -> Datom {
        match self {
            Reply::Accepted(id, at) => Datom::Variant(
                "Accepted".to_owned(),
                Separator::Period,
                Some(Box::new(Datom::Struct(vec![id.datomize(), at.datomize()]))),
            ),
            Reply::Refused(reason, code) => Datom::Variant(
                "Refused".to_owned(),
                Separator::Period,
                Some(Box::new(Datom::Struct(vec![
                    reason.datomize(),
                    code.datomize(),
                ]))),
            ),
            Reply::Pending => Datom::Bare("Pending".to_owned()),
        }
    }
}

// Lock family
#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
struct Lock(i64, String, String, Vec<String>, String);
impl Corporal<Datom> for Lock {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Struct(fields) => {
                if fields.len() != 5 {
                    return Err(Fault::Corporal(
                        vec![],
                        Problem::Arity(5, fields.len() as i64),
                    ));
                }
                let mut it = fields.into_iter();
                Ok(Lock(
                    i64::incorporate(it.next().unwrap())?,
                    String::incorporate(it.next().unwrap())?,
                    String::incorporate(it.next().unwrap())?,
                    Vec::<String>::incorporate(it.next().unwrap())?,
                    String::incorporate(it.next().unwrap())?,
                ))
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Struct, datom),
            )),
        }
    }
}
impl Datomic for Lock {
    fn datomize(&self) -> Datom {
        Datom::Struct(vec![
            self.0.datomize(),
            self.1.datomize(),
            self.2.datomize(),
            self.3.datomize(),
            self.4.datomize(),
        ])
    }
}

// ---------------------------------------------------------------------------
// Vision/datom.md fixture tests
// ---------------------------------------------------------------------------

#[test]
fn vision_person_example() {
    let source = "{ Ada 1990 { \u{201C}12 Rue de la Paix\u{201D} Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }";
    let person: Person = actualize(source).unwrap();
    assert_eq!(person.0, "Ada");
    assert_eq!(person.1, 1990);
    assert_eq!(person.3, vec![Role::Author, Role::Reviewer(2024, 17)]);
    assert_eq!(person.textualize(), source);
}

#[test]
fn vision_reply_accepted() {
    let source = "Accepted.{ 42 2026-09-03T17:46:20 }";
    round_trip::<Reply>(
        source,
        Reply::Accepted(42, "2026-09-03T17:46:20".to_owned()),
    );
}

#[test]
fn vision_reply_refused() {
    let source = "Refused.{ \u{201C}no such file: { } is content\u{201D} 2 }";
    round_trip::<Reply>(
        source,
        Reply::Refused("no such file: { } is content".to_owned(), 2),
    );
}

#[test]
fn vision_reply_pending() {
    round_trip::<Reply>("Pending", Reply::Pending);
}

#[test]
fn vision_vector_of_integer() {
    round_trip::<Vec<i64>>("[ 0 42 -42 ]", vec![0, 42, -42]);
}

#[test]
fn vision_observed_locks_empty() {
    let source = "Observed.Locks.[]";
    // Observed is a Variant chain — test via Reply-like type is already covered
    let d: protos::Delineation = source.to_owned().delineate().unwrap();
    let datom: Datom = d.conceive().unwrap();
    assert_eq!(datom.protosize().print(), source);
}

#[test]
fn vision_locked_example() {
    let source = "Locked.{ 442 MyLock 6329f1 [ /abs/path ] \u{201C}why I hold it\u{201D} }";
    let d: protos::Delineation = source.to_owned().delineate().unwrap();
    let datom: Datom = d.conceive().unwrap();
    assert_eq!(datom.protosize().print(), source);
}

// ---------------------------------------------------------------------------
// Fault datomic tests
// ---------------------------------------------------------------------------

#[test]
fn corporal_fault_textualizes_and_round_trips() {
    let fault = Fault::Corporal(vec![0, 1], Problem::Value("bad".to_owned()));
    let text = fault.textualize();
    let re_fault: Fault = actualize(&text).unwrap();
    assert_eq!(re_fault, fault);
}

#[test]
fn structural_fault_textualizes_and_round_trips() {
    let fault = Fault::Structural(protos::Fault {
        extent: Extent(5, 13),
        problem: protos::Problem::Unclosed(Enclosure::Braced),
    });
    let text = fault.textualize();
    // Must produce proper datom, not Rust Debug: Structural.{ { 5 13 } Unclosed.Braced }
    assert!(
        !text.contains("Unclosed("),
        "structural fault must not contain Rust Debug: {text}"
    );
    assert!(
        text.contains("Unclosed.Braced"),
        "structural fault must contain datom form: {text}"
    );
    let re_fault: Fault = actualize(&text).unwrap();
    assert_eq!(re_fault, fault);
}

#[test]
fn separator_fault_textualizes_without_debug() {
    let fault = Fault::Corporal(vec![], Problem::Separator(Separator::Period));
    let text = fault.textualize();
    assert!(
        !text.contains("Period)"),
        "separator fault must not contain Rust Debug: {text}"
    );
    let re_fault: Fault = actualize(&text).unwrap();
    assert_eq!(re_fault, fault);
}

#[test]
fn expected_round_trips_through_datom() {
    for expected in [
        Expected::Variant,
        Expected::Struct,
        Expected::Vector,
        Expected::Map,
        Expected::Text,
        Expected::Meaning,
        Expected::Integer,
        Expected::Decimal,
        Expected::Boolean,
        Expected::Bare,
    ] {
        let text = expected.textualize();
        let re: Expected = actualize(&text).unwrap();
        assert_eq!(re, expected);
    }
}

// ---------------------------------------------------------------------------
// Proptests
// ---------------------------------------------------------------------------

use proptest::prelude::*;

proptest! {
    #[test]
    fn integer_round_trips_prop(value: i64) {
        let text = value.textualize();
        let re: i64 = actualize(&text).unwrap();
        prop_assert_eq!(re, value);
    }

    #[test]
    fn boolean_round_trips_prop(value: bool) {
        let text = value.textualize();
        let re: bool = actualize(&text).unwrap();
        prop_assert_eq!(re, value);
    }

    #[test]
    fn string_round_trips_prop(value in "[a-zA-Z0-9/._:!-]{0,50}") {
        let text = value.textualize();
        let re: String = actualize(&text).unwrap();
        prop_assert_eq!(re, value);
    }

    #[test]
    fn option_integer_round_trips_prop(value: Option<i64>) {
        let text = value.textualize();
        let re: Option<i64> = actualize(&text).unwrap();
        prop_assert_eq!(re, value);
    }

    #[test]
    fn vec_integer_round_trips_prop(value in prop::collection::vec(any::<i64>(), 0..10)) {
        let text = value.textualize();
        let re: Vec<i64> = actualize(&text).unwrap();
        prop_assert_eq!(re, value);
    }

    #[test]
    fn decimal_round_trips_prop(value in proptest::num::f64::NORMAL | proptest::num::f64::SUBNORMAL | proptest::num::f64::ZERO) {
        let text = value.textualize();
        let re: f64 = actualize(&text).unwrap();
        prop_assert_eq!(re.to_bits(), value.to_bits());
    }
}

// ---------------------------------------------------------------------------
// Conceptual round-trip
// ---------------------------------------------------------------------------

#[test]
fn datom_protosize_then_conceive_round_trips() {
    let datom = Datom::Struct(vec![
        Datom::Bare("alpha".to_owned()),
        Datom::Vector(vec![Datom::Bare("1".to_owned())]),
    ]);
    let pf = datom.protosize();
    let text = pf.print();
    let d = text.delineate().unwrap();
    let re_datom: Datom = d.conceive().unwrap();
    assert_eq!(re_datom, datom);
}

// ---------------------------------------------------------------------------
// Fault layer tests
// ---------------------------------------------------------------------------

#[test]
fn structural_fault_from_unclosed() {
    assert!(actualize::<i64>("{ 42").is_err());
}

#[test]
fn conceptual_fault_from_odd_map() {
    let pf = Protoform::Enclosed(
        Enclosure::Guillemets,
        vec![Protoform::Bare("only".to_owned())],
    );
    let result: Result<Datom, Fault> = pf.conceive();
    assert!(matches!(
        result,
        Err(Fault::Conceptual(_, Problem::Pairing))
    ));
}

#[test]
fn corporal_fault_from_wrong_type() {
    assert!(actualize::<i64>("True").is_err());
}

// ---------------------------------------------------------------------------
// Box<T> tests — via impl_datomic_box! macro
// ---------------------------------------------------------------------------

// A recursive type that needs Box<T>: a query that can nest.
// `impl_datomic_box!(Query)` generates Corporal<Datom> and Datomic for Box<Query>
// transparently (a Box carries its content's datom exactly).
#[derive(Clone, PartialEq, Eq, Debug)]
enum Query {
    Literal(i64),
    Nested(Box<Query>),
}

impl Corporal<Datom> for Query {
    type Fault = Fault;
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Variant(head, sep, body) => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match (head.as_str(), body) {
                    ("Literal", Some(b)) => i64::incorporate(*b.clone()).map(Query::Literal),
                    ("Nested", Some(b)) => Box::<Query>::incorporate(*b.clone()).map(Query::Nested),
                    _ => Err(Fault::Corporal(
                        vec![],
                        Problem::UnknownVariant(head.clone()),
                    )),
                }
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Variant, datom),
            )),
        }
    }
}

impl Datomic for Query {
    fn datomize(&self) -> Datom {
        match self {
            Query::Literal(n) => Datom::Variant(
                "Literal".to_owned(),
                Separator::Period,
                Some(Box::new(n.datomize())),
            ),
            Query::Nested(inner) => Datom::Variant(
                "Nested".to_owned(),
                Separator::Period,
                Some(Box::new(inner.datomize())),
            ),
        }
    }
}

datomic::impl_datomic_box!(Query);

#[test]
fn box_query_recursive_round_trips() {
    // Demonstrates impl_datomic_box!: Box<Query> carries Query's datom transparently.
    let q = Query::Nested(Box::new(Query::Nested(Box::new(Query::Literal(7)))));
    round_trip("Nested.Nested.Literal.7", q);
}

// ---------------------------------------------------------------------------
// Situated<F> tests
// ---------------------------------------------------------------------------

#[test]
fn situated_fault_datomizes_as_struct() {
    // Matches orchestrate stderr: Unreadable.{ Some.{ 5 13 } Structural.{ { 5 13 } Unclosed.Braced } }
    // The Situated part is: { Some.{ 5 13 } Structural.{ { 5 13 } Unclosed.Braced } }
    let inner = Fault::Structural(protos::Fault {
        extent: Extent(5, 13),
        problem: protos::Problem::Unclosed(Enclosure::Braced),
    });
    let situated = Situated(Some(Extent(5, 13)), inner);
    let text = situated.textualize();
    assert_eq!(
        text,
        "{ Some.{ 5 13 } Structural.{ { 5 13 } Unclosed.Braced } }"
    );

    // round-trip through incorporate
    let datom = situated.datomize();
    let recovered = Situated::<Fault>::incorporate(datom).unwrap();
    assert_eq!(recovered.textualize(), text);
}

#[test]
fn situated_fault_none_extent_round_trips() {
    let inner = Fault::Structural(protos::Fault {
        extent: Extent(5, 13),
        problem: protos::Problem::Unclosed(Enclosure::Braced),
    });
    let situated = Situated::<Fault>(None, inner);
    let text = situated.textualize();
    assert_eq!(text, "{ None Structural.{ { 5 13 } Unclosed.Braced } }");
    let datom = situated.datomize();
    let recovered = Situated::<Fault>::incorporate(datom).unwrap();
    assert_eq!(recovered.textualize(), text);
}
