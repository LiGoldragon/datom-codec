use datomic::{Datom, Datomic, Expected, Extent, Fault, Head, Meaning, Prepending, Problem};
use protos::{Conceivable, Protosizable, Textualizable};

// ---------------------------------------------------------------------------
// Test struct for typed round-trips
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
struct Person(String, Vec<i64>, Address);

#[derive(Debug, PartialEq)]
struct Address(String, String, String);

impl Conceivable<Datom> for Person {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<Datom, std::convert::Infallible> {
        Ok(Datom::Struct(vec![
            self.0.conceive()?,
            self.1.conceive()?,
            self.2.conceive()?,
        ]))
    }
}

impl Datomic for Person {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Struct(fields) if fields.len() == 3 => {
                let mut it = fields.into_iter();
                let name =
                    String::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(0))?;
                let scores =
                    Vec::<i64>::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(1))?;
                let address =
                    Address::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(2))?;
                Ok(Person(name, scores, address))
            }
            Datom::Struct(fields) => Err(Fault::Corporate(
                vec![],
                Problem::Arity(3, fields.len() as i64),
            )),
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Struct, other),
            )),
        }
    }
}

impl Conceivable<Datom> for Address {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<Datom, std::convert::Infallible> {
        Ok(Datom::Struct(vec![
            self.0.conceive()?,
            self.1.conceive()?,
            self.2.conceive()?,
        ]))
    }
}

impl Datomic for Address {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Struct(fields) if fields.len() == 3 => {
                let mut it = fields.into_iter();
                let street =
                    String::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(0))?;
                let city =
                    String::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(1))?;
                let zip = String::incorporate_from(it.next().unwrap()).map_err(|f| f.prepend(2))?;
                Ok(Address(street, city, zip))
            }
            Datom::Struct(fields) => Err(Fault::Corporate(
                vec![],
                Problem::Arity(3, fields.len() as i64),
            )),
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Struct, other),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Test enum for typed round-trips
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
enum Reply {
    Accepted(i64, String),
    Refused(String, i64),
    Pending,
}

impl Conceivable<Datom> for Reply {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<Datom, std::convert::Infallible> {
        Ok(match self {
            Reply::Accepted(id, ts) => Datom::Variant(
                Head::Bare("Accepted".to_owned()),
                Box::new(Datom::Struct(vec![id.conceive()?, ts.conceive()?])),
            ),
            Reply::Refused(reason, code) => Datom::Variant(
                Head::Bare("Refused".to_owned()),
                Box::new(Datom::Struct(vec![reason.conceive()?, code.conceive()?])),
            ),
            Reply::Pending => Datom::Bare("Pending".to_owned()),
        })
    }
}

impl Datomic for Reply {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(Head::Bare(name), body) => match name.as_str() {
                "Accepted" => match *body {
                    Datom::Struct(fields) if fields.len() == 2 => {
                        let mut it = fields.into_iter();
                        let id = i64::incorporate_from(it.next().unwrap())?;
                        let ts = String::incorporate_from(it.next().unwrap())?;
                        Ok(Reply::Accepted(id, ts))
                    }
                    other => Err(Fault::Corporate(
                        vec![],
                        Problem::Shape(Expected::Struct, other),
                    )),
                },
                "Refused" => match *body {
                    Datom::Struct(fields) if fields.len() == 2 => {
                        let mut it = fields.into_iter();
                        let reason = String::incorporate_from(it.next().unwrap())?;
                        let code = i64::incorporate_from(it.next().unwrap())?;
                        Ok(Reply::Refused(reason, code))
                    }
                    other => Err(Fault::Corporate(
                        vec![],
                        Problem::Shape(Expected::Struct, other),
                    )),
                },
                _ => Err(Fault::Corporate(vec![], Problem::UnknownVariant(name))),
            },
            Datom::Bare(s) if s == "Pending" => Ok(Reply::Pending),
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Variant, other),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Observation/Response enums for typed round-trips from vision
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
enum Observation {
    Locks(Vec<i64>),
}

impl Conceivable<Datom> for Observation {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<Datom, std::convert::Infallible> {
        match self {
            Observation::Locks(v) => Ok(Datom::Variant(
                Head::Bare("Locks".to_owned()),
                Box::new(v.conceive()?),
            )),
        }
    }
}

impl Datomic for Observation {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(Head::Bare(name), body) if name == "Locks" => {
                Vec::<i64>::incorporate_from(*body).map(Observation::Locks)
            }
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Variant, other),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Response {
    Observed(Observation),
    Success,
}

impl Conceivable<Datom> for Response {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<Datom, std::convert::Infallible> {
        Ok(match self {
            Response::Observed(obs) => {
                Datom::Variant(Head::Bare("Observed".to_owned()), Box::new(obs.conceive()?))
            }
            Response::Success => Datom::Bare("Success".to_owned()),
        })
    }
}

