use std::collections::HashMap;

struct KvStore {
    store: HashMap<String, String>,
}

impl KvStore {
    fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    // set key-value pair
    fn set(&mut self, key: &str, value: &str) {
        assert!(!key.is_empty(), "key must be non-empty");
        assert!(!value.is_empty(), "value must be non-empty");
        self.store.insert(key.to_string(), value.to_string());
    }

    // get value by key
    fn get(&self, key: &str) -> Option<&String> {
        self.store.get(key)
    }

    // delete key-value pair
    fn delete(&mut self, key: &str) -> bool {
        self.store.remove(key).is_some()
    }

    // scan keys by prefix
    fn scan(&self, prefix: &str) -> Vec<(&String, &String)> {
        let mut items: Vec<(&String, &String)> = self
            .store
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .collect();
            
        items.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));
        items
    }

    // count keys by prefix
    fn count(&self, prefix: &str) -> usize {
        self.store.iter().filter(|(k, _)| k.starts_with(prefix)).count()
    }
}

fn main() {
    let mut kv = KvStore::new();

    kv.set("user:1", "alice");
    kv.set("user:2", "bob");
    kv.set("user:3", "carol");
    kv.set("config:theme", "dark");
    kv.set("config:lang", "en");

    println!("get(user:1) = {:?}", kv.get("user:1"));
    println!("get(user:99) = {:?}", kv.get("user:99"));

    println!("scan(user:) = {:?}", kv.scan("user:"));
    println!("scan(config:) = {:?}", kv.scan("config:"));
    println!("scan(x:) = {:?}", kv.scan("x:"));

    println!("delete(user:2) = {}", kv.delete("user:2"));
    println!("delete(user:99) = {}", kv.delete("user:99"));

    println!("count(user:) = {}", kv.count("user:"));
    println!("count(config:) = {}", kv.count("config:"));
}

#[cfg(test)]
mod kv_store_tests;
