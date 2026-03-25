use super::{TypedKvStore, Value};

#[test]
fn set_and_get_mixed_types() {
    let mut store = TypedKvStore::new();
    store.set("name", Value::Text("alice".to_string()));
    store.set("age", Value::Number(30.0));
    store.set("active", Value::Boolean(true));

    assert_eq!(
        store.get("name"),
        Some(&Value::Text("alice".to_string()))
    );
    assert_eq!(store.get("missing"), None);
}

#[test]
fn type_specific_getters_respect_types() {
    let mut store = TypedKvStore::new();
    store.set("name", Value::Text("alice".to_string()));
    store.set("age", Value::Number(30.0));
    store.set("active", Value::Boolean(true));
    store.set(
        "tags",
        Value::List(vec![
            "rust".to_string(),
            "db".to_string(),
            "learning".to_string(),
        ]),
    );

    assert_eq!(store.get_text("name"), Some("alice"));
    assert_eq!(store.get_text("age"), None);

    assert_eq!(store.get_number("age"), Some(30.0));
    assert_eq!(store.get_number("name"), None);

    assert_eq!(store.get_boolean("active"), Some(true));
    assert_eq!(store.get_boolean("name"), None);

    let list = store.get_list("tags").cloned();
    assert_eq!(
        list,
        Some(vec![
            "rust".to_string(),
            "db".to_string(),
            "learning".to_string()
        ])
    );

    assert_eq!(store.get_list("name"), None);
}

#[test]
fn delete_and_type_of_behave_correctly() {
    let mut store = TypedKvStore::new();
    store.set("name", Value::Text("alice".to_string()));

    assert_eq!(store.type_of("name"), "text");
    assert_eq!(store.type_of("missing"), "not found");

    assert!(store.delete("name"));
    assert!(!store.delete("name"));
    assert_eq!(store.type_of("name"), "not found");
}

#[test]
fn overwrite_changes_type_and_getters_follow() {
    let mut store = TypedKvStore::new();
    store.set("x", Value::Text("hi".to_string()));
    assert_eq!(store.type_of("x"), "text");
    assert_eq!(store.get_text("x"), Some("hi"));

    store.set("x", Value::Number(42.0));
    assert_eq!(store.type_of("x"), "number");
    assert_eq!(store.get_text("x"), None);
    assert_eq!(store.get_number("x"), Some(42.0));
}

#[test]
fn empty_list_is_supported() {
    let mut store = TypedKvStore::new();
    store.set("tags", Value::List(vec![]));

    assert_eq!(store.type_of("tags"), "list");
    let list = store.get_list("tags").cloned();
    assert_eq!(list, Some(vec![]));
}

