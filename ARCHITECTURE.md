# Datom architecture

Datom is the typed, positional data dialect on Protos. Protos scans lexical
blocks and owns the single structural walk. Datom assigns those blocks to a
consumer's typed schema and emits the exact reverse canonical block form.

`DatomRoot` is the document boundary: it starts one Protos walk for realization
or textual projection, then its expected type selects the root block. An enum
root starts with its selected variant; a non-enum root has no variant Head.
`DatomText<T>` is the matching textual carrier. `DatomRealizing` and
`DatomTextualizing` receive only live scoped handles; records consume fields by
position and variants own their Head. This leaves structural lifecycle and
lexical parsing outside both the consumer and the dialect schema.

The reusable scalar/container surface is `String`, `bool`, `i64`, `PathBuf`,
`Vec<T>`, and `BTreeMap<String, T>`. `i64` is canonical bare decimal and
range-checked during realization. Maps are headless guillemet blocks; their
entries retain the schema-owned forms already used for one value under a key.

`DatomHeadedUnit` realizes and textualizes a payloadless enum unit as a
headed bare block (`Head.Unit`). It is the type-directed composition for
families such as `Observe.Locks` and `Observe.ExpiredLocks`; Protos supplies
the head/body structure, while Datom selects the unit in the expected enum.
