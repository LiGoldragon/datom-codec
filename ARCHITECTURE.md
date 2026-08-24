# Datom architecture

Datom is the typed, positional data dialect on Protos. Protos scans lexical
blocks and owns the single structural walk. Datom assigns those blocks to a
consumer's typed schema and emits the exact reverse canonical block form.

`DatomRoot` is the document boundary: a root is a headed brace block and it
starts one Protos walk for realization or textual projection. `DatomText<T>`
is the matching textual carrier. `DatomRealizing` and `DatomTextualizing`
receive only live scoped handles; records consume fields by position and
variants own their Head. This leaves structural lifecycle and lexical parsing
outside both the consumer and the dialect schema.

The reusable scalar/container surface is `String`, `bool`, `PathBuf`,
`Vec<T>`, and `BTreeMap<String, T>`. Map entries have one schema value under
their key, so `Map.[key.[value]]` remains unambiguous while concrete models
such as `Report` keep their already-published, more specific projections.
