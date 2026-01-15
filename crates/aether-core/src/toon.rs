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
            Value::Bool(b) => if *b { "T" } else { "F" }.to_string(),
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
                let pad = "  ".repeat(indent);
                let mut out = format!("{}{{{}}}:\n", pad, keys.join(","));

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

    /// Deserialize a TOON string back into a JSON value.
    pub fn deserialize(input: &str) -> Result<Value, String> {
        let lines: Vec<&str> = input.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.is_empty() {
            return Ok(Value::Null);
        }

        Self::parse_level(&lines, 0).map(|(v, _)| v)
    }

    fn parse_level(lines: &[&str], start_idx: usize) -> Result<(Value, usize), String> {
        if start_idx >= lines.len() {
            return Ok((Value::Null, start_idx));
        }

        let first_line = lines[start_idx];
        let indent = first_line.chars().take_while(|c| c.is_whitespace()).count();
        let trimmed = first_line.trim();

        if trimmed.starts_with('{') && trimmed.contains("}:") {
            // Tabular format: {id,name}:
            return Self::parse_tabular(lines, start_idx, indent);
        }

        if trimmed.starts_with("- ") {
            // List format
            return Self::parse_list(lines, start_idx, indent);
        }

        // Object format (key: value)
        let mut map = Map::new();
        let mut idx = start_idx;

        while idx < lines.len() {
            let line = lines[idx];
            let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();
            
            if current_indent < indent {
                break;
            }
            if current_indent > indent {
                // This shouldn't happen in a well-formed object stream without a parent key
                idx += 1;
                continue;
            }

            let line_trimmed = line.trim();
            if let Some(colon_idx) = line_trimmed.find(':') {
                let mut key = line_trimmed[..colon_idx].trim().to_string();
                
                // Strip [len] suffix if present
                if let Some(bracket_idx) = key.find('[') {
                    if key.ends_with(']') {
                        key = key[..bracket_idx].to_string();
                    }
                }

                let val_part = line_trimmed[colon_idx + 1..].trim();

                if val_part.is_empty() && idx + 1 < lines.len() {
                    // Check if next line is more indented (nested object/array)
                    let next_indent = lines[idx + 1].chars().take_while(|c| c.is_whitespace()).count();
                    if next_indent > current_indent {
                        let (child_val, next_idx) = Self::parse_level(lines, idx + 1)?;
                        map.insert(key, child_val);
                        idx = next_idx;
                        continue;
                    }
                }
                
                map.insert(key, Self::parse_primitive(val_part));
                idx += 1;
            } else {
                idx += 1;
            }
        }

        Ok((Value::Object(map), idx))
    }

    fn parse_tabular(lines: &[&str], start_idx: usize, base_indent: usize) -> Result<(Value, usize), String> {
        let header = lines[start_idx].trim();
        let keys_str = header.trim_start_matches('{').trim_end_matches("}:");
        let keys: Vec<&str> = keys_str.split(',').map(|k| k.trim()).collect();
        
        let mut arr = Vec::new();
        let mut idx = start_idx + 1;

        while idx < lines.len() {
            let line = lines[idx];
            let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();
            if current_indent <= base_indent && !line.trim().is_empty() && idx != (start_idx + 1) {
                // We keep moving if it's the first line after header, otherwise check indent
                if current_indent < base_indent { break; }
            }

            let row_trimmed = line.trim();
            if row_trimmed.is_empty() { 
                idx += 1;
                continue; 
            }

            let values: Vec<Value> = row_trimmed.split(',')
                .map(|v| Self::parse_primitive(v.trim()))
                .collect();
            
            let mut obj = Map::new();
            for (i, key) in keys.iter().enumerate() {
                let val = values.get(i).cloned().unwrap_or(Value::Null);
                obj.insert(key.to_string(), val);
            }
            arr.push(Value::Object(obj));
            idx += 1;
        }

        Ok((Value::Array(arr), idx))
    }

    fn parse_list(lines: &[&str], start_idx: usize, base_indent: usize) -> Result<(Value, usize), String> {
        let mut arr = Vec::new();
        let mut idx = start_idx;

        while idx < lines.len() {
            let line = lines[idx];
            let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();
            if current_indent < base_indent {
                break;
            }

            let trimmed = line.trim();
            if trimmed.starts_with("- ") {
                arr.push(Self::parse_primitive(&trimmed[2..]));
            }
            idx += 1;
        }

        Ok((Value::Array(arr), idx))
    }

    fn parse_primitive(s: &str) -> Value {
        match s {
            "~" => Value::Null,
            "T" => Value::Bool(true),
            "F" => Value::Bool(false),
            _ => {
                if let Ok(n) = s.parse::<i64>() {
                    Value::Number(n.into())
                } else if let Ok(f) = s.parse::<f64>() {
                    if let Some(n) = serde_json::Number::from_f64(f) {
                        Value::Number(n)
                    } else {
                        Value::String(s.to_string())
                    }
                } else {
                    Value::String(s.replace("\\,", ",").to_string())
                }
            }
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

    #[test]
    fn test_toon_roundtrip() {
        let original = json!({
            "project": "Aether",
            "active": true,
            "version": 1,
            "null_val": null,
            "tags": ["ai", "rust", "security"],
            "files": [
                {"name": "main.rs", "size": 1024},
                {"name": "lib.rs", "size": 2048}
            ]
        });

        let serialized = Toon::serialize(&original);
        println!("Serialized TOON:\n{}", serialized);
        let deserialized = Toon::deserialize(&serialized).unwrap();

        // Note: Tabular conversion might lose some type info if not careful, 
        // but here it should match. Bool T/F is handled.
        assert_eq!(original["project"], deserialized["project"]);
        assert_eq!(deserialized["active"], json!(true));
        assert_eq!(deserialized["null_val"], Value::Null);
        assert_eq!(deserialized["tags"].as_array().unwrap().len(), 3);
        assert_eq!(deserialized["files"].as_array().unwrap().len(), 2);
    }
}
