#![allow(dead_code, non_camel_case_types, non_snake_case)]
pub trait DatomForming {
    fn datom_form(
        &self,
        input: crate::Path,
    ) -> std::result::Result<crate::Datom, crate::Error>;
}
pub trait Datomizable {
    type Output;
    fn datomize(&self, input: crate::Path) -> Self::Output;
}
pub trait Compositional {
    fn compose(
        input_0: crate::Datom,
        input_1: crate::Budget,
    ) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
pub trait Composable {
    fn compose(&self, input: crate::Budget) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
pub trait Actualizing {
    fn actualize(
        &mut self,
        input: crate::Budget,
    ) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
