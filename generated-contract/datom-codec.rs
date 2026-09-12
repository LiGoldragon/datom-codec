#![allow(dead_code, non_camel_case_types, non_snake_case)]
#[rustfmt::skip]
pub type Path = std::vec::Vec<i64>;
#[rustfmt::skip]
pub type Opaque = String;
#[rustfmt::skip]
pub type Meaning = String;
#[rustfmt::skip]
#[derive(datom_codec::Datomizable, datom_codec::Compositional, Clone, Debug, PartialEq)]
pub struct FormVariant {
    pub symbol: protos::Symbol,
    pub datom: std::boxed::Box<Datom>,
}
#[rustfmt::skip]
#[derive(datom_codec::Datomizable, datom_codec::Compositional, Clone, Debug, PartialEq)]
pub struct Datom {
    pub path: Path,
    pub form: std::boxed::Box<Form>,
}
#[rustfmt::skip]
#[derive(datom_codec::Datomizable, datom_codec::Compositional, Clone, Debug, PartialEq)]
pub enum Form {
    Struct(std::vec::Vec<Datom>),
    Vector(std::vec::Vec<Datom>),
    Variant(std::boxed::Box<FormVariant>),
    Bare(String),
    String(String),
    Meaning(Opaque),
}
#[rustfmt::skip]
#[derive(datom_codec::Datomizable, datom_codec::Compositional, Clone, Debug, PartialEq)]
pub struct Budget {
    pub first_integer: i64,
    pub reader_budget: protos::ReaderBudget,
    pub second_integer: i64,
    pub third_integer: i64,
}
#[rustfmt::skip]
pub trait Datomizable {
    type Output;
    fn datomize(&self, input: Path) -> Self::Output;
}
#[rustfmt::skip]
pub trait Compositional {}
#[rustfmt::skip]
pub trait Composable {}
