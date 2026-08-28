# Datomic agent rules

- Reserve the complete Datomic write set before editing and release its lock
  after the tree is clean and pushed.
- Datomic receives and emits Protos `Portion` values only. Never introduce a
  dialect character reader, writer, or numeric scanner.
- Public outbound anatomies are total. Validate representability when creating
  an invariant-bearing Datomic value, never by panicking during projection.
- Preserve clean breaks: update consumers in their authorized work, never add
  a legacy Datom API or syntax path here.
