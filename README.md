# Datomic

Datomic is pure positional typed data on the Protos Portion substrate. A
single hand-written `Datomic` anatomy maps an expected Rust type to and from
`Portion`; Protos alone delineates and prints text. This 0.7.0 release is a
clean breaking replacement for Datom's former scoped-walk API.

Canonical text uses `True` / `False`, `FiniteDecimal` values with a point and
no exponent, `None` / `Some.value`, headless alternating guillemet maps,
opaque `DatomicString` values, and flat layout. Struct fields are positional;
heads are variants and re-emit themselves.

An expected String recovers a single Protos Portion as canonical text, so
`a.b`, `a!b`, and `a:b` remain bare strings. Content that forms multiple
Portions, such as `two words`, uses opaque curly quotes. This behavior relies
on Protos 0.14's expected-String Portion builder and `PortionText` boundary.

## Migration

Update every consumer to package and import `datomic`. Replace
`DatomRealizing`, `DatomTextualizing`, `DatomRoot`, `DatomText`, and walk
evidence types with the one `Datomic` anatomy and `Text<T>` edge. Replace raw
`f64` and `String` anatomy with `FiniteDecimal` and `DatomicString`: both
types are representable before an outbound Portion exists. Existing Datom
syntax and APIs deliberately have no compatibility path.
