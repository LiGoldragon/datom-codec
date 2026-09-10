#![allow(dead_code, non_camel_case_types, non_snake_case)]
pub type Path = std::vec::Vec<i64>;
pub type Opaque = String;
pub type Meaning = String;
#[derive(datom_codec::Datomizable, datom_codec::Compositional)]
pub struct FormVariant {
    pub symbol: protos::Symbol,
    pub datom: std::boxed::Box<Datom>,
}
#[derive(datom_codec::Datomizable, datom_codec::Compositional)]
pub struct Datom {
    pub path: Path,
    pub form: std::boxed::Box<Form>,
}
#[derive(datom_codec::Datomizable, datom_codec::Compositional)]
pub enum Form {
    Struct(std::vec::Vec<Datom>),
    Vector(std::vec::Vec<Datom>),
    Variant(std::boxed::Box<FormVariant>),
    Bare(String),
    String(String),
    Meaning(Opaque),
}
#[derive(datom_codec::Datomizable, datom_codec::Compositional)]
pub struct Budget {
    pub first_integer: i64,
    pub reader_budget: protos::ReaderBudget,
    pub second_integer: i64,
    pub third_integer: i64,
}
pub trait Datomizable {
    type Output;
    fn datomize(&self, input: Path) -> Self::Output;
}
pub trait Compositional {
    type ARITY;
}
pub trait Composable {}
