# Datomic

Positional typed data over Protos. The datom dialect carries data,
strictly typed, and its whole work is serialization and deserialization.
Schema-driven and positional: the reader walks the expected type,
writing is the exact reverse projection.

## Forms

Struct (braces), Vector (brackets), Variant (headed with dot),
Text (curly-quoted or bare), Meaning (parentheses), Bare (bare symbol).

## The Datomic kind

Every corporate type bears `Datomic`, providing `incorporate_from`
(concept to corporate) and `textualize` (corporate to text through
the chain).
