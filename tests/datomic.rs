use std::collections::BTreeMap;

use datomic::{
    Datom, Datomic, DatomicActualizable, Enclosure, Expected, Fault, MeaningValue, Printing,
    Problem, Protoform, Protosizable, Separator, Textualizable,
};
use protos::{Conceptual, Potential, Structural};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn actualize<T: Datomic>(source: &str) -> Result<T, datomic::Situated> {
    let pot: Potential<T> = Potential::from(source);
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
    round_trip::<i64>("9223372036854775807", i64::MAX);
    round_trip::<i64>("-9223372036854775808", i64::MIN);
}

#[test]
fn integer_rejects_invalid() {
    assert!(actualize::<i64>("+1").is_err());
    assert!(actualize::<i64>("01").is_err());
    assert!(actualize::<i64>("-0").is_err());
    assert!(actualize::<i64>("abc").is_err());
}

#[test]
fn boolean_round_trips() {
    round_trip::<bool>("True", true);
    round_trip::<bool>("False", false);
}

#[test]
fn boolean_rejects_lowercase() {
    assert!(actualize::<bool>("true").is_err());
    assert!(actualize::<bool>("false").is_err());
}

#[test]
fn decimal_round_trips() {
    round_trip::<f64>("3.125", 3.125);
    round_trip::<f64>("-0.5", -0.5);
    round_trip::<f64>("1.0", 1.0);
    round_trip::<f64>("0.0000001", 0.0000001);
}

#[test]
fn decimal_rejects_no_dot() {
    assert!(actualize::<f64>("1").is_err());
}

// ---------------------------------------------------------------------------
// String tests (bare-string rule)
// ---------------------------------------------------------------------------

#[test]
fn string_bare_words_round_trip() {
    round_trip::<String>("alpha", "alpha".to_owned());
    round_trip::<String>("42", "42".to_owned());
}

#[test]
fn string_with_separators_are_bare() {
    // The bare-string rule: a:b, a.b are bare in a Text position
    round_trip::<String>("name:first", "name:first".to_owned());
    round_trip::<String>("a.b", "a.b".to_owned());
    round_trip::<String>("a!b", "a!b".to_owned());
}

#[test]
fn string_with_spaces_uses_curly_quotes() {
    let value: String = actualize("\u{201C}hello world\u{201D}").unwrap();
    assert_eq!(value, "hello world");
    assert_eq!(value.textualize(), "\u{201C}hello world\u{201D}");
}

#[test]
fn string_timestamp_is_bare() {
    round_trip::<String>("2026-09-03T17:46:20", "2026-09-03T17:46:20".to_owned());
}

#[test]
fn string_url_is_bare() {
    round_trip::<String>("http://x", "http://x".to_owned());
}

// ---------------------------------------------------------------------------
// Meaning tests
// ---------------------------------------------------------------------------

#[test]
fn meaning_round_trips() {
    let value: MeaningValue = actualize("(hello world)").unwrap();
    assert_eq!(value, MeaningValue::Plain("hello world".to_owned()));
    assert_eq!(value.textualize(), "(hello world)");
}

#[test]
fn meaning_with_balanced_parens() {
    let value: MeaningValue =
        actualize("(The build passed on the third try (after two timeouts))").unwrap();
    assert_eq!(
        value,
        MeaningValue::Plain("The build passed on the third try (after two timeouts)".to_owned())
    );
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
    let mut expected = BTreeMap::new();
    expected.insert("born".to_owned(), 1990i64);
    expected.insert("name:first".to_owned(), 0i64); // demonstrating the bare-string key rule
    // Note: this specific map would need keys as strings. Let me use a simpler example.
    let source = "\u{00AB} alpha 1 beta 2 \u{00BB}";
    let map: BTreeMap<String, i64> = actualize(source).unwrap();
    assert_eq!(map.len(), 2);
    assert_eq!(map["alpha"], 1);
    assert_eq!(map["beta"], 2);
    // Round-trip: keys in BTreeMap are sorted
    let text = map.textualize();
    let re_map: BTreeMap<String, i64> = actualize(&text).unwrap();
    assert_eq!(re_map, map);
}

#[test]
fn map_duplicate_key_faults() {
    let result = actualize::<BTreeMap<String, i64>>("\u{00AB} alpha 1 alpha 2 \u{00BB}");
    assert!(result.is_err());
}

