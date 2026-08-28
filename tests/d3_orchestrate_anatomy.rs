use datomic::{
    Datomic, DatomicString, Fault, FaultProblem, PortionBuilding, PortionViewing, Text, TextEdge,
};
use protos::{Portion, Separator, StructuralEnclosure};

type LockName = DatomicString;
type FlowId = DatomicString;
type LockPath = DatomicString;
type LockPaths = Vec<LockPath>;
type LockReason = DatomicString;
type LockId = i64;
type Locks = Vec<Lock>;

struct LockRequest {
    name: LockName,
    flow: FlowId,
    paths: LockPaths,
    reason: LockReason,
}

struct Lock {
    identifier: LockId,
    name: LockName,
    flow: FlowId,
    paths: LockPaths,
    reason: LockReason,
}

struct LockOverlap {
    path: LockPath,
    lock: Lock,
}

enum LockRejection {
    DuplicateName(Lock),
    PathOverlap(LockOverlap),
}

enum ReleaseRejection {
    UnknownLockId,
}

enum ObserveSelection {
    Locks,
}

enum Observation {
    Locks(Locks),
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
    Release(LockId),
    Observe(ObserveSelection),
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
            name: LockName::embody(name)?,
            flow: FlowId::embody(flow)?,
            paths: LockPaths::embody(paths)?,
            reason: LockReason::embody(reason)?,
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
            identifier: LockId::embody(identifier)?,
            name: LockName::embody(name)?,
            flow: FlowId::embody(flow)?,
            paths: LockPaths::embody(paths)?,
            reason: LockReason::embody(reason)?,
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
            path: LockPath::embody(path)?,
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

impl Datomic for ObserveSelection {
    fn embody(portion: &Portion) -> Result<Self, Fault> {
        (portion.bare_symbol() == Some("Locks"))
            .then_some(Self::Locks)
            .ok_or_else(|| portion.fault(FaultProblem::Head))
    }

    fn portion(&self) -> Portion {
        match self {
            Self::Locks => "Locks".bare(),
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
            "Release" => {
                let Some(fields) = headed.body.structural(StructuralEnclosure::Braced) else {
                    return Err(portion.fault(FaultProblem::Shape));
                };
                let [identifier] = fields else {
                    return Err(portion.fault(FaultProblem::Arity));
                };
                i64::embody(identifier).map(Self::Release)
            }
            "Observe" => ObserveSelection::embody(&headed.body).map(Self::Observe),
            _ => Err(portion.fault(FaultProblem::Head)),
        }
    }

    fn portion(&self) -> Portion {
        match self {
            Self::Lock(request) => "Lock".headed(Separator::Period, request.portion()),
            Self::Release(identifier) => "Release".headed(
                Separator::Period,
                "".structural(StructuralEnclosure::Braced, vec![identifier.portion()]),
            ),
            Self::Observe(selection) => "Observe".headed(Separator::Period, selection.portion()),
        }
    }
}

#[test]
fn approved_orchestrate_operations_use_one_declarative_anatomy_pattern() {
    for source in [
        "Lock.{datomicD0D4 root/realize_datomic [/a /b] “one line”}",
        "Release.{-42}",
        "Observe.Locks",
    ] {
        let operation = Text::<Operation>::from(source)
            .embody()
            .expect("approved request root embodies");
        assert_eq!(operation.textualize().as_ref(), source);
    }
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
        "Observed.Locks.[{7 lock flow [/a /b] reason}]",
    ] {
        let reply = Text::<Reply>::from(source)
            .embody()
            .expect("approved reply embodies");
        assert_eq!(reply.textualize().as_ref(), source);
    }
}
