# datom-codec

The pure-data dialect on protos. Datom carries data, strictly typed, and its
whole work is serialization and deserialization. Schema-driven and positional:
the reader walks the expected type, writing is the exact reverse projection,
and all naming lives in the type; the text carries only the data.

```
{ Ada 1990 { “12 Rue de la Paix” Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }
```

## In and out

```rust
let person: Person = Potential::<Person>::from(text).actualize()?;   // may fault
let text: String = person.textualize();                              // cannot
```

`Potential<T>` is `protos::Potential<T, Datom>`; `actualize` is protosize,
conceive, incorporate. `Datomic::textualize` is conceive, protosize,
textualize.

## The concept

`Datom` is what a protoform means before a type is known: `Variant` (a head and
a body), `Struct`, `Vector`, `Text` (quoted), `Meaning` (parenthesized), `Word`
(bare; the position decides). Conception walks the situated protoform and
yields a situated datom, one situation node per datom, in the path convention
protos states: a variant's head is child 0 and its body child 1.

## Datomic

Every corporate type bears `Datomic`: `incorporate(site)` builds the value from
a `Site`, the datom at its situation; `conceive()` projects the value into a
datom. A reader takes the site as the one form its position declares:

```rust
impl Datomic for Reply {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        let v = site.variant()?;
        match v.name {
            "Accepted" => { let mut p = v.positions(2)?; Ok(Reply::Accepted(p.position()?, p.position()?)) }
            "Pending"  => { v.nothing()?; Ok(Reply::Pending) }
            other => Err(site.refuse(Problem::UnknownVariant(other.to_owned()))),
        }
    }
    fn conceive(&self) -> Datom { /* the reverse projection */ }
}
```

`Sited` reads a site as a struct (`positions`), a vector (`elements`), a variant,
a word or text; `Positional` reads each position as its type; `Carrying` and
`Headed` read a variant's body. Every fault raised below is placed under its
parent's index on the way up, so a `Corporate` or `Conceptual` fault carries a
`Locus`: the path from the root datom and the extent in the text.

Scalars bear `Worded`: read from one bare word, written to one. `Integer`,
`Decimal`, `Boolean` and every unit enum of the crate are worded; `Text` is
`protos::Text`, written bare when it is one run of plain and separator glyphs
and quoted otherwise; `Meaning` is text in parentheses. `Vec`, `Option`,
`Result` and `Box` of a Datomic are Datomic.

## Faults

`Fault::Structural` passes a protos fault through. `Conceptual` names a
structure with no datom form (`Formless`: an angled enclosure, a qualified head,
a chain with an enclosed body) or a text that is not one value. `Corporate`
names what the type refused: `Shape(expected, found)`, `Arity`,
`UnknownVariant`, `Value`. Faults are themselves datomic:

```
Corporate.{ { [ 1 ] { 4 5 } } Value.x }      ; [ 1 x ] as Vector<Integer>
```

## Anatomy

| module | what | kind |
|---|---|---|
| `anatomy` | the concept, the meaning, the faults | |
| `kinds` | the kinds | `Datomic`, `Worded`, `Sited`, `Positional`, `Counted`, `Carrying`, `Headed` |
| `site` | a datom at its situation, read as one form | `Sited`, `Positional`, `Carrying`, `Headed`, `Incorporable` |
| `conception` | situated protoform to situated datom | `Conceivable<Datom>` |
| `protosization` | datom to protoform; the writer computes the situation | `Protosizable`, `Textualizable` |
| `worded` | the scalars | `Worded` |
| `containers` | text, meaning, vector, option, result, box | `Datomic` |
| `faults` | the faults, pathed and datomic | `Pathed`, `Datomic` |
| `dropping` | iterative drop of the datom tree | |

No free functions, no inherent impls, no zero-sized bearers, no closures fed to
macros: `nix flake check` carries the guards, with build, test, fmt, clippy and
doc. Every walk is iterative; incorporation recurses only as deep as the
corporate type itself nests, which the text cannot exceed.
