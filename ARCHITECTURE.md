# Datomic architecture

Datomic is the positional typed-data dialect over Protos. Protos owns the
only text delineator and printer. Datomic owns the expected-type anatomy that
maps one received `Portion` to an embodied value and maps an invariant-bearing
embodied value back to a `Portion`.

`Text<T>` is the D4 edge. Its inbound `TextEdge<T>` delineates once through
Protos then asks `T` for its anatomy; failure remains attached to the relevant
Protos extent. `Datomic::textualize` hands the constructed Portion to the
Protos writer and returns that writer's typed canonical Text directly.

Scalars use Protos's expected-scalar boundary. Datomic does not inspect
digits, decimal points, exponent notation, or numeric ranges. A finite decimal
is an invariant-bearing Datomic type, so non-finite floating values cannot
enter an outbound anatomy. Likewise a Datomic string is representable before
it is stored: no-escape, asymmetric curly-quote balance is checked at creation
rather than during textualization.

Records consume structural children by position. Heads belong to variants and
re-emit unchanged. Vectors are bracketed, maps are headless guillemet blocks
with alternating keys and values, duplicate keys fault at the duplicate key
extent, and options are `None` or `Some.value`.