#[test]
fn map_odd_count_faults() {
    let result = actualize::<BTreeMap<String, i64>>("\u{00AB} alpha \u{00BB}");
    assert!(result.is_err());
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
// Hand-written struct and enum fixtures (what ethos-zero will generate)
// ---------------------------------------------------------------------------

// --- Address ---
#[derive(Debug, PartialEq, Clone)]
struct Address(String, String, String); // street, city, zip

impl Datomic for Address {
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
                let street = String::incorporate(it.next().unwrap())?;
                let city = String::incorporate(it.next().unwrap())?;
                let zip = String::incorporate(it.next().unwrap())?;
                Ok(Address(street, city, zip))
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Struct, datom),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        Datom::Struct(vec![
            self.0.datomize(),
            self.1.datomize(),
            self.2.datomize(),
        ])
    }
}

// --- Role ---
#[derive(Debug, PartialEq, Clone)]
enum Role {
    Author,
    Reviewer(i64, i64), // year, count
}

impl Datomic for Role {
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
                            Datom::Struct(fields) => {
                                if fields.len() != 2 {
                                    return Err(Fault::Corporal(
                                        vec![],
                                        Problem::Arity(2, fields.len() as i64),
                                    ));
                                }
                                let year = i64::incorporate(fields[0].clone())?;
                                let count = i64::incorporate(fields[1].clone())?;
                                Ok(Role::Reviewer(year, count))
                            }
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

    fn datomize(&self) -> Datom {
        match self {
            Role::Author => Datom::Bare("Author".to_owned()),
            Role::Reviewer(year, count) => Datom::Variant(
                "Reviewer".to_owned(),
                Separator::Period,
                Some(Box::new(Datom::Struct(vec![
                    year.datomize(),
                    count.datomize(),
                ]))),
            ),
        }
    }
}

// --- Person ---
#[derive(Debug, PartialEq, Clone)]
struct Person(String, i64, Address, Vec<Role>);

impl Datomic for Person {
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
                let name = String::incorporate(it.next().unwrap())?;
                let born = i64::incorporate(it.next().unwrap())?;
                let address = Address::incorporate(it.next().unwrap())?;
                let roles = Vec::<Role>::incorporate(it.next().unwrap())?;
                Ok(Person(name, born, address, roles))
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::Shape(Expected::Struct, datom),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        Datom::Struct(vec![
            self.0.datomize(),
            self.1.datomize(),
            self.2.datomize(),
            self.3.datomize(),
        ])
    }
}

// --- Reply ---
#[derive(Debug, PartialEq, Clone)]
enum Reply {
    Accepted(i64, String),
    Refused(String, i64),
    Pending,
}

