# datomic

Datomic: positional typed data over Protos Protoform. The datom
dialect: Concept layer between Protoform and Corporal.

## What datomic is

Datom is the most advanced textual data format in the world. It
carries data, strictly typed, super dense, no field names. Schema-
driven and positional: the reader walks the expected type, writing
is the exact reverse projection, and decoding lands directly in
the typed Rust structs.

## Ethos declaration

See `datomic.ethos` at the repository root.

## The Datom concept

A Datom is the concept type: Variant, Struct, Vector, Map, Text,
Meaning, or Bare. It sits between Protoform (structural) and
Corporal (the Rust value). Conceive lifts a Protoform to a Datom;
incorporate descends a Datom to a typed value; datomize ascends a
typed value to a Datom.

## Datomic (the kind)

The corporal kind: `incorporate` (Datom -> Self) and `datomize`
(&self -> Datom). Implemented for Integer, Decimal, Boolean, Text,
MeaningValue, Vec, BTreeMap, Option, Result. `Textualizable` is
provided for every Datomic.

## The bare-string rule

In a Text position, a Bare or an all-bare Variant chain is accepted
and rejoined: `name:first`, `2026-09-03T17:46:20`, `http://x` are
bare strings. Datomize writes bare when the word alone delineates
and rejoins to itself; otherwise curly-quoted.

## Meaning

Parenthesized text lands as `MeaningValue::Plain` today. The full
Meaning type (structured string with annotations) is future work.
The escape for an unbalanced parenthesis is not yet designed; the
proptest keeps content balanced and this is stated here.

## Faults

Three layers: Structural (protos fault), Conceptual (path + problem),
Corporal (path + problem). Actualize joins a fault's path to the
delineation's situation into a Situated fault with an extent.
