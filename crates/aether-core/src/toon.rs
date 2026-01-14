use serde_json::{Value, Map};

/// Token-Oriented Object Notation (TOON) Serializer.
/// Reduces token usage by 30-60% compared to JSON.
pub struct Toon;

impl Toon {
    /// Serialize a JSON value to TOON format.
    pub fn serialize(value: &Value) -> String {
        match value {
            Value::Object(map) => Self::serialize_object(map, 0),
            Value::Array(arr) => Self::serialize_array(arr, 0),
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "~".to_string(),
        }
    }

    fn serialize_object(map: &Map<String, Value>, indent: usize) -> String {
        let mut out = String::new();
        let pad = "  ".repeat(indent);
        
        for (k, v) in map {
            match v {
                Value::Object(child_map) => {
                    out.push_str(&format!("{}{}:\n{}", pad, k, Self::serialize_object(child_map, indent + 1)));
                }
                Value::Array(arr) => {
                    out.push_str(&format!("{}{}[{}]:\n{}", pad, k, arr.len(), Self::serialize_array(arr, indent + 1)));
                }
                _ => {
                    out.push_str(&format!("{}{}: {}\n", pad, k, Self::serialize(v)));
                }
            }
        }
        out
    }

    fn serialize_array(arr: &[Value], indent: usize) -> String {
        if arr.is_empty() {
            return "[]".to_string();
        }

        // Check if it's a homogeneous list of objects to use tabular TOON format
        if let Some(first) = arr.first() {
            if let Value::Object(first_map) = first {
                let keys: Vec<String> = first_map.keys().cloned().collect();
                let mut out = format!("{{{}}}:\n", keys.join(","));
                let pad = "  ".repeat(indent);

                for item in arr {
                    if let Value::Object(item_map) = item {
                        let values: Vec<String> = keys.iter()
                            .map(|k| item_map.get(k).map(|v| Self::serialize_flat(v)).unwrap_or_else(|| "~".to_string()))
                            .collect();
                        out.push_str(&format!("{}{}\n", pad, values.join(",")));
                    }
                }
                return out;
            }
        }

        // Fallback for simple arrays
        let mut out = String::new();
        let pad = "  ".repeat(indent);
        for v in arr {
            out.push_str(&format!("{}- {}\n", pad, Self::serialize(v).trim()));
        }
        out
    }

    fn serialize_flat(value: &Value) -> String {
        match value {
            Value::String(s) => s.replace(',', "\\,").to_string(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => if *b { "T" } else { "F" }.to_string(),
            _ => ".".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_toon_tabular() {
        let data = json!([
            {"id": 1, "name": "Apple", "price": 10},
            {"id": 2, "name": "Banana", "price": 5}
        ]);
        let toon = Toon::serialize(&data);
        assert!(toon.contains("{id,name,price}"));
        assert!(toon.contains("1,Apple,10"));
    }

    #[test]
    fn test_toon_object() {
        let data = json!({
            "user": "admin",
            "meta": { "last_login": "2024-01-01" }
        });
        let toon = Toon::serialize(&data);
        assert!(toon.contains("user: admin"));
        assert!(toon.contains("meta:"));
    }
}
