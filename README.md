# datomic

Positional typed data over Protos. The datom dialect: Concept
layer between Protoform and Corporate.

## Datom (the concept)

Variant, Struct, Vector, Text, Meaning, Bare. No map: a struct
when keys are fixed, a vector of structs when they are not.

## Datomic (the kind)

`incorporate` (Datom -> Self, may fault), `conceive` (&self -> Datom,
cannot fault), `textualize` (&self -> Text, chains through protos).
Borne by Integer, Decimal, Boolean, Text, Meaning, Vec, Option, Result.

## Ethos declaration

See `datomic.ethos`.
