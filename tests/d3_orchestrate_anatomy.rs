use datomic::{
    Datomic, DatomicString, Fault, FaultProblem, PortionBuilding, PortionViewing, Text, TextEdge,
};
use protos::{Portion, Separator, StructuralEnclosure};

struct LockRequest {
    name: DatomicString,
    flow: DatomicString,
    paths: Vec<DatomicString>,
    reason: DatomicString,
}

struct Lock {
    identifier: i64,
    name: DatomicString,
    flow: DatomicString,
    paths: Vec<DatomicString>,
    reason: DatomicString,
}

struct LockOverlap {
    path: DatomicString,
    lock: Lock,
}

enum LockRejection {
    DuplicateName(Lock),
    PathOverlap(LockOverlap),
}

enum ReleaseRejection {
    UnknownLockId,
}

enum Observation {
    Locks(Vec<Lock>),
}

enum Reply {
    Locked(Box<Lock>),
    LockRejected(Box<LockRejection>),
    Released(Box<Lock>),
    ReleaseRejected(Box<ReleaseRejection>),
    Observed(Box<Observation>),
}

enum Operation {
    Lock(Box<LockRequest>),
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
            name: DatomicString::embody(name)?,
            flow: DatomicString::embody(flow)?,
            paths: Vec::<DatomicString>::embody(paths)?,
            reason: DatomicString::embody(reason)?,
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
            name: DatomicString::embody(name)?,
            flow: DatomicString::embody(flow)?,
            paths: Vec::<DatomicString>::embody(paths)?,
            reason: DatomicString::embody(reason)?,
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

impl Datomic for LockOverlap {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(fields) = portion.structural(StructuralEnclosure::Braced) else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        let [path, lock] = fields else {
            return Err(portion.fault(FaultProblem::Arity));
        };
        Ok(Self {
            path: DatomicString::embody(path)?,
            lock: Lock::embody(lock)?,
        })
    }

    fn portion(&self) -> Portion {
        "".structural(
            StructuralEnclosure::Braced,
            vec![self.path.portion(), self.lock.portion()],
        )
    }
}

impl Datomic for LockRejection {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(headed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        if headed.separator != Separator::Period {
            return Err(portion.fault(FaultProblem::Head));
        }
        match headed.head.as_ref() {
            "DuplicateName" => Lock::embody(&headed.body).map(Self::DuplicateName),
            "PathOverlap" => LockOverlap::embody(&headed.body).map(Self::PathOverlap),
            _ => Err(portion.fault(FaultProblem::Head)),
        }
    }

    fn portion(&self) -> Portion {
        match self {
            Self::DuplicateName(lock) => "DuplicateName".headed(Separator::Period, lock.portion()),
            Self::PathOverlap(overlap) => {
                "PathOverlap".headed(Separator::Period, overlap.portion())
            }
        }
    }
}

impl Datomic for ReleaseRejection {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        (portion.bare_symbol() == Some("UnknownLockId"))
            .then_some(Self::UnknownLockId)
            .ok_or_else(|| portion.fault(FaultProblem::Head))
    }

    fn portion(&self) -> Portion {
        match self {
            Self::UnknownLockId => "UnknownLockId".bare(),
        }
    }
}

impl Datomic for Observation {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(headed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        if headed.head.as_ref() != "Locks" || headed.separator != Separator::Period {
            return Err(portion.fault(FaultProblem::Head));
        }
        Vec::<Lock>::embody(&headed.body).map(Self::Locks)
    }

    fn portion(&self) -> Portion {
        match self {
            Self::Locks(locks) => "Locks".headed(Separator::Period, locks.portion()),
        }
    }
}

impl Datomic for Reply {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        let Some(headed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        if headed.separator != Separator::Period {
            return Err(portion.fault(FaultProblem::Head));
        }
        match headed.head.as_ref() {
            "Locked" => Lock::embody(&headed.body).map(|lock| Self::Locked(Box::new(lock))),
            "LockRejected" => LockRejection::embody(&headed.body)
                .map(|rejection| Self::LockRejected(Box::new(rejection))),
            "Released" => Lock::embody(&headed.body).map(|lock| Self::Released(Box::new(lock))),
            "ReleaseRejected" => ReleaseRejection::embody(&headed.body)
                .map(|rejection| Self::ReleaseRejected(Box::new(rejection))),
            "Observed" => Observation::embody(&headed.body)
                .map(|observation| Self::Observed(Box::new(observation))),
            _ => Err(portion.fault(FaultProblem::Head)),
        }
    }

    fn portion(&self) -> Portion {
        match self {
            Self::Locked(lock) => "Locked".headed(Separator::Period, lock.portion()),
            Self::LockRejected(rejection) => {
                "LockRejected".headed(Separator::Period, rejection.portion())
            }
            Self::Released(lock) => "Released".headed(Separator::Period, lock.portion()),
            Self::ReleaseRejected(rejection) => {
                "ReleaseRejected".headed(Separator::Period, rejection.portion())
            }
            Self::Observed(observation) => {
                "Observed".headed(Separator::Period, observation.portion())
            }
        }
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
            "Lock" => {
                LockRequest::embody(&headed.body).map(|request| Self::Lock(Box::new(request)))
            }
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
    assert_eq!(lock.name.as_ref(), "datomicD0D4");
    assert_eq!(lock.flow.as_ref(), "root/realize_datomic");
    assert!(lock.paths.iter().map(AsRef::as_ref).eq(["/a", "/b"]));
    assert_eq!(lock.reason.as_ref(), "one line");
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

#[test]
fn approved_orchestrate_replies_round_trip_byte_identically_through_d4() {
    for source in [
        "Locked.{7 lock flow [/a /b] reason}",
        "LockRejected.DuplicateName.{7 lock flow [/a /b] reason}",
        "LockRejected.PathOverlap.{/a {7 lock flow [/a /b] reason}}",
        "Released.{7 lock flow [/a /b] reason}",
        "ReleaseRejected.UnknownLockId",
        "Observed.Locks.[]",
    ] {
        let reply = Text::<Reply>::from(source)
            .embody()
            .expect("approved reply embodies");
        assert_eq!(reply.textualize().as_ref(), source);
    }
}
