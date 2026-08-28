use datomic::{Datomic, Fault, FaultProblem, PortionBuilding, PortionViewing, Text, TextEdge};
use protos::{Portion, Separator, StructuralEnclosure};

struct Lock {
    identifier: i64,
    name: String,
    flow: String,
    paths: Vec<String>,
    reason: String,
}

enum Observe {
    Locks,
}

enum Request {
    Lock(Lock),
    Observe(Observe),
}

enum Reply {
    Observed(Vec<Lock>),
}

impl Datomic for Lock {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(fields) = portion.structural(StructuralEnclosure::Braced) else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        let [identifier, name, flow, paths, reason] = fields else {
            return Err(portion.fault(FaultProblem::Arity));
        };
        Ok(Self {
            identifier: i64::embody(identifier)?,
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
                self.identifier.portion(),
                self.name.portion(),
                self.flow.portion(),
                self.paths.portion(),
                self.reason.portion(),
            ],
        )
    }
}

impl Datomic for Observe {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(headed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        if headed.head.as_ref() != "Observe" || headed.separator != Separator::Period {
            return Err(portion.fault(FaultProblem::Head));
        }
        (headed.body.bare_symbol() == Some("Locks"))
            .then_some(Self::Locks)
            .ok_or_else(|| portion.fault(FaultProblem::Head))
    }

    fn portion(&self) -> Portion {
        match self {
            Self::Locks => "Observe".headed(Separator::Period, "Locks".bare()),
        }
    }
}

impl Datomic for Request {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(headed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        match headed.head.as_ref() {
            "Lock" if headed.separator == Separator::Period => {
                Lock::embody(&headed.body).map(Self::Lock)
            }
            "Observe" => Observe::embody(portion).map(Self::Observe),
            _ => Err(portion.fault(FaultProblem::Head)),
        }
    }

    fn portion(&self) -> Portion {
        match self {
            Self::Lock(lock) => "Lock".headed(Separator::Period, lock.portion()),
            Self::Observe(observe) => observe.portion(),
        }
    }
}

impl Datomic for Reply {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(observed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        let Some(locks) = observed.body.headed() else {
            return Err(portion.fault(FaultProblem::Head));
        };
        if observed.head.as_ref() != "Observed"
            || locks.head.as_ref() != "Locks"
            || observed.separator != Separator::Period
            || locks.separator != Separator::Period
        {
            return Err(portion.fault(FaultProblem::Head));
        }
        Vec::<Lock>::embody(&locks.body).map(Self::Observed)
    }

    fn portion(&self) -> Portion {
        match self {
            Self::Observed(locks) => "Observed".headed(
                Separator::Period,
                "Locks".headed(Separator::Period, locks.portion()),
            ),
        }
    }
}

#[test]
fn root_struct_enum_unit_and_payload_chains_round_trip() {
    let lock = Text::<Lock>::from("{9 lock flow [/a /b] “because it matters”}")
        .embody()
        .expect("root struct embodies");
    assert_eq!(lock.identifier, 9);
    assert_eq!(lock.name, "lock");
    assert_eq!(lock.flow, "flow");
    assert_eq!(lock.paths, vec!["/a", "/b"]);
    assert_eq!(lock.reason, "because it matters");
    assert_eq!(
        lock.textualize().as_ref(),
        "{9 lock flow [/a /b] “because it matters”}"
    );

    let observe = Text::<Request>::from("Observe.Locks")
        .embody()
        .expect("headed unit embodies");
    assert!(matches!(observe, Request::Observe(Observe::Locks)));
    assert_eq!(observe.textualize().as_ref(), "Observe.Locks");

    let observed = Text::<Reply>::from("Observed.Locks.[]")
        .embody()
        .expect("payload chain embodies");
    assert!(matches!(observed, Reply::Observed(ref locks) if locks.is_empty()));
    assert_eq!(observed.textualize().as_ref(), "Observed.Locks.[]");

    let request = Text::<Request>::from("Lock.{9 lock flow [/a /b] “because it matters”}")
        .embody()
        .expect("struct payload embodies");
    assert_eq!(
        request.textualize().as_ref(),
        "Lock.{9 lock flow [/a /b] “because it matters”}"
    );
}
