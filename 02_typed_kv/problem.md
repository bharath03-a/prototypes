# Exercise 02 — Typed key-value store

## Goal
Extend your key-value store to hold typed values instead of plain strings.
This forces you to think about how Rust models "a value that could be one of several types."

## Problem: Design a typed key-value store

Your store must hold values of different types under the same key namespace.
Implement a `Value` enum with the following variants:

- `Text(String)` — a UTF-8 string
- `Number(f64)` — a floating point number
- `Boolean(bool)` — true or false
- `List(Vec<String>)` — an ordered list of strings

Then implement a `TypedKvStore` struct with these operations:

**`set(key, value)`** — Insert or update a key with a `Value`.

**`get(key)`** — Return a reference to the `Value` if it exists, or `None`.

**`get_text(key)`** — Return the inner `&str` if the value is `Text`, otherwise `None`.

**`get_number(key)`** — Return the inner `f64` if the value is `Number`, otherwise `None`.

**`get_boolean(key)`** — Return the inner `bool` if the value is `Boolean`, otherwise `None`.

**`get_list(key)`** — Return a reference to the inner `Vec<String>` if the value is `List`, otherwise `None`.

**`delete(key)`** — Remove the key, return `true` if it existed.

**`type_of(key)`** — Return a `&str` describing the type: `"text"`, `"number"`, `"boolean"`, `"list"`, or `"not found"`.

---

## Example
```text
set("name", Text("alice"))
set("age", Number(30.0))
set("active", Boolean(true))
set("tags", List(["rust", "db", "learning"]))

get("name")           // => Some(Text("alice"))
get("missing")        // => None

get_text("name")      // => Some("alice")
get_text("age")       // => None  (it's a Number, not Text)

get_number("age")     // => Some(30.0)
get_boolean("active") // => Some(true)
get_list("tags")      // => Some(["rust", "db", "learning"])

type_of("name")       // => "text"
type_of("age")        // => "number"
type_of("active")     // => "boolean"
type_of("tags")       // => "list"
type_of("missing")    // => "not found"

delete("name")        // => true
get("name")           // => None
```

---

## Constraints
- `get_text`, `get_number` etc. must return `None` if the key holds a different type — no panics
- `type_of` must never panic — even for missing keys
- Your `Value` enum must be the single source of truth for all type logic — no separate type tracking

---

## Bonus challenges
Attempt these after the base problem is working:

1. Implement `std::fmt::Display` for `Value` so you can `println!("{}", value)` and get human-readable output like `Text: alice`, `Number: 30`, `Boolean: true`, `List: [rust, db, learning]`
2. Add a `scan_by_type(type_name: &str) -> Vec<(&String, &Value)>` method that returns all keys holding a given type, sorted by key
3. Add a `update_number(key, delta: f64) -> Option<f64>` method that adds `delta` to an existing number value and returns the new value, or `None` if the key doesn't exist or isn't a number

---

## Hint
For `get_text` and friends, `match` is your tool:
```rust
match self.data.get(key) {
    Some(Value::Text(s)) => Some(s.as_str()),
    _ => None,
}
```

The `_` arm catches both "wrong type" and "missing key" in one go.

---

## Files
- `src/main.rs` — your implementation goes here
- `Cargo.toml` — no external dependencies needed for this exercise