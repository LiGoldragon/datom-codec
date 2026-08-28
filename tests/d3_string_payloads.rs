use datomic::{
    Datomic, DatomicString, Fault, FaultProblem, PortionBuilding, PortionViewing, Text, TextEdge,
};
use protos::{Portion, Separator};

enum Note {
    Text(DatomicString),
}

impl Datomic for Note {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(headed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        if headed.head.as_ref() != "Note" || headed.separator != Separator::Period {
            return Err(portion.fault(FaultProblem::Head));
        }
        DatomicString::embody(&headed.body).map(Self::Text)
    }

    fn portion(&self) -> Portion {
        match self {
            Self::Text(value) => "Note".headed(Separator::Period, value.portion()),
        }
    }
}

#[test]
fn enum_string_payload_angle_expression_and_balanced_parentheses_use_datomic_string() {
    let note = Text::<Note>::from("Note.“sub note”")
        .embody()
        .expect("enum string payload embodies");
    assert_eq!(note.textualize().as_ref(), "Note.“sub note”");

    let angle = Text::<DatomicString>::from("<Kind>")
        .embody()
        .expect("angle expression embodies where String is expected");
    assert_eq!(angle.as_ref(), "<Kind>");
    assert_eq!(angle.textualize().as_ref(), "<Kind>");

    let parenthetical = Text::<DatomicString>::from("(outer (nested) tail)")
        .embody()
        .expect("balanced parenthetical String embodies");
    assert_eq!(parenthetical.as_ref(), "outer (nested) tail");
    assert_eq!(parenthetical.textualize().as_ref(), "“outer (nested) tail”");
}
