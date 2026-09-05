//! The reading rules as datom sees them: bare words in text positions, scalars,
//! escapes, and the refusal of the closing curly quote.

mod common;

use common::*;
use datom_codec::{
    Actualizable, Conceivable, Datom, Datomic, Decimal, Expected, Found, Meaning, Opaque,
    Potential, Problem, Protosizable, Refusal, Situated, Text, Textualizable,
};
use proptest::prelude::*;

fn read<T: Datomic>(text: &str) -> Result<T, datom_codec::Fault> {
    Potential::<T>::from(text).actualize()
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
        ".a",
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
        assert_eq!(t.textualize(), word, "{word:?} writes bare");
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
        assert!(
            value_fault(&read::<Decimal>(t).unwrap_err()),
            "{t:?} is not a decimal"
        );
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
    let Situated(at, datom) = text.protosize().unwrap().conceive().unwrap();
    assert_eq!(
        datom,
        Datom::Struct(vec![
            Datom::Word("Ada".to_owned()),
            Datom::Vector(vec![
                Datom::Word("12".to_owned()),
                Datom::Word("7".to_owned()),
                Datom::Word("-3".to_owned())
            ])
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
fn a_decimal_projection_has_the_anatomy_of_its_written_text() {
    let value = Decimal::try_from(3.25).unwrap();
    let projected = value.conceive().protosize().unwrap();
    let read = value.textualize().protosize().unwrap();
    assert_eq!(projected, read);
}

#[test]
fn non_period_chains_are_words_and_period_chains_are_variants() {
    let text = "{ a:b Some.42 }";
    let Situated(_, datom) = text.protosize().unwrap().conceive().unwrap();
    assert_eq!(
        datom,
        Datom::Struct(vec![
            Datom::Word("a:b".to_owned()),
            Datom::Variant("Some".to_owned(), Box::new(Datom::Word("42".to_owned())))
        ])
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
    let value = datom_codec::Problem::Value("a;b".to_owned());
    let written = value.textualize();
    assert_eq!(read::<datom_codec::Problem>(&written).unwrap(), value);
}

#[test]
fn exhausted_positions_refuse_without_moving_the_cursor() {
    use datom_codec::{Counted, Positional, Sited};

    let datom = Datom::Vector(vec![]);
    let at = protos::Situation {
        extent: protos::Extent(0, 2),
        children: vec![],
    };
    let site = datom_codec::Site {
        datom: &datom,
        at: &at,
    };
    let mut positions = site.elements().unwrap();
    let exhausted: Result<i64, _> = positions.position();
    assert!(exhausted.is_err());
    assert_eq!(positions.remaining(), 0);
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
