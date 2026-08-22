use std::collections::BTreeMap;

use datom::{
    DatomProblem, Entry, EvidenceObserving, EvidencedRealizing, EvidencedTextualizing, Group,
    InterimNote, InterimNoteText, PathLock, PathLockConstructing, PathLockPath,
    PathLockPathConstructing, PathLockPathViewing, PathLockRegistered,
    PathLockRegisteredConstructing, PathLockRegisteredText, PathLockRegisteredViewing,
    PathLockRegistrationRejected, PathLockRegistrationRejectedConstructing,
    PathLockRegistrationRejectedText, PathLockRegistrationRejectedViewing,
    PathLockRegistrationRejection, PathLockText, PathLockViewing, ProjectionViewing,
    RealizationViewing, Report, ReportText, Text,
};
use protos::{
    FrameObserving, ObservationViewing, ParentObserving, Realize, SourceSlicing, SourceText,
    Textualize, TransitionObserving, WalkTransitionKind,
};

const DEEP: &str = r#"Report.{
  Q3
  Map.[
    north.[
      Note.(quick note)
      Group.{
        Ops
        [ Note.(sub note)
          Group.{
            (Deep } ] “quote)
            [ Note.tail ]
            Map.[ remark.(child sees } ] and (nested markup) only as text) ] }
          ]
          Map.[ kind.core ] }
      Tags.[ alpha beta ] ] ]
  Some.(inside } ] “current context) }
"#;

const CARRIERS: &str = r#"InterimNote.{
  (a plain string)
  (nested (balanced) parentheses are content — the seed of markup)
  (a lone unbalanced one is escaped \( like this)
  “legacy carrier still accepted”
}
"#;

#[test]
fn path_lock_realizes_normalized_paths_and_canonically_projects() {
    let source = PathLockText {
        source: SourceText(
            "PathLock.{ datom [ //home//li/./primary /var///lock ] (protect the active paths) }"
                .into(),
        ),
    };
    let lock = source.realize().expect("path lock realizes");
    assert_eq!(
        lock,
        PathLock::try_new(
            "datom".into(),
            vec!["/home/li/primary".into(), "/var/lock".into()],
            "protect the active paths".into(),
        )
        .expect("valid path lock")
    );
    assert_eq!(lock.name(), "datom");
    assert_eq!(lock.paths(), ["/home/li/primary", "/var/lock"]);
    assert_eq!(lock.description(), "protect the active paths");
    let canonical = lock.textualize().expect("path lock textualizes");
    assert_eq!(
        canonical.source.0,
        "PathLock.{datom [/home/li/primary /var/lock] (protect the active paths)}"
    );
    assert_eq!(canonical.realize().expect("canonical path lock"), lock);
}

#[test]
fn path_lock_constructor_rejects_invalid_names_paths_and_descriptions() {
    for name in ["", " \t", "first\nsecond"] {
        assert!(
            PathLock::try_new(name.into(), vec!["/home/li".into()], "description".into()).is_err(),
            "{name:?} must be rejected"
        );
    }

    for source in [
        "PathLock.{ () [/home/li] description }",
        "PathLock.{ (first\nsecond) [/home/li] description }",
        "PathLock.{ datom [] description }",
        "PathLock.{ datom [ relative ] description }",
        "PathLock.{ datom [ /home/../li ] description }",
        "PathLock.{ datom [ /home/li //home//li/. ] description }",
        "PathLock.{ datom [ /home/li ] () }",
        "PathLock.{ datom [ /home/li ] (first\nsecond) }",
    ] {
        assert!(
            PathLockText {
                source: SourceText(source.into()),
            }
            .realize()
            .is_err(),
            "{source} must be rejected"
        );
    }

    for (paths, description) in [
        (Vec::new(), "description"),
        (vec!["relative".into()], "description"),
        (vec!["/home/../li".into()], "description"),
        (
            vec!["/home/li".into(), "//home//li/.".into()],
            "description",
        ),
        (vec!["/home/li".into()], " \t"),
        (vec!["/home/li".into()], "first\nsecond"),
    ] {
        assert!(
            PathLock::try_new("datom".into(), paths, description.into()).is_err(),
            "invalid constructor input must be rejected"
        );
    }
}

