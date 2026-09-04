# Upgrading from datomic 0.8 to 0.9

## New features

### Datomic for Situated<F>

`protos::Situated<F>` now implements `Corporal<Datom>` and `Datomic` for
any `F: Datomic`. The datom is `{ Option<Extent> <F's datom> }`, matching
the format orchestrate prints to stderr:
`Unreadable.{ Some.{ 5 13 } Structural.{ { 5 13 } Unclosed.Braced } }`.

Import `datomic:[ Situated Fault ]` and use `Situated<Fault>` directly.

### impl_datomic_box! macro for recursive types

Rust's orphan rule prevents a blanket `Corporal<Datom>` impl for `Box<T>`.
For recursive types that require `Box<T>`, call:

```rust
datomic::impl_datomic_box!(YourType);
```

This generates transparent `Corporal<Datom>` and `Datomic` impls for
`Box<YourType>`: a Box carries its content's datom exactly.

---

# Upgrading from datomic 0.7 to 0.8

## Breaking changes

### Portion is now Protoform

All references to `Portion` become `Protoform`. The protos crate
(0.15.0) provides the new types.

### New concept type: Datom

The `Datom` enum replaces the old `Portion`-based pattern matching.
Instead of pattern-matching on `Portion::Headed`, `Portion::Enclosed`,
etc., you now work with `Datom::Variant`, `Datom::Struct`,
`Datom::Vector`, etc.

### Datomic trait: embody/portion -> incorporate/datomize

| 0.7 | 0.8 |
|---|---|
| `Datomic::embody(portion: &Portion) -> Result<Self, Fault>` | `Datomic::incorporate(datom: Datom) -> Result<Self, Fault>` |
| `Datomic::portion(&self) -> Portion` | `Datomic::datomize(&self) -> Datom` |

`incorporate` takes ownership of the Datom (no reference).
`datomize` returns a Datom (not a Protoform).

### DatomicString -> String

`DatomicString` is removed. Use `String` directly; datomize handles
the bare-safe check internally.

### FiniteDecimal -> f64

`FiniteDecimal` is removed. Use `f64` directly; incorporate checks
for finite values.

### TextEdge -> DatomicActualizable

The `TextEdge` trait is replaced by `DatomicActualizable`, which
provides `actualize()` on `Potential<T>` for any `T: Datomic`.

### Fault taxonomy

The old `Fault { extent, problem: FaultProblem }` becomes a three-layer
`Fault` enum: `Structural(protos::Fault)`, `Conceptual(Path, Problem)`,
`Corporal(Path, Problem)`. The `Situated` struct joins a fault to an
extent via the delineation's situation map.

### PortionViewing / PortionBuilding removed

These helper traits are no longer needed. The Datom concept type
provides the abstraction layer; implement `Datomic` directly.

### Migration steps

1. Update protos dependency to 0.15.0.
2. Replace all `Portion`-based code with `Datom`-based code.
3. Rename `embody` -> `incorporate`, `portion` -> `datomize`.
4. Replace `DatomicString` with `String`.
5. Replace `FiniteDecimal` with `f64`.
6. Replace `TextEdge::embody` with `DatomicActualizable::actualize`.
7. Update fault handling to the three-layer taxonomy.
