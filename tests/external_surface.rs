use std::{collections::BTreeMap, path::PathBuf};

use datom::{
    DatomFault, DatomHeadedUnit, DatomProblem, DatomRealizing, DatomRoot, DatomText,
    DatomTextualizing, PositionAdvancing, RecordPosition,
};
use protos::{
    Block, Head, Headed, Realize, RealizeScope, RealizeScoping, Shape, ShapeDefined, SourceText,
    TextualizeScope, TextualizeScoping,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum Request {
    Serve(Serve),
    Inspect(PathBuf),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Integer(i64);

#[derive(Clone, Debug, Eq, PartialEq)]
enum Observe {
    Locks,
    ExpiredLocks,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Serve {
    enabled: bool,
    name: String,
    paths: Vec<PathBuf>,
    labels: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RequestSelection {
    Serve,
    Inspect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ServeSelection {
    Serve,
}

impl ShapeDefined for Request {
    type Selection = RequestSelection;

    fn shapes() -> &'static [Shape] {
        &[Shape::DottedBraced]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        match (shape, head) {
            (Shape::DottedBraced, Some(head)) if head == &Head("Serve".into()) => {
                Some(RequestSelection::Serve)
            }
            (Shape::DottedBraced, Some(head)) if head == &Head("Inspect".into()) => {
                Some(RequestSelection::Inspect)
            }
            _ => None,
        }
    }
}

impl ShapeDefined for Serve {
    type Selection = ServeSelection;

    fn shapes() -> &'static [Shape] {
        &[Shape::DottedBraced]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::DottedBraced && head == Some(&Head("Serve".into())))
            .then_some(ServeSelection::Serve)
    }
}

impl DatomRealizing for Request {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        match Self::select(block.shape, block.head()).ok_or(DatomFault {
            problem: DatomProblem::Shape,
        })? {
            RequestSelection::Serve => Ok(Self::Serve(Serve::realize_block(scope, block)?)),
            RequestSelection::Inspect => {
                let mut paths = scope.realize_body(&mut |path_scope, path| {
                    PathBuf::realize_block(path_scope, path)
                })?;
                if paths.len() != 1 {
                    return Err(DatomFault {
                        problem: DatomProblem::Position,
                    });
                }
                Ok(Self::Inspect(paths.remove(0)))
            }
        }
    }
}

impl DatomTextualizing for Request {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        match self {
            Self::Serve(serve) => {
                let head = Head("Serve".into());
                scope.textualize_block(Shape::DottedBraced, Some(&head), |body| {
                    serve.textualize_in(body)
                })
            }
            Self::Inspect(path) => {
                let head = Head("Inspect".into());
                scope.textualize_block(Shape::DottedBraced, Some(&head), |body| {
                    path.textualize_in(body)
                })
            }
        }
    }
}

impl DatomRealizing for Serve {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        if Self::select(block.shape, block.head()).is_none() {
            return Err(DatomFault {
                problem: DatomProblem::Shape,
            });
        }
        let mut position = RecordPosition::default();
        let mut enabled = None;
        let mut name = None;
        let mut paths = None;
        let mut labels = None;
        scope.realize_body(&mut |child_scope, child| {
            match position.next_position() {
                0 => enabled = Some(bool::realize_block(child_scope, child)?),
                1 => name = Some(String::realize_block(child_scope, child)?),
                2 => paths = Some(Vec::<PathBuf>::realize_block(child_scope, child)?),
                3 => {
                    labels = Some(BTreeMap::<String, String>::realize_block(
                        child_scope,
                        child,
                    )?)
                }
                _ => {
                    return Err(DatomFault {
                        problem: DatomProblem::ExtraPosition,
                    });
                }
            }
            Ok(())
        })?;
        Ok(Self {
            enabled: enabled.ok_or(DatomFault {
                problem: DatomProblem::MissingPosition,
            })?,
            name: name.ok_or(DatomFault {
                problem: DatomProblem::MissingPosition,
            })?,
            paths: paths.ok_or(DatomFault {
                problem: DatomProblem::MissingPosition,
            })?,
            labels: labels.ok_or(DatomFault {
                problem: DatomProblem::MissingPosition,
            })?,
        })
    }
}

