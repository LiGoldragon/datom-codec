//! The reading rules as datom sees them: bare words in text positions, scalars,
//! escapes, and the refusal of the closing curly quote.

mod common;

use common::*;
use datom_codec::{
    Actualizable, Conceivable, Datom, DatomWord, Datomic, Decimal, Expected, Found, Meaning,
    Opaque, Potential, Problem, Protosizable, Refusal, Situated, Text, Textualizable, WordRefusal,
};
use proptest::prelude::*;
use protos::{Symbol, Word};

fn word(text: &str) -> Datom {
    match DatomWord::try_from(text) {
        Ok(word) => Datom::Word(word),
        Err(WordRefusal::Period(raw)) => {
            let text = raw.as_ref();
            let (head, body) = text.split_once('.').unwrap();
            variant(head, word(body))
        }
        Err(WordRefusal::Unstable(raw)) => Datom::Text(Text::try_from(raw.as_ref()).unwrap()),
        Err(WordRefusal::Bare(refusal)) => panic!("invalid test word: {refusal:?}"),
    }
}

fn variant(name: &str, body: Datom) -> Datom {
    Datom::Variant(Symbol::try_from(name).unwrap(), Box::new(body))
}

fn read<T: Datomic>(text: &str) -> Result<T, datom_codec::Fault> {
    Potential::<T>::from(text).actualize(budget())
}
fn value_fault(f: &datom_codec::Fault) -> bool {
    matches!(f, datom_codec::Fault::Corporate(_, Problem::Value(_)))
}

#[test]
fn bare_words_in_a_text_position_keep_their_syntax_glyphs() {
    for word in [
        "name:first",
        "Some.42",
        "a..b",
        "a.",
        "2026-09-03T17:46:20",
        "a:b.c",
        "a.b:c",
        "-",
        "a\\b",
        "True",
        "42",
    ] {
        let t: Text = read(word).unwrap();
        assert_eq!(t.as_ref(), word, "{word:?} reads whole");
        assert_eq!(
            read::<Text>(&t.textualize()).unwrap(),
            t,
            "{word:?} round-trips"
        );
    }
}

#[test]
fn text_is_quoted_when_it_must_be() {
    for (content, written) in [
        ("a b", "“a b”"),
        ("", "“”"),
        ("a;b", "“a;b”"),
        ("{", "“{”"),
        ("a“b", "“a“b”"),
        ("(x)", "“(x)”"),
        ("Vector<Text>", "“Vector<Text>”"),
        ("tab\there", "“tab\there”"),
    ] {
        let t = text(content);
        assert_eq!(t.textualize(), written);
        assert_eq!(read::<Text>(written).unwrap(), t);
    }
}

#[test]
fn text_refuses_the_closing_curly_quote() {
    assert_eq!(
        Text::try_from("a”"),
        Err(Refusal {
            glyph: '”',
            offset: 1
        })
    );
    assert!(matches!(
        read::<Text>("“a” ”").unwrap_err(),
        datom_codec::Fault::Structural(protos::Fault {
            problem: protos::Problem::Stray(protos::Boundary::CurlyQuotes),
            ..
        })
    ));
}

#[test]
fn integers() {
    for (t, v) in [
        ("0", 0i64),
        ("42", 42),
        ("-42", -42),
        ("-9223372036854775808", i64::MIN),
        ("9223372036854775807", i64::MAX),
    ] {
        assert_eq!(read::<i64>(t).unwrap(), v);
        assert_eq!(v.textualize(), t);
    }
    for t in [
        "+1",
        "01",
        "-0",
        "1.0",
        "1e3",
        "9223372036854775808",
        "0x1",
        "1_000",
    ] {
        assert!(
            value_fault(&read::<i64>(t).unwrap_err()),
            "{t:?} is not an integer"
        );
    }
    assert!(read::<i64>("").is_err());
    assert!(read::<i64>("- 1").is_err());
}

