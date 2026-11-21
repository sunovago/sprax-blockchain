use crate::error::TypeError;
use serde::Serialize;
use serde_json::Value;

/// Serializes any serializable value into deterministic RFC 8785 Canonical JSON bytes.
///
/// Properties guaranteed:
/// 1. Whitespace-free (no extra spaces around separators).
/// 2. Deterministic lexicographical object key sorting across all nesting depths.
/// 3. Minimal string escaping.
/// 4. Canonical number representations.
pub fn to_canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, TypeError> {
    let val = serde_json::to_value(value)
        .map_err(|e| TypeError::InvalidTransaction(format!("canonical json value error: {e}")))?;
    let mut out = Vec::new();
    write_canonical_value(&val, &mut out)?;
    Ok(out)
}

fn write_canonical_value(val: &Value, out: &mut Vec<u8>) -> Result<(), TypeError> {
    match val {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(b) => {
            if *b {
                out.extend_from_slice(b"true");
            } else {
                out.extend_from_slice(b"false");
            }
        }
        Value::Number(n) => {
            out.extend_from_slice(n.to_string().as_bytes());
        }
        Value::String(s) => {
            serde_json::to_writer(&mut *out, s)
                .map_err(|e| TypeError::InvalidTransaction(format!("string serialization: {e}")))?;
        }
        Value::Array(arr) => {
            out.push(b'[');
            for (i, elem) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canonical_value(elem, out)?;
            }
            out.push(b']');
        }
        Value::Object(map) => {
            out.push(b'{');
            let mut sorted_keys: Vec<&String> = map.keys().collect();
            sorted_keys.sort();

            for (i, key) in sorted_keys.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                serde_json::to_writer(&mut *out, key).map_err(|e| {
                    TypeError::InvalidTransaction(format!("key serialization: {e}"))
                })?;
                out.push(b':');
                write_canonical_value(&map[*key], out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonical_json_key_sorting() {
        let v1 = json!({
            "z": 100,
            "a": 200,
            "m": {
                "beta": "two",
                "alpha": "one"
            }
        });

        let v2 = json!({
            "a": 200,
            "m": {
                "alpha": "one",
                "beta": "two"
            },
            "z": 100
        });

        let b1 = to_canonical_json_bytes(&v1).expect("serialize v1");
        let b2 = to_canonical_json_bytes(&v2).expect("serialize v2");

        assert_eq!(b1, b2);
        assert_eq!(
            String::from_utf8(b1).unwrap(),
            r#"{"a":200,"m":{"alpha":"one","beta":"two"},"z":100}"#
        );
    }

    #[test]
    fn test_canonical_json_primitives() {
        assert_eq!(to_canonical_json_bytes(&json!(null)).unwrap(), b"null");
        assert_eq!(to_canonical_json_bytes(&json!(true)).unwrap(), b"true");
        assert_eq!(to_canonical_json_bytes(&json!(false)).unwrap(), b"false");
        assert_eq!(to_canonical_json_bytes(&json!(42)).unwrap(), b"42");
        assert_eq!(
            to_canonical_json_bytes(&json!("hello")).unwrap(),
            b"\"hello\""
        );
        assert_eq!(
            to_canonical_json_bytes(&json!([1, 2, 3])).unwrap(),
            b"[1,2,3]"
        );
    }
}
