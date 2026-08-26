# datom

Datom is pure positional typed data on the published Protos substrate. It
does not generate Rust; that belongs to Ethos. Protos owns lexical blocks,
shape vocabulary, string carriers, and the only structural walk. Datom adds
only context-sensitive typed realization and textual projection.

## Current typed surface

- `Report`, `Entry`, `Group`, `TagList`, `Text`, and `InterimNote` are the
  concrete positional models used by the registered design examples.
- `ReportText` and `InterimNoteText` are concrete typed textual carriers. They
  implement Protos `Realize`; their matching real types implement Protos
  `Textualize`.
- `DatomText<T>` is the public typed textual carrier for a consumer-defined
  `DatomRoot`. `DatomRealizing` and `DatomTextualizing` are the scoped schema
  seams for its variants and positional records; `DatomRoot` starts the one
  document walk and lets the expected root type select its own syntax.
- `String`, `bool`, `i64`, `PathBuf`, `Vec<T>`, and `BTreeMap<String, T>` implement
  those seams. Booleans project as bare `true` or `false`; paths use Datom's
  ordinary string carrier and reject non-Unicode host paths. `i64` accepts and
  projects canonical bare ASCII decimal: `0`, nonzero positive digits, or `-`
  followed by nonzero digits, range-checked to `i64`. Vectors are headless
  square blocks. Generic maps project as `«key.[value]»`, one value
  per key; their schema-owned entry forms remain unchanged.
- `EvidencedRealizing` and `EvidencedTextualizing` return those same typed
  values/texts paired with read-only transition evidence copied from the
  actual Protos driver; dialect code never owns or proxies `Walk`.
- Records read and write field positions only. Variants carry their Head;
  their payload is headless in its own context.
- Strings project bare exactly when the Protos scanner can carry the complete
  value as one bare block; otherwise they use curly quotes. A curly-quoted
  String block keeps its interior opaque to structural delimiters. Parentheses
  are reserved for the still-unimplemented structured String, Meaning.
- `«north.[…]»` carries a keyed vector entry with one structural Protos frame.
  `«kind.core»` carries the existing bare pair form. Keys containing dots,
  and delimited keys followed by `.`, are deliberately unsupported pending a
  psyche ruling; they return a Datom fault rather than changing Protos.
- `DatomHeadedUnit` is the reusable payloadless enum seam. An enum that
  implements it and `DatomRoot` reads and writes a headed unit directly, such
  as `Observe.Locks` or `Observe.ExpiredLocks`; its `Head` is fixed by the
  enum, and `select_unit` / `unit` select the sibling unit.

Canonical text is a block projection, not preservation of original whitespace.

## Consumer-defined root

An external program defines its own root type and positional configuration
records, implements `DatomRealizing` and `DatomTextualizing` for each, and
implements `DatomRoot` on the root type. An enum root reads and writes its
variant directly; a non-enum root reads and writes its own selected shape. It reads with
`DatomText::<Request>::from(SourceText(input.into())).realize()` and writes
with `request.textualize_source()`. The supplied scopes are the only recursive
entry point: do not construct a Protos walk or parser inside the program.