#[test]
fn path_lock_replies_are_native_and_canonical() {
    assert!(PathLockPath::try_new("relative".into()).is_err());

    let requested = PathLock::try_new(
        "datom".into(),
        vec!["/home/li/primary".into()],
        "protect the active paths".into(),
    )
    .expect("requested lock");
    let holder = PathLock::try_new(
        "other".into(),
        vec!["/var/lock".into()],
        "holds a conflicting path".into(),
    )
    .expect("holder lock");

    let registered = PathLockRegistered::new(requested.clone());
    assert_eq!(registered.lock(), &requested);
    let registered_text = registered.textualize().expect("registered text");
    assert_eq!(
        registered_text.source.0,
        "PathLockRegistered.{PathLock.{datom [/home/li/primary] (protect the active paths)}}"
    );
    assert_eq!(
        PathLockRegisteredText {
            source: registered_text.source
        }
        .realize()
        .expect("registered realizes"),
        registered
    );

    let duplicate = PathLockRegistrationRejected::new(
        requested.clone(),
        PathLockRegistrationRejection::DuplicateActiveName {
            holder: holder.clone(),
        },
    );
    assert_eq!(duplicate.requested(), &requested);
    assert!(matches!(
        duplicate.reason(),
        PathLockRegistrationRejection::DuplicateActiveName { holder: actual } if actual == &holder
    ));
    assert_eq!(
        duplicate.textualize().expect("duplicate text").source.0,
        "PathLockRegistrationRejected.{PathLock.{datom [/home/li/primary] (protect the active paths)} DuplicateActiveName.{PathLock.{other [/var/lock] (holds a conflicting path)}}}"
    );

    let overlap = PathLockRegistrationRejected::new(
        requested,
        PathLockRegistrationRejection::PathOverlap {
            path: PathLockPath::try_new("//var///lock/.".into()).expect("normalized conflict"),
            holder,
        },
    );
    assert!(matches!(
        overlap.reason(),
        PathLockRegistrationRejection::PathOverlap { path, .. } if path.path() == "/var/lock"
    ));
    assert_eq!(
        overlap.textualize().expect("overlap text").source.0,
        "PathLockRegistrationRejected.{PathLock.{datom [/home/li/primary] (protect the active paths)} PathOverlap.{/var/lock PathLock.{other [/var/lock] (holds a conflicting path)}}}"
    );
    assert_eq!(
        PathLockRegistrationRejectedText {
            source: overlap.textualize().expect("stable overlap text").source
        }
        .realize()
        .expect("overlap realizes"),
        overlap
    );
}

#[test]
fn deep_ruled_fixture_uses_protos_scopes_and_is_canonically_stable() {
    let source = ReportText {
        source: SourceText(DEEP.into()),
    };
    let realized = source.realize_evidenced().expect("deep fixture realizes");
    let report = realized.value();
    assert_eq!(report.heading.0, "Q3");
    assert_eq!(report.groups.len(), 1);
    let north = report.groups.get("north").expect("north key");
    assert_eq!(north.len(), 3);
    assert_eq!(north[0], Entry::Note(Text("quick note".into())));
    let Entry::Group(group) = &north[1] else {
        panic!("second north entry is Group");
    };
    assert_eq!(group.title, Text("Ops".into()));
    assert_eq!(group.children[0], Entry::Note(Text("sub note".into())));
    let Entry::Group(deep) = &group.children[1] else {
        panic!("nested child is Group");
    };
    assert_eq!(deep.title, Text("Deep } ] “quote".into()));
    assert_eq!(deep.children, vec![Entry::Note(Text("tail".into()))]);
    assert_eq!(
        deep.annotations.get("remark"),
        Some(&Text(
            "child sees } ] and (nested markup) only as text".into()
        ))
    );
    assert_eq!(group.annotations.get("kind"), Some(&Text("core".into())));
    assert_eq!(
        north[2],
        Entry::Tags(datom::TagList(vec![
            Text("alpha".into()),
            Text("beta".into())
        ]))
    );
    assert_eq!(
        report.latest.as_ref().expect("Some").0,
        "inside } ] “current context"
    );

    let canonical = report.textualize().expect("canonical projection");
    let again = canonical.realize().expect("canonical source realizes");
    assert_eq!(again, report.clone());
    assert_eq!(again.textualize().expect("stable projection"), canonical);

    let source_evidence = realized.evidence();
    assert_eq!(source_evidence.cursor(), source.source.0.len());
    let source_history = source_evidence.observation().history();
    for close in source_history
        .iter()
        .filter(|transition| transition.kind() == WalkTransitionKind::Close)
    {
        if let Some(parent) = close.parent_before() {
            assert_eq!(close.parent_after(), Some(parent));
            let resume = &source_history[close.ordinal() + 1];
            assert_eq!(resume.kind(), WalkTransitionKind::Resume);
            assert_eq!(resume.frame().identity(), parent.frame());
            assert_eq!(resume.parent_before(), Some(parent));
            assert_eq!(
                resume.parent_after().expect("advance").position(),
                parent.position() + 1
            );
        }
        assert!(source.source.source_slice(close.frame().span()).is_some());
    }
    assert!(source_history.iter().any(|transition| {
        transition.kind() == WalkTransitionKind::Close
            && source
                .source
                .source_slice(transition.frame().span())
                .is_some_and(|text| text.starts_with("north.["))
    }));

    let projected = report.textualize_evidenced().expect("evidenced projection");
    let output = &projected.text().source;
    let output_history = projected.evidence().observation().history();
    assert_eq!(projected.evidence().cursor(), output.0.len());
    for close in output_history
        .iter()
        .filter(|transition| transition.kind() == WalkTransitionKind::Close)
    {
        if let Some(parent) = close.parent_before() {
            assert_eq!(close.parent_after(), Some(parent));
            let resume = &output_history[close.ordinal() + 1];
            assert_eq!(resume.kind(), WalkTransitionKind::Resume);
            assert_eq!(resume.frame().identity(), parent.frame());
            assert_eq!(resume.parent_before(), Some(parent));
            assert_eq!(
                resume.parent_after().expect("advance").position(),
                parent.position() + 1
            );
        }
        assert!(output.source_slice(close.frame().span()).is_some());
    }
    let keyed = output_history
        .iter()
        .find(|transition| {
            transition.kind() == WalkTransitionKind::Close
                && output
                    .source_slice(transition.frame().span())
                    .is_some_and(|text| text.starts_with("north.["))
        })
        .expect("north keyed map frame");
    assert!(
        output
            .source_slice(keyed.frame().span())
            .expect("north frame")
            .starts_with("north.[")
    );
    let tags = output_history
        .iter()
        .find(|transition| {
            transition.kind() == WalkTransitionKind::Close
                && output
                    .source_slice(transition.frame().span())
                    .is_some_and(|text| text.starts_with("Tags.["))
        })
        .expect("single Tags frame");
    assert_eq!(
        output.source_slice(tags.frame().span()),
        Some("Tags.[alpha beta]")
    );
    assert!(
        !output
            .source_slice(tags.frame().span())
            .expect("Tags frame")
            .contains("[[")
    );
    let groups: Vec<_> = output_history
        .iter()
        .filter(|transition| {
            transition.kind() == WalkTransitionKind::Close
                && output
                    .source_slice(transition.frame().span())
                    .is_some_and(|text| text.starts_with("Group.{"))
        })
        .collect();
    assert_eq!(groups.len(), 2);
    assert!(groups.iter().all(|group| {
        !output
            .source_slice(group.frame().span())
            .expect("Group frame")
            .contains("{{")
    }));
}

