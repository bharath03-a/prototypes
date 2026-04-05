use super::KvStore;

#[test]
fn set_and_get_existing_and_missing_key() {
    let mut kv = KvStore::new();
    kv.set("user:1", "alice");

    assert_eq!(kv.get("user:1"), Some(&"alice".to_string()));
    assert_eq!(kv.get("user:99"), None);
}

#[test]
fn delete_returns_correct_status() {
    let mut kv = KvStore::new();
    kv.set("user:1", "alice");

    assert!(kv.delete("user:1"));
    assert!(!kv.delete("user:1"));
}

#[test]
fn scan_is_sorted_by_key() {
    let mut kv = KvStore::new();
    kv.set("user:2", "bob");
    kv.set("user:1", "alice");
    kv.set("user:3", "carol");

    let rows = kv.scan("user:");
    let pairs: Vec<(String, String)> = rows
        .iter()
        .map(|(k, v)| ((*k).clone(), (*v).clone()))
        .collect();

    assert_eq!(
        pairs,
        vec![
            ("user:1".to_string(), "alice".to_string()),
            ("user:2".to_string(), "bob".to_string()),
            ("user:3".to_string(), "carol".to_string()),
        ]
    );
}

#[test]
fn count_respects_deletes() {
    let mut kv = KvStore::new();
    kv.set("user:1", "alice");
    kv.set("user:2", "bob");
    kv.set("config:theme", "dark");

    assert_eq!(kv.count("user:"), 2);
    assert!(kv.delete("user:2"));
    assert_eq!(kv.count("user:"), 1);
    assert_eq!(kv.count("config:"), 1);
}

#[test]
fn set_overwrites_existing_key() {
    let mut kv = KvStore::new();
    kv.set("k", "v1");
    kv.set("k", "v2");

    assert_eq!(kv.get("k"), Some(&"v2".to_string()));
    assert_eq!(kv.count("k"), 1);
}

#[test]
fn scan_with_empty_prefix_returns_all_sorted() {
    let mut kv = KvStore::new();
    kv.set("b", "2");
    kv.set("a", "1");
    kv.set("c", "3");

    let rows = kv.scan("");
    let keys: Vec<String> = rows.iter().map(|(k, _)| (*k).clone()).collect();
    assert_eq!(keys, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
}

#[test]
fn delete_then_set_works() {
    let mut kv = KvStore::new();
    kv.set("user:1", "alice");
    assert!(kv.delete("user:1"));

    kv.set("user:1", "alice2");
    assert_eq!(kv.get("user:1"), Some(&"alice2".to_string()));
    assert_eq!(kv.count("user:"), 1);
}

#[test]
#[should_panic(expected = "key must be non-empty")]
fn set_panics_on_empty_key() {
    let mut kv = KvStore::new();
    kv.set("", "value");
}

#[test]
#[should_panic(expected = "value must be non-empty")]
fn set_panics_on_empty_value() {
    let mut kv = KvStore::new();
    kv.set("key", "");
}
