#![allow(dead_code, non_camel_case_types, non_snake_case)]
#[rustfmt::skip]
pub trait DatomForming {
    fn datom_form(
        &self,
        input: crate::Path,
    ) -> std::result::Result<crate::Datom, crate::Error>;
}
#[rustfmt::skip]
pub trait Datomizable {
    fn datomize(&self, input: crate::Path) -> crate::Datom;
}
#[rustfmt::skip]
pub trait Composing {
    fn compose(
        input_0: crate::Datom,
        input_1: crate::Budget,
    ) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
#[rustfmt::skip]
pub trait Compositional: Composing {
    const ARITY: i64;
    fn from_positions(input: crate::Positions) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
#[rustfmt::skip]
pub trait Composable {
    fn compose(&self, input: crate::Budget) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
    fn compose_positions(
        &self,
        input: crate::Budget,
    ) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
#[rustfmt::skip]
pub trait Actualizing {
    fn actualize(
        &mut self,
        input: crate::Budget,
    ) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