#[test]
fn interim_note_accepts_both_carriers_and_reescapes_unbalanced_parentheses() {
    let source = InterimNoteText {
        source: SourceText(CARRIERS.into()),
    };
    let note = source.realize().expect("carrier fixture realizes");
    assert_eq!(note.a, "a plain string");
    assert_eq!(
        note.b,
        "nested (balanced) parentheses are content — the seed of markup"
    );
    assert_eq!(note.c, "a lone unbalanced one is escaped ( like this");
    assert_eq!(note.d, "legacy carrier still accepted");

    let canonical = note.textualize().expect("carrier projection");
    assert!(canonical.source.0.contains(r"\("));
    assert!(!canonical.source.0.contains('“'));
    assert_eq!(canonical.realize().expect("canonical carrier source"), note);
}

#[test]
fn headed_legacy_curly_carriers_are_contextual_input_only() {
    let bare = ReportText {
        source: SourceText(
            "Report.{ Q3 Map.[ north.[ Note.“legacy” Group.{ Ops [] Map.[ remark.“legacy”] } ] ] Some.“legacy” }"
                .into(),
        ),
    };
    let report = bare.realize().expect("headed legacy carriers realize");
    let north = report.groups.get("north").expect("north");
    assert_eq!(north[0], Entry::Note(Text("legacy".into())));
    let Entry::Group(group) = &north[1] else {
        panic!("second north entry is Group");
    };
    assert_eq!(
        group.annotations.get("remark"),
        Some(&Text("legacy".into()))
    );
    assert_eq!(report.latest, Some(Text("legacy".into())));
    let canonical = report.textualize().expect("bare canonical headed text");
    assert!(canonical.source.0.contains("Note.legacy"));
    assert!(canonical.source.0.contains("remark.legacy"));
    assert!(canonical.source.0.contains("Some.legacy"));
    assert!(!canonical.source.0.contains('“'));

    let delimited = ReportText {
        source: SourceText(
            "Report.{ Q3 Map.[ north.[ Note.“legacy } ]” Group.{ Ops [] Map.[ remark.“legacy } ]”] } ] ] Some.“legacy } ]” }"
                .into(),
        ),
    };
    let canonical = delimited
        .realize()
        .expect("delimited headed legacy carriers realize")
        .textualize()
        .expect("parenthesis canonical headed text");
    assert!(canonical.source.0.contains("Note.(legacy } ])"));
    assert!(canonical.source.0.contains("remark.(legacy } ])"));
    assert!(canonical.source.0.contains("Some.(legacy } ])"));
    assert!(!canonical.source.0.contains('“'));
}

