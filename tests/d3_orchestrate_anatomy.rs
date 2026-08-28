use datomic::{Datomic, Fault, FaultProblem, PortionBuilding, PortionViewing, Text, TextEdge};
use protos::{Portion, Separator, StructuralEnclosure};

struct LockRequest {
    name: String,
    flow: String,
    paths: Vec<String>,
    reason: String,
}

enum Operation {
    Lock(LockRequest),
    Release(i64),
    Observe,
}

impl Datomic for LockRequest {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(fields) = portion.structural(StructuralEnclosure::Braced) else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        let [name, flow, paths, reason] = fields else {
            return Err(portion.fault(FaultProblem::Arity));
        };
        Ok(Self {
            name: String::embody(name)?,
            flow: String::embody(flow)?,
            paths: Vec::<String>::embody(paths)?,
            reason: String::embody(reason)?,
        })
    }

    fn portion(&self) -> Portion {
        "".structural(
            StructuralEnclosure::Braced,
            vec![
                self.name.portion(),
                self.flow.portion(),
                self.paths.portion(),
                self.reason.portion(),
            ],
        )
    }
}

impl Datomic for Operation {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(headed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        if headed.separator != Separator::Period {
            return Err(portion.fault(FaultProblem::Head));
        }
        match headed.head.as_ref() {
            "Lock" => LockRequest::embody(&headed.body).map(Self::Lock),
            "Release" => i64::embody(&headed.body).map(Self::Release),
            "Observe" if headed.body.bare_symbol() == Some("Locks") => Ok(Self::Observe),
            _ => Err(portion.fault(FaultProblem::Head)),
        }
    }

    fn portion(&self) -> Portion {
        match self {
            Self::Lock(request) => "Lock".headed(Separator::Period, request.portion()),
            Self::Release(identifier) => "Release".headed(Separator::Period, identifier.portion()),
            Self::Observe => "Observe".headed(Separator::Period, "Locks".bare()),
        }
    }
}

#[test]
fn approved_orchestrate_operations_use_one_declarative_anatomy_pattern() {
    let lock =
        Text::<Operation>::from("Lock.{datomicD0D4 root/realize_datomic [/a /b] “one line”}")
            .embody()
            .expect("Lock request embodies");
    let Operation::Lock(lock) = lock else {
        panic!("Lock selects LockRequest");
    };
    assert_eq!(lock.name, "datomicD0D4");
    assert_eq!(lock.flow, "root/realize_datomic");
    assert_eq!(lock.paths, vec!["/a", "/b"]);
    assert_eq!(lock.reason, "one line");
    assert_eq!(
        lock.textualize().as_ref(),
        "{datomicD0D4 root/realize_datomic [/a /b] “one line”}"
    );

    let release = Text::<Operation>::from("Release.80")
        .embody()
        .expect("Release embodies");
    assert!(matches!(release, Operation::Release(80)));
    assert_eq!(release.textualize().as_ref(), "Release.80");

    let observe = Text::<Operation>::from("Observe.Locks")
        .embody()
        .expect("Observe embodies");
    assert!(matches!(observe, Operation::Observe));
    assert_eq!(observe.textualize().as_ref(), "Observe.Locks");
}