#[test]
fn decimals() {
    for (t, v) in [
        ("3.25", 3.25f64),
        ("-0.5", -0.5),
        ("0.0", 0.0),
        ("1.5", 1.5),
        ("100.0", 100.0),
    ] {
        let v = Decimal::try_from(v).unwrap();
        assert_eq!(read::<Decimal>(t).unwrap(), v);
        assert_eq!(v.textualize(), t);
    }
    assert_eq!(Decimal::try_from(1.50).unwrap().textualize(), "1.5");
    for t in ["1", "1.", ".5", "1e300", "01.5", "NaN", "inf", "-", "1.5.2"] {
        assert!(read::<Decimal>(t).is_err(), "{t:?} is not a decimal");
    }
    assert!(Decimal::try_from(f64::NAN).is_err());
    assert!(Decimal::try_from(f64::INFINITY).is_err());
}

#[test]
fn booleans() {
    assert!(read::<bool>("True").unwrap());
    assert!(!read::<bool>("False").unwrap());
    assert_eq!(true.textualize(), "True");
    for t in ["true", "false", "1", "T"] {
        assert!(value_fault(&read::<bool>(t).unwrap_err()));
    }
}

#[test]
fn a_string_position_takes_bare_quoted_and_chains_only() {
    assert_eq!(
        read::<Text>("(x)").unwrap_err(),
        datom_codec::Fault::Corporate(
            datom_codec::Locus {
                path: vec![],
                extent: datom_codec::Extent(0, 3)
            },
            Problem::Shape(Expected::Text, Found::Meaning)
        )
    );
    assert!(matches!(
        read::<Text>("{ a }").unwrap_err(),
        datom_codec::Fault::Corporate(_, Problem::Shape(Expected::Text, Found::Struct))
    ));
    assert!(matches!(
        read::<Text>("Some.{ 1 }").unwrap_err(),
        datom_codec::Fault::Corporate(_, Problem::Shape(Expected::Text, Found::Variant))
    ));
}

#[test]
fn meaning_escapes() {
    for (content, written) in [
        ("a (b) c", "(a (b) c)"),
        ("a ) b", "(a \\) b)"),
        ("a ( b", "(a \\( b)"),
        ("\\", "(\\\\)"),
        ("", "()"),
        ("a “ b ; c", "(a “ b ; c)"),
    ] {
        let m = meaning(content);
        assert_eq!(m.textualize(), written);
        assert_eq!(read::<Meaning>(written).unwrap(), m);
    }
}

#[test]
fn meaning_keeps_a_curly_quote_closer_as_its_own_content() {
    let value = read::<Meaning>("(a ” b)").unwrap();
    let Meaning::Plain(content) = &value;
    assert_eq!(content.as_ref(), "a ” b");
    assert_eq!(value.textualize(), "(a ” b)");
}

#[test]
fn the_concept_is_situated_by_the_reader_and_by_the_writer() {
    use datom_codec::Locating;
    let text = "{ Ada [ 12 7 -3 ] }";
    let Situated(at, datom): Situated<Datom> = text.protosize().unwrap().conceive().unwrap();
    assert_eq!(
        datom,
        Datom::Struct(vec![
            word("Ada"),
            Datom::Vector(vec![word("12"), word("7"), word("-3")])
        ])
    );
    assert_eq!(at.locate(&[1, 2]), Some(datom_codec::Extent(13, 15)));
    let Ok(written) = datom.protosize();
    assert_eq!(
        written.0[0].0,
        "{ Ada [ 12 7 -3 ] }".protosize().unwrap().0[0].0
    );
    assert_eq!(datom.textualize(), text);
}

#[test]
fn words_admit_only_one_canonical_datom_anatomy() {
    for text in ["a.b", "a.b:c", "a.b.c"] {
        let period = Word::try_from(text).unwrap();
        assert_eq!(
            DatomWord::try_from(period.clone()),
            Err(WordRefusal::Period(period)),
            "{text:?}"
        );
    }
    for text in ["a..b", ".a", "a."] {
        let unstable = Word::try_from(text).unwrap();
        assert_eq!(
            DatomWord::try_from(unstable.clone()),
            Err(WordRefusal::Unstable(unstable))
        );
    }
    for text in ["a:b", "a!b", "a:b.c", "a!b.c", "a..b", ".a", "a."] {
        let datom = variant("Some", word(text));
        let projected = datom.protosize().unwrap();
        let written =
            <datom_codec::Delineation as Textualizable<datom_codec::Delineation>>::textualize(
                &projected,
            );
        let reparsed = written.protosize().unwrap();
        assert_eq!(reparsed, projected, "{text:?}");
        let back: Datom = reparsed.conceive().unwrap().1;
        assert_eq!(back, datom, "{text:?}");
    }
    for value in [3.25, -42.0, 0.5] {
        let decimal = Decimal::try_from(value).unwrap().conceive().unwrap().1;
        let projected = decimal.protosize().unwrap();
        let written =
            <datom_codec::Delineation as Textualizable<datom_codec::Delineation>>::textualize(
                &projected,
            );
        let reparsed = written.protosize().unwrap();
        assert_eq!(reparsed, projected, "{value}");
        let back: Datom = reparsed.conceive().unwrap().1;
        assert_eq!(back, decimal, "{value}");
    }
}

