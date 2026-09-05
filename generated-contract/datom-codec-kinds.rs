#![allow(dead_code)]
pub trait Datomic: Sized + protos::Conceivable<
        crate::Datom,
    > + protos::Textualizable<crate::Datom> {
    fn incorporate(input: crate::Site) -> Result<Self, crate::Fault>;
}
pub trait Worded: Sized {
    const EXPECTED: crate::Expected;
    fn to_word(&self) -> std::string::String;
    fn incorporate_word(input: crate::Site) -> Result<Self, crate::Fault>;
    fn conceive_word(&self) -> crate::Datom;
}
pub trait Sited {
    fn positions(
        &self,
        input: protos::Integer,
    ) -> Result<crate::Positions, crate::Fault>;
    fn elements(&self) -> Result<crate::Positions, crate::Fault>;
    fn variant(&self) -> Result<crate::Variant, crate::Fault>;
    fn text(&self) -> Result<protos::Text, crate::Fault>;
    fn found(&self) -> crate::Found;
    fn refuse(&self, input: crate::Problem) -> crate::Fault;
}
pub trait Positional<A: Datomic> {
    fn position(&mut self) -> Result<A, crate::Fault>;
}
pub trait Counted {
    fn remaining(&self) -> protos::Integer;
}
pub trait Carrying<A: Datomic> {
    fn body(&self) -> Result<A, crate::Fault>;
}
pub trait Headed: Sized {
    fn positions(
        &self,
        input: protos::Integer,
    ) -> Result<crate::Positions, crate::Fault>;
    fn nothing(&self) -> Result<Self, crate::Fault>;
    fn reject(&self, input: crate::Problem) -> crate::Fault;
}