impl Datomic for Response {
    fn incorporate_from(datom: Datom) -> Result<Self, Fault> {
        match datom {
            Datom::Variant(Head::Bare(name), body) if name == "Observed" => {
                Observation::incorporate_from(*body).map(Response::Observed)
            }
            Datom::Bare(s) if s == "Success" => Ok(Response::Success),
            other => Err(Fault::Corporate(
                vec![],
                Problem::Shape(Expected::Variant, other),
            )),
        }
    }
}

datomic::impl_datomic_box!(Observation);

// ---------------------------------------------------------------------------
// Typed round-trip helpers
// ---------------------------------------------------------------------------

fn round_trip<T: Datomic>(source: &str) -> T {
    let text = source.to_owned();
    let delineation = text.protosize().expect("delineates");
    let datom: Datom = delineation.conceive().expect("conceives");
    T::incorporate_from(datom).expect("incorporates")
}

fn round_trip_text<T: Datomic>(value: &T) -> String {
    value.textualize()
}

// ---------------------------------------------------------------------------
// Vision/datom.md examples: typed round-trips
// ---------------------------------------------------------------------------

#[test]
fn person_example_round_trips() {
    let source = "{ Ada [ 12 7 -3 ] { \u{201C}12 Rue de la Paix\u{201D} Paris 75002 } }";
    let person: Person = round_trip(source);
    assert_eq!(person.0, "Ada");
    assert_eq!(person.1, vec![12, 7, -3]);
    assert_eq!(person.2.0, "12 Rue de la Paix");
    assert_eq!(person.2.1, "Paris");
    assert_eq!(person.2.2, "75002");
    assert_eq!(round_trip_text(&person), source);
}

#[test]
fn scores_example() {
    let source = "{ Ada [ 12 7 -3 ] { \u{201C}12 Rue de la Paix\u{201D} Paris 75002 } }";
    let person: Person = round_trip(source);
    assert_eq!(person.1, vec![12, 7, -3]);
}

#[test]
fn reply_accepted_round_trips() {
    let source = "Accepted.{ 42 2026-09-03T17:46:20 }";
    let reply: Reply = round_trip(source);
    assert_eq!(reply, Reply::Accepted(42, "2026-09-03T17:46:20".to_owned()));
    assert_eq!(round_trip_text(&reply), source);
}

#[test]
fn reply_refused_round_trips() {
    let source = "Refused.{ \u{201C}no such file: { } is content\u{201D} 2 }";
    let reply: Reply = round_trip(source);
    assert_eq!(
        reply,
        Reply::Refused("no such file: { } is content".to_owned(), 2)
    );
    assert_eq!(round_trip_text(&reply), source);
}

#[test]
fn reply_pending_round_trips() {
    let reply: Reply = round_trip("Pending");
    assert_eq!(reply, Reply::Pending);
    assert_eq!(round_trip_text(&reply), "Pending");
}

#[test]
fn observed_locks_empty_typed_round_trip() {
    let response: Response = round_trip("Observed.Locks.[]");
    assert_eq!(response, Response::Observed(Observation::Locks(vec![])));
    assert_eq!(round_trip_text(&response), "Observed.Locks.[]");
}

#[test]
fn success_typed_round_trip() {
    let response: Response = round_trip("Success");
    assert_eq!(response, Response::Success);
    assert_eq!(round_trip_text(&response), "Success");
}

// ---------------------------------------------------------------------------
// Scalars
// ---------------------------------------------------------------------------

#[test]
fn integer_round_trips() {
    for (source, expected) in [("0", 0i64), ("42", 42), ("-42", -42)] {
        let v: i64 = round_trip(source);
        assert_eq!(v, expected);
        assert_eq!(round_trip_text(&v), source);
    }
}

#[test]
fn integer_minus_zero_rejected() {
    let d = "-0".to_owned().protosize().unwrap();
    let datom: Datom = d.conceive().unwrap();
    assert!(i64::incorporate_from(datom).is_err());
}

#[test]
fn integer_leading_zero_rejected() {
    let d = "01".to_owned().protosize().unwrap();
    let datom: Datom = d.conceive().unwrap();
    assert!(i64::incorporate_from(datom).is_err());
}

#[test]
fn integer_plus_rejected() {
    let d = "+1".to_owned().protosize().unwrap();
    let datom: Datom = d.conceive().unwrap();
    assert!(i64::incorporate_from(datom).is_err());
}

#[test]
#[allow(clippy::approx_constant)]
fn decimal_round_trips() {
    for (source, expected) in [("3.14", 3.14f64), ("-0.5", -0.5), ("0.0", 0.0)] {
        let v: f64 = round_trip(source);
        assert!((v - expected).abs() < f64::EPSILON);
    }
}

#[test]
fn decimal_point_mandatory() {
    let d = "42".to_owned().protosize().unwrap();
    let datom: Datom = d.conceive().unwrap();
    assert!(f64::incorporate_from(datom).is_err());
}