#[test]
fn a_decimal_projection_has_the_anatomy_of_its_written_text() {
    let value = Decimal::try_from(3.25).unwrap();
    let projected = value.conceive().unwrap().1.protosize().unwrap();
    let read = value.textualize().protosize().unwrap();
    assert_eq!(projected, read);
}

#[test]
fn non_period_chains_are_words_and_period_chains_are_variants() {
    let text = "{ a:b Some.42 }";
    let Situated(_, datom): Situated<Datom> = text.protosize().unwrap().conceive().unwrap();
    assert_eq!(
        datom,
        Datom::Struct(vec![word("a:b"), variant("Some", word("42"))])
    );
}

#[test]
fn vectors_of_text_take_bare_and_quoted_alike() {
    let v: Vec<Text> = read("[ /abs/path “a b” c ]").unwrap();
    assert_eq!(v, vec![text("/abs/path"), text("a b"), text("c")]);
    assert_eq!(v.textualize(), "[ /abs/path “a b” c ]");
    assert_eq!(Vec::<i64>::new().textualize(), "[]");
}

#[test]
fn text_payloads_preserve_the_variant_boundary() {
    let value = Some(text("."));
    let written = value.textualize();
    assert_eq!(written, "Some.“.”");
    assert_eq!(read::<Option<Text>>(&written).unwrap(), value);
}

#[test]
fn fault_text_payloads_remain_data() {
    let value = datom_codec::Problem::Value(Opaque::from("a;b"));
    let written = value.textualize();
    assert_eq!(read::<datom_codec::Problem>(&written).unwrap(), value);
}

#[test]
fn exhausted_positions_refuse_without_moving_the_cursor() {
    use std::convert::Infallible;

    use datom_codec::{Conceivable, Extent, Positional, Sited, Situated, Situation};

    #[derive(Debug)]
    struct RequiresOne;

    impl Conceivable<Datom> for RequiresOne {
        type Fault = Infallible;

        fn conceive(&self) -> Result<Situated<Datom>, Self::Fault> {
            Ok(Situated(
                Situation {
                    extent: Extent(0, 0),
                    children: vec![],
                },
                Datom::Vector(vec![word("x")]),
            ))
        }
    }

    impl Datomic for RequiresOne {
        fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
            let mut positions = site.elements()?;
            let _: i64 = positions.position()?;
            Ok(Self)
        }
    }

    let fault = Potential::<RequiresOne>::from("[]")
        .actualize(budget())
        .unwrap_err();
    assert!(matches!(
        fault,
        datom_codec::Fault::Corporate(_, Problem::Exhausted)
    ));
}

proptest! {
    #[test]
    fn any_text_round_trips(content in "[^”]*") {
        let t = Text::try_from(content.as_str()).unwrap();
        prop_assert_eq!(read::<Text>(&t.textualize()).unwrap(), t);
    }

    #[test]
    fn any_meaning_round_trips(content in ".*") {
        let m = Meaning::Plain(Opaque::from(content));
        prop_assert_eq!(read::<Meaning>(&m.textualize()).unwrap(), m);
    }

    #[test]
    fn any_integer_round_trips(v in any::<i64>()) {
        prop_assert_eq!(read::<i64>(&v.textualize()).unwrap(), v);
    }

    #[test]
    fn any_finite_decimal_round_trips(v in any::<f64>().prop_filter("finite", |v| v.is_finite())) {
        let v = Decimal::try_from(v).unwrap();
        prop_assert_eq!(read::<Decimal>(&v.textualize()).unwrap(), v);
    }
}
