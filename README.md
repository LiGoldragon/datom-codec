# datom-codec

The pure-data dialect on protos. Datom carries data, strictly typed, and its
whole work is serialization and deserialization. Schema-driven and positional:
the reader walks the expected type, writing is the exact reverse projection,
and all naming lives in the type; the text carries only the data.

```
{ Ada 1990 { «12 Rue de la Paix» Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }
```

## In and out

```rust
let person: Person = Potential::<Person>::from(text).actualize(&mut budget)?;  // may fault
let text: String = person.datomize(Path::new()).protosize().textualize();      // cannot
```

`Potential<T>` holds the text, the reader tree once it has one, and actualizes
by protosize, datomize, compose. The ascent is total.

## The datom

`Datom` is a `Form` at a `Path`: `Struct`, `Vector`, `Variant` (a head and a
body), `Bare`, `String` (guillemets), `Meaning` (parentheses). A datom's path
is where it sits: a struct's or vector's child *i* is at `child(i)`, and a
variant's body is at `child(1)`, the convention protos states.

What a structure means — struct, vector, string, integer, decimal, variant — is
said by the position it sits in, never by the structure alone. `3.14` is a
headed structure to protos; a Decimal position rejoins it, a String position
reads it as the text `3.14`, and an Integer position refuses it.

## The kinds

`Datomizable::datomize(at)` projects a value into a datom at a path.
`Composing::compose(datom, budget)` reads one back. Both are derived, with no
attributes, for any Rust struct or enum: field order is position order, a
field's type is the position's type, a variant carrying nothing is its head
alone, a single-field variant carries its type's own form, and a multi-field
variant carries an inline struct. Hand-written impls are reserved to the
intrinsics: `String`, `i64`, `Decimal`, `bool`, `Meaning`, `Vec`, `Option`,
`Result`, `Box`, the tuples, and the protos types the errors carry.

`Decimal` is the finite decimal, and `f64` bears no datom kind. A datom
decimal is finite and point-mandatory; a bare `f64` is neither, so it has no
datom text and no position reads one back. `Decimal` holds its float privately
and admits only a finite value, which is why it is `Eq`, `Hash` and `Ord` where
`f64` is none of them.

The datom composes; the type states its positions. A type whose form is a
struct bears `Compositional`: it declares `ARITY` and builds itself from
`Positions` in order, and reads nothing. The reading — the budget it spends,
the arity it demands, the positions it hands out, and the path it refuses at —
is `Composable::compose_positions` on the datom, written once. A multi-field
variant's payload is a positional type with no name of its own, so it is read
as the tuple it is; the tuples bear `Compositional` for arities two to twelve.

## Strings

A string is written bare when the run needs no delimiters: no space, no
delimiter glyph, and no separator run that would swallow the structure around
it — a leading or trailing `.`, `!` or `:`, or two of them adjacent. So
`gpt-5.6-luna`, `a:b:c` and `https://example.org/a` are bare, and `.a`, `a..b`
and `5::7/128` are written in guillemets. Everything else is delimited, where
a closing guillemet is escaped with a backslash.

## Errors

An `Error` names the layer that raised it — `Protos`, `Datom`, `Composition` —
the path of the datom where it arose, and its kind: `Budget`, `Structural`,
`Form`, `Arity`, `Value`, `Variant`. Errors are themselves datomizable, and
`Potential::reader_extent(path)` resolves an error's path back to the protos
node, and so to the extent in the text.

```
[ 1 x ]                        ; read as Vector<Integer>
Error.{ Composition [ 1 ] Value.{ Integer x } }
```

## Budget

`Budget` bounds a composition: `remaining` nodes, a composition depth, and the
protos `ReaderBudget` for the parse. Every composition path spends, the bare
variant included.

## Anatomy

| module | what |
|---|---|
| `core` | the datom, the budget, the errors, the kinds, `Potential` |
| `composition` | protos to datom, the intrinsics, the scalars |
| `projection` | datom to protos; the writer computes the extents |
| `dropping` | iterative drop of the datom tree |

No free functions, no inherent impls, no zero-sized bearers: `nix flake check`
carries the guards, with build, test, fmt, clippy, doc and the generated
contract. Every walk is iterative.
