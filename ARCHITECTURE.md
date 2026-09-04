# Architecture

## Four layers

| Layer | Type | Descent | Ascent |
|---|---|---|---|
| Text | `Text`, `Potential<T>` | `Structural::delineate` | -- |
| Protoform | `Protoform`, `Delineation` | `Conceptual<Datom>::conceive` | `Protosizable::protosize` (on Datom) |
| Concept | `Datom` | `Datomic::incorporate` | `Datomic::datomize` |
| Corporal | Rust value | -- | -- |

## Conceive rules (Protoform -> Datom)

```
; ethos examples of datom values
{ Ada 1990 }                    ; a struct of name Text, born Integer
[ Author Reviewer.{ 2024 17 } ] ; a vector of Role
Observed.Locks.[]               ; a variant chain
```

```rust
// The target Rust
struct Person(String, i64);
enum Role { Author, Reviewer(i64, i64) }
```

- Braced -> Struct(children)
- Bracketed -> Vector(children)
- Guillemets -> Map(pairs); odd count faults
- CurlyQuotes -> Text(content)
- Parentheses -> Meaning(content)
- Bare -> Bare(symbol)
- Headed -> Variant(head, separator, body)
- Angled -> fault (Shape)

## Incorporate rules (Datom -> T)

Positional: the reader walks the expected type.

- **Integer**: Bare, ASCII decimal, optional `-`, no `+`, no leading zero
- **Boolean**: Bare `True` / `False`
- **Decimal**: Bare or all-bare Variant chain rejoined; finite; must contain `.`
- **Text**: Text/Bare/all-bare chain rejoined (the bare-string rule)
- **Meaning**: Meaning -> Plain
- **Vec<T>**: Vector of T
- **BTreeMap<K,V>**: Map; DuplicateKey faults
- **Option<T>**: `None` / `Some.T`
- **Result<T,E>**: `Ok.T` / `Err.E`
- **Struct**: Struct with exact arity
- **Enum**: Bare for unit variant, `Variant.body` for data variant (Period only)
