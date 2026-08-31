# collections

`collections` contains collection aliases shared across the workspace. The current
implementation re-exports standard-library collections along with `FxHashMap`, `FxHashSet`,
and `FxHasher` from `rustc_hash`; its local `HashMap` and `HashSet` aliases use the faster
hasher.

This crate intentionally contains no policy: it must not depend on GPUI or own application
state. Add an alias only when it meaningfully reduces repeated type noise across multiple
crates. If a public API relies on ordering or hash semantics, do not hide those important
constraints behind an alias.
