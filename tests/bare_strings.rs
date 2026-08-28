use datomic::{Datomic, Text, TextEdge};

#[test]
fn separator_bearing_strings_are_bare_at_the_typed_edge() {
    for source in ["a.b", "a!b", "a:b"] {
        let value = Text::<String>::from(source)
            .embody()
            .expect("one Portion is an expected String");
        assert_eq!(value, source);
        assert_eq!(value.textualize().as_ref(), source);
    }
}

#[test]
fn spaces_use_an_opaque_carrier_while_utf8_remains_bare() {
    let spaced = "two words".to_owned();
    assert_eq!(spaced.textualize().as_ref(), "“two words”");
    assert_eq!(
        Text::<String>::from("“two words”")
            .embody()
            .expect("opaque string embodies"),
        spaced
    );

    let utf8 = "héllo世界".to_owned();
    assert_eq!(utf8.textualize().as_ref(), "héllo世界");
    assert_eq!(
        Text::<String>::from("héllo世界")
            .embody()
            .expect("bare UTF-8 embodies"),
        utf8
    );
}

#[test]
fn balanced_curly_content_is_opaque_and_the_d4_edge_round_trips_it() {
    let value = "outer “inner” tail".to_owned();
    let text = value.textualize();
    assert_eq!(text.as_ref(), "“outer “inner” tail”");
    assert_eq!(text.embody().expect("canonical typed text embodies"), value);
}
