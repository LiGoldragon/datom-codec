//! Every example of Vision/datom.md, typed, verbatim, through actualize and back.

mod common;

use common::*;
use datom_codec::{Actualizable, Datomic, Decimal, Potential};

fn round_trip<T: Datomic + PartialEq + std::fmt::Debug>(text: &str, value: T) {
    let potential = Potential::<T>::from(text);
    let read: T = potential.actualize().unwrap();
    assert_eq!(read, value);
    assert_eq!(value.textualize(), text);
}

#[test]
fn person_with_roles() {
    round_trip(
        "{ Ada 1990 { “12 Rue de la Paix” Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }",
        Person(
            text("Ada"),
            1990,
            Address(text("12 Rue de la Paix"), text("Paris"), text("75002")),
            vec![Role::Author, Role::Reviewer(2024, 17)],
        ),
    );
}

#[test]
fn reply_accepted() {
    round_trip(
        "Accepted.{ 42 2026-09-03T17:46:20 }",
        Reply::Accepted(42, text("2026-09-03T17:46:20")),
    );
}

#[test]
fn reply_refused() {
    round_trip(
        "Refused.{ “no such file: { } is content” 2 }",
        Reply::Refused(text("no such file: { } is content"), 2),
    );
}

#[test]
fn reply_pending() {
    round_trip("Pending", Reply::Pending);
}

#[test]
fn scores() {
    round_trip("{ Ada [ 12 7 -3 ] }", Scores(text("Ada"), vec![12, 7, -3]));
}

#[test]
fn integer_vector() {
    round_trip("[ 0 42 -42 ]", vec![0i64, 42, -42]);
}

#[test]
fn note() {
    round_trip(
        "{ Ada (The build passed on the third try (after two timeouts)) }",
        Note(
            text("Ada"),
            meaning("The build passed on the third try (after two timeouts)"),
        ),
    );
}

#[test]
fn remark() {
    round_trip(
        "{ Ada “The build passed on the third try (after two timeouts)” }",
        Remark(
            text("Ada"),
            text("The build passed on the third try (after two timeouts)"),
        ),
    );
}

#[test]
fn standup() {
    round_trip(
        "{ Backend [ (Ada fixed the flaky test (the one with the timeout)) (Bo is out (back Monday)) ] }",
        Standup(
            text("Backend"),
            vec![
                meaning("Ada fixed the flaky test (the one with the timeout)"),
                meaning("Bo is out (back Monday)"),
            ],
        ),
    );
}

#[test]
fn observed_locks_empty() {
    round_trip(
        "Observed.Locks.[]",
        Response::Observed(Observation::Locks(vec![])),
    );
}

#[test]
fn success() {
    round_trip("Success", Response::Success);
}

#[test]
fn lock() {
    round_trip(
        "Lock.{ MyLock 6329f1 [ /abs/path ] “why I hold it” }",
        Request::Lock(LockRequest(
            text("MyLock"),
            text("6329f1"),
            vec![text("/abs/path")],
            text("why I hold it"),
        )),
    );
    round_trip("Release.442", Request::Release(442));
}

#[test]
fn intrinsics_actualize_alone() {
    round_trip("42", 42i64);
    round_trip("-0.5", Decimal::try_from(-0.5).unwrap());
    round_trip("True", true);
    round_trip("Ada", text("Ada"));
    round_trip("“a b”", text("a b"));
    round_trip("(a (b))", meaning("a (b)"));
    round_trip("Some.42", Some(42i64));
    round_trip("None", None::<i64>);
    round_trip("Ok.1", Ok::<i64, bool>(1));
    round_trip("Err.False", Err::<i64, bool>(false));
    round_trip("Reviewer.{ 1 2 }", Box::new(Role::Reviewer(1, 2)));
}

#[test]
fn the_potential_is_the_text() {
    use datom_codec::Texted;
    let potential = Potential::<i64>::from("42");
    assert_eq!(potential.text(), "42");
}