impl Datomic for Reply {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) if s == "Pending" => Ok(Reply::Pending),
            Datom::Variant(head, sep, body) => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match head.as_str() {
                    "Accepted" => match body {
                        Some(b) => match b.as_ref() {
                            Datom::Struct(fields) => {
                                if fields.len() != 2 {
                                    return Err(Fault::Corporal(
                                        vec![],
                                        Problem::Arity(2, fields.len() as i64),
                                    ));
                                }
                                let id = i64::incorporate(fields[0].clone())?;
                                let at = String::incorporate(fields[1].clone())?;
                                Ok(Reply::Accepted(id, at))
                            }
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
                    "Refused" => match body {
                        Some(b) => match b.as_ref() {
                            Datom::Struct(fields) => {
                                if fields.len() != 2 {
                                    return Err(Fault::Corporal(
                                        vec![],
                                        Problem::Arity(2, fields.len() as i64),
                                    ));
                                }
                                let reason = String::incorporate(fields[0].clone())?;
                                let code = i64::incorporate(fields[1].clone())?;
                                Ok(Reply::Refused(reason, code))
                            }
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

// ---------------------------------------------------------------------------
// Orchestrate Lock family
// ---------------------------------------------------------------------------

type LockId = i64;
type LockName = String;
type FlowId = String;
type LockPath = String;

#[derive(Debug, PartialEq, Clone)]
struct Lock(LockId, LockName, FlowId, Vec<LockPath>, String);

impl Datomic for Lock {
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

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
enum ObserveSelection {
    Locks,
}

impl Datomic for ObserveSelection {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Bare(s) if s == "Locks" => Ok(Self::Locks),
            _ => Err(Fault::Corporal(
                vec![],
                Problem::UnknownVariant("(not Locks)".to_owned()),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        match self {
            Self::Locks => Datom::Bare("Locks".to_owned()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Observation {
    Locks(Vec<Lock>),
}

impl Datomic for Observation {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Variant(head, sep, body) if head == "Locks" => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match body {
                    Some(b) => Vec::<Lock>::incorporate(*b.clone()).map(Self::Locks),
                    None => Err(Fault::Corporal(
                        vec![],
                        Problem::Shape(Expected::Vector, datom),
                    )),
                }
            }
            _ => Err(Fault::Corporal(
                vec![],
                Problem::UnknownVariant("(not Locks)".to_owned()),
            )),
        }
    }

    fn datomize(&self) -> Datom {
        match self {
            Self::Locks(locks) => Datom::Variant(
                "Locks".to_owned(),
                Separator::Period,
                Some(Box::new(locks.datomize())),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum OrchestrateReply {
    Locked(Box<Lock>),
    Released(Box<Lock>),
    ReleaseRejected(String),
    Observed(Box<Observation>),
}

impl Datomic for OrchestrateReply {
    fn incorporate(datom: Datom) -> Result<Self, Fault> {
        match &datom {
            Datom::Variant(head, sep, body) => {
                if *sep != Separator::Period {
                    return Err(Fault::Corporal(vec![], Problem::Separator(*sep)));
                }
                match head.as_str() {
                    "Locked" => match body {
                        Some(b) => Lock::incorporate(*b.clone()).map(|l| Self::Locked(Box::new(l))),
                        None => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Struct, datom),
                        )),
                    },
                    "Released" => match body {
                        Some(b) => {
                            Lock::incorporate(*b.clone()).map(|l| Self::Released(Box::new(l)))
                        }
                        None => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Struct, datom),
                        )),
                    },
                    "ReleaseRejected" => match body {
                        Some(b) => match b.as_ref() {
                            Datom::Bare(s) if s == "UnknownLockId" => {
                                Ok(Self::ReleaseRejected("UnknownLockId".to_owned()))
                            }
                            _ => Err(Fault::Corporal(
                                vec![],
                                Problem::UnknownVariant("(unknown rejection)".to_owned()),
                            )),
                        },
                        None => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Variant, datom),
                        )),
                    },
                    "Observed" => match body {
                        Some(b) => Observation::incorporate(*b.clone())
                            .map(|o| Self::Observed(Box::new(o))),
                        None => Err(Fault::Corporal(
                            vec![],
                            Problem::Shape(Expected::Variant, datom),
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

    fn datomize(&self) -> Datom {
        match self {
            Self::Locked(lock) => Datom::Variant(
                "Locked".to_owned(),
                Separator::Period,
                Some(Box::new(lock.datomize())),
            ),
            Self::Released(lock) => Datom::Variant(
                "Released".to_owned(),
                Separator::Period,
                Some(Box::new(lock.datomize())),
            ),
            Self::ReleaseRejected(reason) => Datom::Variant(
                "ReleaseRejected".to_owned(),
                Separator::Period,
                Some(Box::new(Datom::Bare(reason.clone()))),
            ),
            Self::Observed(obs) => Datom::Variant(
                "Observed".to_owned(),
                Separator::Period,
                Some(Box::new(obs.datomize())),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Vision/datom.md fixture tests
// ---------------------------------------------------------------------------

#[test]
fn vision_person_example() {
    // { Ada 1990 { "12 Rue de la Paix" Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }
    let source = "{ Ada 1990 { \u{201C}12 Rue de la Paix\u{201D} Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }";
    let person: Person = actualize(source).unwrap();
    assert_eq!(person.0, "Ada");
    assert_eq!(person.1, 1990);
    assert_eq!(
        person.2,
        Address(
            "12 Rue de la Paix".to_owned(),
            "Paris".to_owned(),
            "75002".to_owned()
        )
    );
    assert_eq!(person.3, vec![Role::Author, Role::Reviewer(2024, 17)]);
    assert_eq!(person.textualize(), source);
}

#[test]
fn vision_reply_accepted() {
    let source = "Accepted.{ 42 2026-09-03T17:46:20 }";
    let reply: Reply = actualize(source).unwrap();
    assert_eq!(reply, Reply::Accepted(42, "2026-09-03T17:46:20".to_owned()));
    assert_eq!(reply.textualize(), source);
}

#[test]
fn vision_reply_refused() {
    let source = "Refused.{ \u{201C}no such file: { } is content\u{201D} 2 }";
    let reply: Reply = actualize(source).unwrap();
    assert_eq!(
        reply,
        Reply::Refused("no such file: { } is content".to_owned(), 2)
    );
    assert_eq!(reply.textualize(), source);
}

#[test]
fn vision_reply_pending() {
    round_trip::<Reply>("Pending", Reply::Pending);
}

#[test]
fn vision_vector_of_integer() {
    let source = "[ 0 42 -42 ]";
    round_trip::<Vec<i64>>(source, vec![0, 42, -42]);
}

#[test]
fn vision_map_with_bare_string_keys() {
    // « name:first Ada  born 1990 » — the vision example demonstrates the bare-string
    // rule: name:first has a colon but it is content because the position holds a string.
    // Judgment call (6329f1): the vision labels this "a map of Text to Integer" but Ada is
    // not an integer; the actual type demonstrated is Text to Text.
    let source = "\u{00AB} name:first Ada born 1990 \u{00BB}";
    let map: BTreeMap<String, String> = actualize(source).unwrap();
    assert_eq!(map.len(), 2);
    assert_eq!(map["name:first"], "Ada");
    assert_eq!(map["born"], "1990");
}

#[test]
fn vision_observed_locks_empty() {
    let source = "Observed.Locks.[]";
    let reply: OrchestrateReply = actualize(source).unwrap();
    assert_eq!(
        reply,
        OrchestrateReply::Observed(Box::new(Observation::Locks(vec![])))
    );
    assert_eq!(reply.textualize(), source);
}

#[test]
fn vision_locked_example() {
    let source = "Locked.{ 442 MyLock 6329f1 [ /abs/path ] \u{201C}why I hold it\u{201D} }";
    let reply: OrchestrateReply = actualize(source).unwrap();
    match &reply {
        OrchestrateReply::Locked(lock) => {
            assert_eq!(lock.0, 442);
            assert_eq!(lock.1, "MyLock");
            assert_eq!(lock.2, "6329f1");
            assert_eq!(lock.3, vec!["/abs/path".to_owned()]);
            assert_eq!(lock.4, "why I hold it");
        }
        _ => panic!("expected Locked"),
    }
    assert_eq!(reply.textualize(), source);
}

#[test]
fn vision_release_rejected() {
    let source = "ReleaseRejected.UnknownLockId";
    let reply: OrchestrateReply = actualize(source).unwrap();
    assert_eq!(
        reply,
        OrchestrateReply::ReleaseRejected("UnknownLockId".to_owned())
    );
    assert_eq!(reply.textualize(), source);
}

// ---------------------------------------------------------------------------
// Proptest: textualize then actualize round-trips
// ---------------------------------------------------------------------------

use proptest::prelude::*;

proptest! {
    #[test]
    fn integer_textualize_then_actualize_round_trips(value: i64) {
        let text = value.textualize();
        let re_value: i64 = actualize(&text).unwrap();
        prop_assert_eq!(re_value, value);
    }

    #[test]
    fn boolean_textualize_then_actualize_round_trips(value: bool) {
        let text = value.textualize();
        let re_value: bool = actualize(&text).unwrap();
        prop_assert_eq!(re_value, value);
    }

    #[test]
    fn string_textualize_then_actualize_round_trips(value in "[a-zA-Z0-9/._:!-]{0,50}") {
        // Only test strings that don't contain problematic characters
        let text = value.textualize();
        let re_value: String = actualize(&text).unwrap();
        prop_assert_eq!(re_value, value);
    }

    #[test]
    fn option_integer_textualize_then_actualize_round_trips(value: Option<i64>) {
        let text = value.textualize();
        let re_value: Option<i64> = actualize(&text).unwrap();
        prop_assert_eq!(re_value, value);
    }

    #[test]
    fn vec_integer_textualize_then_actualize_round_trips(value in prop::collection::vec(any::<i64>(), 0..10)) {
        let text = value.textualize();
        let re_value: Vec<i64> = actualize(&text).unwrap();
        prop_assert_eq!(re_value, value);
    }
}

// ---------------------------------------------------------------------------
// Protoform print then delineate round-trip (from protos)
// ---------------------------------------------------------------------------

#[test]
fn datom_protosize_then_print_then_delineate_then_conceive_round_trips() {
    let datom = Datom::Struct(vec![
        Datom::Bare("alpha".to_owned()),
        Datom::Vector(vec![
            Datom::Bare("1".to_owned()),
            Datom::Bare("2".to_owned()),
        ]),
    ]);
    let pf = datom.protosize();
    let text = pf.print();
    let d = text.delineate().unwrap();
    let re_datom: Datom = d.conceive().unwrap();
    assert_eq!(re_datom, datom);
}

// ---------------------------------------------------------------------------
// Fault tests
// ---------------------------------------------------------------------------

#[test]
fn structural_fault_from_unclosed() {
    let result = actualize::<i64>("{ 42");
    assert!(result.is_err());
}

#[test]
fn conceptual_fault_from_odd_map() {
    // Map with odd count faults at conceive
    let pf = Protoform::Enclosed(
        Enclosure::Guillemets,
        vec![Protoform::Bare("only_one".to_owned())],
    );
    let result: Result<Datom, Fault> = pf.conceive();
    assert!(matches!(
        result,
        Err(Fault::Conceptual(_, Problem::Pairing))
    ));
}

#[test]
fn corporal_fault_from_wrong_type() {
    let result = actualize::<i64>("True");
    assert!(result.is_err());
}
