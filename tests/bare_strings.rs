use datomic::{Datomic, DatomicString, Text, TextEdge};

#[test]
fn separator_bearing_strings_are_bare_at_the_typed_edge() {
    for source in ["a.b", "a!b", "a:b"] {
        let value = Text::<DatomicString>::from(source)
            .embody()
            .expect("one Portion is an expected String");
        assert_eq!(value.as_ref(), source);
        assert_eq!(value.textualize().as_ref(), source);
    }
}

#[test]
fn spaces_use_an_opaque_carrier_while_utf8_remains_bare() {
    let spaced = DatomicString::try_from("two words".to_owned()).expect("representable string");
    assert_eq!(spaced.textualize().as_ref(), "“two words”");
    assert_eq!(
        Text::<DatomicString>::from("“two words”")
            .embody()
            .expect("opaque string embodies"),
        spaced
    );

    let utf8 = DatomicString::try_from("héllo世界".to_owned()).expect("representable string");
    assert_eq!(utf8.textualize().as_ref(), "héllo世界");
    assert_eq!(
        Text::<DatomicString>::from("héllo世界")
            .embody()
            .expect("bare UTF-8 embodies"),
        utf8
    );
}

#[test]
fn balanced_curly_content_is_opaque_and_the_d4_edge_round_trips_it() {
    let value = DatomicString::try_from("outer “inner” tail".to_owned())
        .expect("balanced curly content is representable");
    let text = value.textualize();
    assert_eq!(text.as_ref(), "“outer “inner” tail”");
    assert_eq!(text.embody().expect("canonical typed text embodies"), value);
}

#[test]
fn unbalanced_curly_content_is_rejected_before_outbound_projection() {
    assert!(DatomicString::try_from("unbalanced “".to_owned()).is_err());
    assert!(DatomicString::try_from("unbalanced ”".to_owned()).is_err());
}