impl DatomTextualizing for Serve {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        self.enabled.textualize_in(scope)?;
        self.name.textualize_in(scope)?;
        self.paths.textualize_in(scope)?;
        self.labels.textualize_in(scope)
    }
}

impl DatomRoot for Request {}

impl DatomRealizing for Integer {
    fn realize_block(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, DatomFault> {
        Ok(Self(i64::realize_block(scope, block)?))
    }
}

impl DatomTextualizing for Integer {
    fn textualize_in(&self, scope: &mut TextualizeScope<'_>) -> Result<(), DatomFault> {
        self.0.textualize_in(scope)
    }
}

impl DatomRoot for Integer {}

impl DatomHeadedUnit for Observe {
    fn head() -> &'static str {
        "Observe"
    }

    fn select_unit(unit: &str) -> Option<Self> {
        match unit {
            "Locks" => Some(Self::Locks),
            "ExpiredLocks" => Some(Self::ExpiredLocks),
            _ => None,
        }
    }

    fn unit(&self) -> &'static str {
        match self {
            Self::Locks => "Locks",
            Self::ExpiredLocks => "ExpiredLocks",
        }
    }
}

impl DatomRoot for Observe {}

#[test]
fn external_request_realizes_and_textualizes_through_datom() {
    let source = SourceText(
        "Serve.{ true curriculum-deploy [ /etc/curriculum /srv/runtime ] « source.[ local ] mode.[ fast ] » }".into(),
    );
    let text = DatomText::<Request>::from(source.clone());
    let request = text.realize().expect("external request realizes");
    assert_eq!(
        request,
        Request::Serve(Serve {
            enabled: true,
            name: "curriculum-deploy".into(),
            paths: vec![
                PathBuf::from("/etc/curriculum"),
                PathBuf::from("/srv/runtime")
            ],
            labels: BTreeMap::from([
                ("mode".into(), "fast".into()),
                ("source".into(), "local".into()),
            ]),
        })
    );

    let canonical = request
        .textualize_source()
        .expect("canonical request projection");
    assert_eq!(
        canonical,
        SourceText(
            "Serve.{true curriculum-deploy [/etc/curriculum /srv/runtime] «mode.[fast] source.[local]»}".into(),
        )
    );
    assert_eq!(
        DatomText::<Request>::from(canonical)
            .realize()
            .expect("canonical request realizes"),
        request
    );

    let inspect = Request::Inspect(PathBuf::from("/var/lib/curriculum"));
    let canonical = inspect
        .textualize_source()
        .expect("inspect request projection");
    assert_eq!(
        canonical,
        SourceText("Inspect.{/var/lib/curriculum}".into())
    );
    assert_eq!(
        DatomText::<Request>::from(canonical)
            .realize()
            .expect("inspect request realizes"),
        inspect
    );
}

#[test]
fn integer_is_canonical_bare_decimal_i64() {
    for (source, value) in [
        ("0", 0),
        ("42", 42),
        ("-42", -42),
        ("9223372036854775807", i64::MAX),
        ("-9223372036854775808", i64::MIN),
    ] {
        let realized = DatomText::<Integer>::from(SourceText(source.into()))
            .realize()
            .expect("canonical integer realizes");
        assert_eq!(realized, Integer(value));
        assert_eq!(
            realized
                .textualize_source()
                .expect("canonical integer projects"),
            SourceText(source.into())
        );
    }

    for source in [
        "+1",
        "01",
        "-0",
        "-01",
        "1.0",
        "٩",
        "9223372036854775808",
        "-9223372036854775809",
    ] {
        assert!(
            DatomText::<Integer>::from(SourceText(source.into()))
                .realize()
                .is_err(),
            "{source} is not canonical i64 text"
        );
    }
}

#[test]
fn headed_unit_enum_projects_observe_locks_and_future_siblings() {
    assert_eq!(
        DatomText::<Observe>::from(SourceText("Observe.Locks".into()))
            .realize()
            .expect("Observe.Locks realizes"),
        Observe::Locks
    );
    assert_eq!(
        Observe::ExpiredLocks
            .textualize_source()
            .expect("future sibling projects"),
        SourceText("Observe.ExpiredLocks".into())
    );
    assert!(
        DatomText::<Observe>::from(SourceText("Observe.Unknown".into()))
            .realize()
            .is_err()
    );
}
