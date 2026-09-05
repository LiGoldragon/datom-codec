use datom_codec::{Datom, DatomWord};
use protos::Symbol;

const DEPTH: usize = 20_000;

fn deep() -> Datom {
    let mut datom = Datom::Word(DatomWord::try_from("leaf").expect("leaf is datom word"));
    for _ in 0..DEPTH {
        datom = Datom::Variant(
            Symbol::try_from("Node").expect("Node is a symbol"),
            Box::new(datom),
        );
    }
    datom
}

#[test]
fn recursive_datom_traits_are_iterative() {
    let datom = deep();
    let copied = datom.clone();
    assert_eq!(datom, copied);
    assert!(format!("{datom:?}").starts_with("Variant(Symbol("));
}