#[test]
fn optional_bare_and_delimited_forms_are_canonical_and_stable() {
    let bare = Report {
        heading: Text("Q3".into()),
        groups: BTreeMap::new(),
        latest: Some(Text("core".into())),
    };
    let bare_text = bare.textualize().expect("bare optional projection");
    assert!(bare_text.source.0.contains("Some.core"));
    assert_eq!(bare_text.realize().expect("bare optional input"), bare);

    let delimited = Report {
        heading: Text("Q3".into()),
        groups: BTreeMap::new(),
        latest: Some(Text("inside } ]".into())),
    };
    let delimited_text = delimited
        .textualize()
        .expect("delimited optional projection");
    assert!(delimited_text.source.0.contains("Some.(inside } ])"));
    assert_eq!(
        delimited_text.realize().expect("delimited optional input"),
        delimited
    );
}

#[test]
fn parenthesis_projection_preserves_escapes_and_trailing_backslash() {
    let note = InterimNote {
        a: "trailing \\".into(),
        b: "literal \\( and \\)".into(),
        c: "nested (balanced) parentheses".into(),
        d: "a lone ( stays literal".into(),
    };
    let projected = note.textualize().expect("projection");
    assert!(projected.source.0.contains(r"\\"));
    assert!(projected.source.0.contains(r"\("));
    assert!(projected.source.0.contains(r"\)"));
    assert_eq!(projected.realize().expect("escaped carrier input"), note);
}

#[test]
fn map_key_duplicate_and_record_arity_fail_without_new_grammar() {
    for key in ["", "bad key", "north.bad", "bad["] {
        let invalid_group_key = Report {
            heading: Text("Q3".into()),
            groups: BTreeMap::from([(key.into(), Vec::new())]),
            latest: None,
        };
        assert_eq!(
            invalid_group_key.textualize(),
            Err(datom::DatomFault {
                problem: DatomProblem::AmbiguousMapPair
            })
        );
    }
    let invalid_text_key = Group {
        title: Text("Ops".into()),
        children: Vec::new(),
        annotations: BTreeMap::from([("bad key".into(), Text("core".into()))]),
    };
    let report = Report {
        heading: Text("Q3".into()),
        groups: BTreeMap::from([("north".into(), vec![Entry::Group(invalid_text_key)])]),
        latest: None,
    };
    assert_eq!(
        report.textualize(),
        Err(datom::DatomFault {
            problem: DatomProblem::AmbiguousMapPair
        })
    );
    let invalid_delimited_text_key = Group {
        title: Text("Ops".into()),
        children: Vec::new(),
        annotations: BTreeMap::from([("bad[".into(), Text("inside } ]".into()))]),
    };
    let report = Report {
        heading: Text("Q3".into()),
        groups: BTreeMap::from([(
            "north".into(),
            vec![Entry::Group(invalid_delimited_text_key)],
        )]),
        latest: None,
    };
    assert_eq!(
        report.textualize(),
        Err(datom::DatomFault {
            problem: DatomProblem::AmbiguousMapPair
        })
    );

    let duplicate = ReportText {
        source: SourceText(
            "Report.{ Q3 Map.[ north.[ Note.one ] north.[ Note.two ] ] None }".into(),
        ),
    };
    assert_eq!(
        duplicate.realize(),
        Err(datom::DatomFault {
            problem: DatomProblem::Position
        })
    );
    let missing = ReportText {
        source: SourceText("Report.{ Q3 Map.[] }".into()),
    };
    assert_eq!(
        missing.realize(),
        Err(datom::DatomFault {
            problem: DatomProblem::MissingPosition
        })
    );

    let wrong_vector = ReportText {
        source: SourceText(
            "Report.{ Q3 Map.[ north.[ Group.{ Ops Wrong.[ Note.one ] Map.[] } ] ] None }".into(),
        ),
    };
    assert_eq!(
        wrong_vector.realize(),
        Err(datom::DatomFault {
            problem: DatomProblem::Shape
        })
    );
}

#[test]
fn direct_scalar_fields_reject_unrecognized_heads() {
    let report = ReportText {
        source: SourceText("Report.{ Wrong.(oops) Map.[] None }".into()),
    };
    assert_eq!(
        report.realize(),
        Err(datom::DatomFault {
            problem: DatomProblem::Shape
        })
    );
    let interim = InterimNoteText {
        source: SourceText("InterimNote.{ Wrong.(a) b c d }".into()),
    };
    assert_eq!(
        interim.realize(),
        Err(datom::DatomFault {
            problem: DatomProblem::Shape
        })
    );
}
