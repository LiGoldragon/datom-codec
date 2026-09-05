#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Datom {
    Variant(protos::Symbol, Box<Datom>),
    Struct(Vec<Datom>),
    Vector(Vec<Datom>),
    Text(protos::Text),
    Meaning(protos::Opaque),
    Word(DatomWord),
}
impl datom_codec::Datomic for Datom {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Variant" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: protos::Symbol = datom_codec::Positional::position(&mut p)?;
                let p1: Box<Datom> = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Variant(p0, p1))
            }
            "Struct" => Ok(Self::Struct(datom_codec::Carrying::body(v)?)),
            "Vector" => Ok(Self::Vector(datom_codec::Carrying::body(v)?)),
            "Text" => Ok(Self::Text(datom_codec::Carrying::body(v)?)),
            "Meaning" => Ok(Self::Meaning(datom_codec::Carrying::body(v)?)),
            "Word" => Ok(Self::Word(datom_codec::Carrying::body(v)?)),
            _ => {
                Err(
                    datom_codec::Sited::refuse(
                        site,
                        datom_codec::Problem::UnknownVariant(v.name.to_owned()),
                    ),
                )
            }
        }
    }
    fn conceive(&self) -> datom_codec::Datom {
        match self {
            Self::Variant(p0, p1) => {
                datom_codec::Datom::Variant(
                    "Variant".to_owned(),
                    Box::new(
                        datom_codec::Datom::Struct(
                            vec![
                                datom_codec::Datomic::conceive(p0),
                                datom_codec::Datomic::conceive(p1)
                            ],
                        ),
                    ),
                )
            }
            Self::Struct(p0) => {
                datom_codec::Datom::Variant(
                    "Struct".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Vector(p0) => {
                datom_codec::Datom::Variant(
                    "Vector".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Text(p0) => {
                datom_codec::Datom::Variant(
                    "Text".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Meaning(p0) => {
                datom_codec::Datom::Variant(
                    "Meaning".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Word(p0) => {
                datom_codec::Datom::Variant(
                    "Word".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
    }
}
pub type DatomWord = protos::Word;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WordRefusal {
    Bare(protos::BareRefusal),
    Period(protos::Word),
}
impl datom_codec::Datomic for WordRefusal {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Bare" => Ok(Self::Bare(datom_codec::Carrying::body(v)?)),
            "Period" => Ok(Self::Period(datom_codec::Carrying::body(v)?)),
            _ => {
                Err(
                    datom_codec::Sited::refuse(
                        site,
                        datom_codec::Problem::UnknownVariant(v.name.to_owned()),
                    ),
                )
            }
        }
    }
    fn conceive(&self) -> datom_codec::Datom {
        match self {
            Self::Bare(p0) => {
                datom_codec::Datom::Variant(
                    "Bare".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Period(p0) => {
                datom_codec::Datom::Variant(
                    "Period".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Meaning {
    Plain(protos::Opaque),
}
impl datom_codec::Datomic for Meaning {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Plain" => Ok(Self::Plain(datom_codec::Carrying::body(v)?)),
            _ => {
                Err(
                    datom_codec::Sited::refuse(
                        site,
                        datom_codec::Problem::UnknownVariant(v.name.to_owned()),
                    ),
                )
            }
        }
    }
    fn conceive(&self) -> datom_codec::Datom {
        match self {
            Self::Plain(p0) => {
                datom_codec::Datom::Variant(
                    "Plain".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expected {
    Variant,
    Struct,
    Vector,
    Text,
    Meaning,
    Integer,
    Decimal,
    Boolean,
    Word,
}
impl datom_codec::Datomic for Expected {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Variant" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Variant)
            }
            "Struct" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Struct)
            }
            "Vector" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Vector)
            }
            "Text" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Text)
            }
            "Meaning" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Meaning)
            }
            "Integer" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Integer)
            }
            "Decimal" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Decimal)
            }
            "Boolean" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Boolean)
            }
            "Word" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Word)
            }
            _ => {
                Err(
                    datom_codec::Sited::refuse(
                        site,
                        datom_codec::Problem::UnknownVariant(v.name.to_owned()),
                    ),
                )
            }
        }
    }
    fn conceive(&self) -> datom_codec::Datom {
        match self {
            Self::Variant => datom_codec::Datom::Word("Variant".to_owned()),
            Self::Struct => datom_codec::Datom::Word("Struct".to_owned()),
            Self::Vector => datom_codec::Datom::Word("Vector".to_owned()),
            Self::Text => datom_codec::Datom::Word("Text".to_owned()),
            Self::Meaning => datom_codec::Datom::Word("Meaning".to_owned()),
            Self::Integer => datom_codec::Datom::Word("Integer".to_owned()),
            Self::Decimal => datom_codec::Datom::Word("Decimal".to_owned()),
            Self::Boolean => datom_codec::Datom::Word("Boolean".to_owned()),
            Self::Word => datom_codec::Datom::Word("Word".to_owned()),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Found {
    Variant,
    Struct,
    Vector,
    Text,
    Meaning,
    Word,
    Angled,
    Qualified,
    Chain,
}
impl datom_codec::Datomic for Found {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Variant" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Variant)
            }
            "Struct" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Struct)
            }
            "Vector" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Vector)
            }
            "Text" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Text)
            }
            "Meaning" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Meaning)
            }
            "Word" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Word)
            }
            "Angled" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Angled)
            }
            "Qualified" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Qualified)
            }
            "Chain" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Chain)
            }
            _ => {
                Err(
                    datom_codec::Sited::refuse(
                        site,
                        datom_codec::Problem::UnknownVariant(v.name.to_owned()),
                    ),
                )
            }
        }
    }
    fn conceive(&self) -> datom_codec::Datom {
        match self {
            Self::Variant => datom_codec::Datom::Word("Variant".to_owned()),
            Self::Struct => datom_codec::Datom::Word("Struct".to_owned()),
            Self::Vector => datom_codec::Datom::Word("Vector".to_owned()),
            Self::Text => datom_codec::Datom::Word("Text".to_owned()),
            Self::Meaning => datom_codec::Datom::Word("Meaning".to_owned()),
            Self::Word => datom_codec::Datom::Word("Word".to_owned()),
            Self::Angled => datom_codec::Datom::Word("Angled".to_owned()),
            Self::Qualified => datom_codec::Datom::Word("Qualified".to_owned()),
            Self::Chain => datom_codec::Datom::Word("Chain".to_owned()),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    Shape(Expected, Found),
    Arity(protos::Integer, protos::Integer),
    UnknownVariant(protos::Word),
    Value(protos::Opaque),
    Formless(Found),
    OneValue(protos::Integer),
    Exhausted,
    BudgetExhausted,
}
impl datom_codec::Datomic for Problem {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Shape" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: Expected = datom_codec::Positional::position(&mut p)?;
                let p1: Found = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Shape(p0, p1))
            }
            "Arity" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: protos::Integer = datom_codec::Positional::position(&mut p)?;
                let p1: protos::Integer = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Arity(p0, p1))
            }
            "UnknownVariant" => Ok(Self::UnknownVariant(datom_codec::Carrying::body(v)?)),
            "Value" => Ok(Self::Value(datom_codec::Carrying::body(v)?)),
            "Formless" => Ok(Self::Formless(datom_codec::Carrying::body(v)?)),
            "OneValue" => Ok(Self::OneValue(datom_codec::Carrying::body(v)?)),
            "Exhausted" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Exhausted)
            }
            "BudgetExhausted" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::BudgetExhausted)
            }
            _ => {
                Err(
                    datom_codec::Sited::refuse(
                        site,
                        datom_codec::Problem::UnknownVariant(v.name.to_owned()),
                    ),
                )
            }
        }
    }
    fn conceive(&self) -> datom_codec::Datom {
        match self {
            Self::Shape(p0, p1) => {
                datom_codec::Datom::Variant(
                    "Shape".to_owned(),
                    Box::new(
                        datom_codec::Datom::Struct(
                            vec![
                                datom_codec::Datomic::conceive(p0),
                                datom_codec::Datomic::conceive(p1)
                            ],
                        ),
                    ),
                )
            }
            Self::Arity(p0, p1) => {
                datom_codec::Datom::Variant(
                    "Arity".to_owned(),
                    Box::new(
                        datom_codec::Datom::Struct(
                            vec![
                                datom_codec::Datomic::conceive(p0),
                                datom_codec::Datomic::conceive(p1)
                            ],
                        ),
                    ),
                )
            }
            Self::UnknownVariant(p0) => {
                datom_codec::Datom::Variant(
                    "UnknownVariant".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Value(p0) => {
                datom_codec::Datom::Variant(
                    "Value".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Formless(p0) => {
                datom_codec::Datom::Variant(
                    "Formless".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::OneValue(p0) => {
                datom_codec::Datom::Variant(
                    "OneValue".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Exhausted => datom_codec::Datom::Word("Exhausted".to_owned()),
            Self::BudgetExhausted => {
                datom_codec::Datom::Word("BudgetExhausted".to_owned())
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locus(pub protos::Path, pub protos::Extent);
impl datom_codec::Datomic for Locus {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: protos::Path = datom_codec::Positional::position(&mut p)?;
        let p1: protos::Extent = datom_codec::Positional::position(&mut p)?;
        Ok(Self(p0, p1))
    }
    fn conceive(&self) -> datom_codec::Datom {
        datom_codec::Datom::Struct(
            vec![
                datom_codec::Datomic::conceive(& self.0),
                datom_codec::Datomic::conceive(& self.1)
            ],
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    Structural(protos::Fault),
    Conceptual(Locus, Problem),
    Corporate(Locus, Problem),
}
impl datom_codec::Datomic for Fault {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Structural" => Ok(Self::Structural(datom_codec::Carrying::body(v)?)),
            "Conceptual" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: Locus = datom_codec::Positional::position(&mut p)?;
                let p1: Problem = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Conceptual(p0, p1))
            }
            "Corporate" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: Locus = datom_codec::Positional::position(&mut p)?;
                let p1: Problem = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Corporate(p0, p1))
            }
            _ => {
                Err(
                    datom_codec::Sited::refuse(
                        site,
                        datom_codec::Problem::UnknownVariant(v.name.to_owned()),
                    ),
                )
            }
        }
    }
    fn conceive(&self) -> datom_codec::Datom {
        match self {
            Self::Structural(p0) => {
                datom_codec::Datom::Variant(
                    "Structural".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Conceptual(p0, p1) => {
                datom_codec::Datom::Variant(
                    "Conceptual".to_owned(),
                    Box::new(
                        datom_codec::Datom::Struct(
                            vec![
                                datom_codec::Datomic::conceive(p0),
                                datom_codec::Datomic::conceive(p1)
                            ],
                        ),
                    ),
                )
            }
            Self::Corporate(p0, p1) => {
                datom_codec::Datom::Variant(
                    "Corporate".to_owned(),
                    Box::new(
                        datom_codec::Datom::Struct(
                            vec![
                                datom_codec::Datomic::conceive(p0),
                                datom_codec::Datomic::conceive(p1)
                            ],
                        ),
                    ),
                )
            }
        }
    }
}
const _: () = {
    fn assert_datom_textualizable<T: protos::Textualizable>() {}
    let _ = assert_datom_textualizable::<Datom>;
    fn assert_datom_protosizable<T: protos::Protosizable>() {}
    let _ = assert_datom_protosizable::<Datom>;
};
const _: () = {
    fn assert_meaning_datomic<T: crate::Datomic>() {}
    let _ = assert_meaning_datomic::<Meaning>;
};
const _: () = {
    fn assert_expected_worded<T: crate::Worded>() {}
    let _ = assert_expected_worded::<Expected>;
    fn assert_expected_datomic<T: crate::Datomic>() {}
    let _ = assert_expected_datomic::<Expected>;
};
const _: () = {
    fn assert_found_worded<T: crate::Worded>() {}
    let _ = assert_found_worded::<Found>;
    fn assert_found_datomic<T: crate::Datomic>() {}
    let _ = assert_found_datomic::<Found>;
};
const _: () = {
    fn assert_problem_datomic<T: crate::Datomic>() {}
    let _ = assert_problem_datomic::<Problem>;
};
const _: () = {
    fn assert_locus_datomic<T: crate::Datomic>() {}
    let _ = assert_locus_datomic::<Locus>;
};
const _: () = {
    fn assert_fault_datomic<T: crate::Datomic>() {}
    let _ = assert_fault_datomic::<Fault>;
};
const _: () = {
    fn assert_integer_worded<T: crate::Worded>() {}
    let _ = assert_integer_worded::<protos::Integer>;
    fn assert_integer_datomic<T: crate::Datomic>() {}
    let _ = assert_integer_datomic::<protos::Integer>;
};
const _: () = {
    fn assert_decimal_worded<T: crate::Worded>() {}
    let _ = assert_decimal_worded::<protos::Decimal>;
    fn assert_decimal_datomic<T: crate::Datomic>() {}
    let _ = assert_decimal_datomic::<protos::Decimal>;
};
const _: () = {
    fn assert_boolean_worded<T: crate::Worded>() {}
    let _ = assert_boolean_worded::<protos::Boolean>;
    fn assert_boolean_datomic<T: crate::Datomic>() {}
    let _ = assert_boolean_datomic::<protos::Boolean>;
};
const _: () = {
    fn assert_text_datomic<T: crate::Datomic>() {}
    let _ = assert_text_datomic::<protos::Text>;
};
const _: () = {
    fn assert_extent_datomic<T: crate::Datomic>() {}
    let _ = assert_extent_datomic::<protos::Extent>;
};
const _: () = {
    fn assert_separator_worded<T: crate::Worded>() {}
    let _ = assert_separator_worded::<protos::Separator>;
    fn assert_separator_datomic<T: crate::Datomic>() {}
    let _ = assert_separator_datomic::<protos::Separator>;
};
const _: () = {
    fn assert_enclosure_worded<T: crate::Worded>() {}
    let _ = assert_enclosure_worded::<protos::Enclosure>;
    fn assert_enclosure_datomic<T: crate::Datomic>() {}
    let _ = assert_enclosure_datomic::<protos::Enclosure>;
};
const _: () = {
    fn assert_boundary_worded<T: crate::Worded>() {}
    let _ = assert_boundary_worded::<protos::Boundary>;
    fn assert_boundary_datomic<T: crate::Datomic>() {}
    let _ = assert_boundary_datomic::<protos::Boundary>;
};
const _: () = {
    fn assert_structuralproblem_datomic<T: crate::Datomic>() {}
    let _ = assert_structuralproblem_datomic::<protos::Problem>;
};
const _: () = {
    fn assert_structuralfault_datomic<T: crate::Datomic>() {}
    let _ = assert_structuralfault_datomic::<protos::Fault>;
};
