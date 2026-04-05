# Exercise 01 — In-memory key-value store

## Goal

Implement an in-memory key-value store in Rust from scratch.

## Problem: Design an in-memory key-value store

Implement a `KvStore` struct that supports the following operations:

**`set(key, value)`** — Insert or update the key with the given value.

**`get(key)`** — Return the value for the key. If the key does not exist, return `None`.

**`delete(key)`** — Remove the key. Return `true` if it existed, `false` if it didn't.

**`scan(prefix)`** — Return all key-value pairs where the key starts with the given prefix, sorted by key alphabetically.

**`count(prefix)`** — Return the number of keys that start with the given prefix.

---

## Example

```text
set("user:1", "alice")
set("user:2", "bob")
set("user:3", "carol")
set("config:theme", "dark")
set("config:lang", "en")

get("user:1")         // => Some("alice")
get("user:99")        // => None

scan("user:")         // => [("user:1","alice"), ("user:2","bob"), ("user:3","carol")]
scan("config:")       // => [("config:lang","en"), ("config:theme","dark")]
scan("x:")            // => []

delete("user:2")      // => true
delete("user:99")     // => false

count("user:")        // => 2  (user:2 was deleted)
count("config:")      // => 2
```

---

## Constraints

- All keys and values are non-empty strings
- Keys follow the format `"namespace:identifier"` but your code should not assume this — `scan` must work on any prefix string
- `scan` results must be sorted alphabetically by key
- After a `delete`, that key must not appear in `scan` or `count`

---

## Bonus challenges

Attempt these after the base problem is working:

1. Add a `set_many(pairs: Vec<(&str, &str)>)` method that inserts multiple keys at once
2. Add a `delete_prefix(prefix: &str) -> usize` method that deletes all keys with the given prefix and returns how many were deleted
3. Add a `keys() -> Vec<&String>` method that returns all keys sorted alphabetically

---

## Hint

For `scan`, think about what iterator methods are available on a `HashMap`:
`.iter()` → `.filter()` → `.collect()`. Sorting needs one extra step after collecting.

---

## Files

- `src/main.rs` — your implementation goes here
- `Cargo.toml` — no external dependencies needed for this exercise
