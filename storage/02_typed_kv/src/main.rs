use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Value {
    Text(String),
    Number(f64),
    Boolean(bool),
    List(Vec<String>),
}

struct TypedKvStore {
    data: HashMap<String, Value>,
}

impl TypedKvStore {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn set(&mut self, key: &str, value: Value) {
        self.data.insert(key.to_string(), value);
    }

    fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    fn get_text(&self, key: &str) -> Option<&str> {
        match self.data.get(key) {
            Some(Value::Text(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        match self.data.get(key) {
            Some(Value::Number(n)) => Some(*n),
            _ => None,
        }
    }

    fn get_boolean(&self, key: &str) -> Option<bool> {
        match self.data.get(key) {
            Some(Value::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    fn get_list(&self, key: &str) -> Option<&Vec<String>> {
        match self.data.get(key) {
            Some(Value::List(list)) => Some(list),
            _ => None,
        }
    }

    fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    fn type_of(&self, key: &str) -> &str {
        match self.data.get(key) {
            Some(Value::Text(_)) => "text",
            Some(Value::Number(_)) => "number",
            Some(Value::Boolean(_)) => "boolean",
            Some(Value::List(_)) => "list",
            None => "not found",
        }
    }
}

fn main() {
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

    println!("get(name) = {:?}", store.get("name"));
    println!("get(missing) = {:?}", store.get("missing"));

    println!("get_text(name) = {:?}", store.get_text("name"));
    println!("get_text(age) = {:?}", store.get_text("age"));

    println!("get_number(age) = {:?}", store.get_number("age"));
    println!("get_boolean(active) = {:?}", store.get_boolean("active"));
    println!("get_list(tags) = {:?}", store.get_list("tags"));

    println!("type_of(name) = {}", store.type_of("name"));
    println!("type_of(age) = {}", store.type_of("age"));
    println!("type_of(active) = {}", store.type_of("active"));
    println!("type_of(tags) = {}", store.type_of("tags"));
    println!("type_of(missing) = {}", store.type_of("missing"));

    println!("delete(name) = {}", store.delete("name"));
    println!("get(name) after delete = {:?}", store.get("name"));
}

#[cfg(test)]
mod typed_kv_tests;
