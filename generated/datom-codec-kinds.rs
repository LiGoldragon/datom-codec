#![allow(dead_code, non_camel_case_types, non_snake_case)]
#[rustfmt::skip]
pub trait Branchable {}
#[rustfmt::skip]
pub trait Budgeted {}
#[rustfmt::skip]
pub trait Positional {}
#[rustfmt::skip]
pub trait DatomForming {
    fn datom_form<N: Branchable>(
        &self,
        input: N,
    ) -> std::result::Result<crate::Datom, crate::Error>;
}
#[rustfmt::skip]
pub trait Datomizable {
    fn datomize<N: Branchable>(&self, input: N) -> crate::Datom;
}
#[rustfmt::skip]
pub trait Composing {
    fn compose<N: Composable, O: Budgeted>(
        input_0: N,
        input_1: O,
    ) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
#[rustfmt::skip]
pub trait Compositional: Composing {
    const ARITY: i64;
    fn from_positions<N: Positional>(input: N) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
#[rustfmt::skip]
pub trait Composable {
    fn compose<N: Budgeted>(&self, input: N) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
    fn compose_positions<N: Budgeted>(
        &self,
        input: N,
    ) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
#[rustfmt::skip]
pub trait Actualizing {
    fn actualize<N: Budgeted>(
        &mut self,
        input: N,
    ) -> std::result::Result<Self, crate::Error>
    where
        Self: Sized;
}
#[rustfmt::skip]
pub trait Represented {
    type Representation: Datomizable + Composing;
    fn represent(&self) -> Self::Representation;
    fn from_representation(
        input: Self::Representation,
    ) -> std::result::Result<Self, crate::ErrorKind>
    where
        Self: Sized;
}
#[rustfmt::skip]
const _: () = {
    fn assert_path_branchable<T: Branchable>() {}
    let _ = assert_path_branchable::<crate::Path>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_budget_budgeted<T: Budgeted>() {}
    let _ = assert_budget_budgeted::<crate::Budget>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_positions_positional<T: Positional>() {}
    let _ = assert_positions_positional::<crate::Positions>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_datom_composable<T: Composable>() {}
    let _ = assert_datom_composable::<crate::Datom>;
};
