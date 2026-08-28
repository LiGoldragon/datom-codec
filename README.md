# Datomic

Datomic is pure positional typed data on the Protos Portion substrate. A
single hand-written `Datomic` anatomy maps an expected Rust type to and from
`Portion`; Protos alone delineates and prints text. This 0.6.0 release is a
clean breaking replacement for Datom's former scoped-walk API.

Canonical text uses `True` / `False`, finite decimal `f64` values with a
point and no exponent, `None` / `Some.value`, headless alternating guillemet
maps, opaque curly strings, and flat layout. Struct fields are positional;
heads are variants and re-emit themselves.

## Migration

Update every consumer to package and import `datomic`. Replace
`DatomRealizing`, `DatomTextualizing`, `DatomRoot`, `DatomText`, and walk
evidence types with the one `Datomic` anatomy and `Text<T>` edge. Existing
Datom syntax and APIs deliberately have no compatibility path.