#[test]
fn decimal_no_leading_zero_except_zero_dot() {
    let d = "01.5".to_owned().protosize().unwrap();
    let datom: Datom = d.conceive().unwrap();
    assert!(f64::incorporate_from(datom).is_err());
}

#[test]
#[allow(clippy::approx_constant)]
fn decimal_shortest_round_trip() {
    assert_eq!(round_trip_text(&3.14f64), "3.14");
    assert_eq!(round_trip_text(&0.0f64), "0.0");
    assert_eq!(round_trip_text(&1.0f64), "1.0");
}

#[test]
fn boolean_round_trips() {
    assert!(round_trip::<bool>("True"));
    assert!(!round_trip::<bool>("False"));
    assert_eq!(round_trip_text(&true), "True");
    assert_eq!(round_trip_text(&false), "False");
}

#[test]
fn text_bare_round_trips() {
    let v: String = round_trip("Ada");
    assert_eq!(v, "Ada");
}

#[test]
fn text_quoted_round_trips() {
    let v: String = round_trip("\u{201C}hello world\u{201D}");
    assert_eq!(v, "hello world");
}

#[test]
fn bare_word_with_separator_as_string() {
    let v: String = round_trip("name:first");
    assert_eq!(v, "name:first");
}

#[test]
fn meaning_round_trips() {
    let source = "(The build passed)";
    let d = source.to_owned().protosize().unwrap();
    let datom: Datom = d.conceive().unwrap();
    let m = Meaning::incorporate_from(datom).unwrap();
    assert_eq!(m, Meaning::Plain("The build passed".to_owned()));
}

// ---------------------------------------------------------------------------
// Containers
// ---------------------------------------------------------------------------

#[test]
fn vector_integer_round_trips() {
    let v: Vec<i64> = round_trip("[ 0 42 -42 ]");
    assert_eq!(v, vec![0, 42, -42]);
    assert_eq!(round_trip_text(&v), "[ 0 42 -42 ]");
}

#[test]
fn option_some_round_trips() {
    let v: Option<i64> = round_trip("Some.42");
    assert_eq!(v, Some(42));
    assert_eq!(round_trip_text(&v), "Some.42");
}

#[test]
fn option_none_round_trips() {
    let v: Option<i64> = round_trip("None");
    assert_eq!(v, None);
    assert_eq!(round_trip_text(&v), "None");
}

#[test]
fn result_ok_round_trips() {
    let v: Result<i64, String> = round_trip("Ok.42");
    assert_eq!(v, Ok(42));
}

#[test]
fn result_err_round_trips() {
    let v: Result<i64, String> = round_trip("Err.\u{201C}something went wrong\u{201D}");
    assert_eq!(v, Err("something went wrong".to_owned()));
}

// ---------------------------------------------------------------------------
// Fault path propagation
// ---------------------------------------------------------------------------

#[test]
fn vector_fault_carries_index_path() {
    let d = "[ 1 x ]".to_owned().protosize().unwrap();
    let datom: Datom = d.conceive().unwrap();
    let err = Vec::<i64>::incorporate_from(datom).unwrap_err();
    match &err {
        Fault::Corporate(path, Problem::Value(_)) => {
            assert_eq!(path, &vec![1]);
        }
        other => panic!("expected Corporate Value fault, got {other:?}"),
    }
}

#[test]
fn struct_fault_carries_field_path() {
    let d = "{ 1 x }".to_owned().protosize().unwrap();
    let datom: Datom = d.conceive().unwrap();
    let err = Extent::incorporate_from(datom).unwrap_err();
    match &err {
        Fault::Corporate(path, Problem::Value(_)) => {
            assert_eq!(path, &vec![1]);
        }
        other => panic!("expected Corporate Value fault at [1], got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Situation computed on ascent
// ---------------------------------------------------------------------------

#[test]
fn protosizable_datom_computes_situation() {
    let datom = Datom::Struct(vec![
        Datom::Bare("42".to_owned()),
        Datom::Bare("hello".to_owned()),
    ]);
    let d = datom.protosize().unwrap();
    assert!(!d.situation.is_empty(), "situation should not be empty");
    use protos::Situating as _;
    assert!(d.situate(&[0]).is_some());
    assert!(d.situate(&[0, 0]).is_some());
    assert!(d.situate(&[0, 1]).is_some());
}

// ---------------------------------------------------------------------------
// Self-describing fault types
// ---------------------------------------------------------------------------

#[test]
fn fault_round_trips_through_datom() {
    let fault = Fault::Corporate(vec![1, 2], Problem::Value("bad".to_owned()));
    let datom = fault.conceive().unwrap();
    let text = Textualizable::textualize(&datom);
    let d = text.protosize().unwrap();
    let datom2: Datom = d.conceive().unwrap();
    let fault2 = Fault::incorporate_from(datom2).unwrap();
    assert_eq!(fault, fault2);
}
