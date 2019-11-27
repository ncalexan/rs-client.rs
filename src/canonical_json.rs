use serde_json;

pub fn serialize(input: &serde_json::value::Value) -> String {
  format!("{}", input)
}

#[cfg(test)]
mod test_serialize {
  use crate::canonical_json::serialize;
  use serde_json;
  use serde_json::json;

  #[test]
  fn test_empty_object() {
    let result = serialize(&json!({}));

    assert_eq!(result, "{}");
  }

  #[test]
  fn test_null_value() {
    let result = serialize(&json!({ "z": null }));

    assert_eq!(result, r#"{"z":null}"#);
  }

  #[test]
  fn test_keys_are_sorted() {
    let result = serialize(&json!({"z": 2, "a":1}));

    assert_eq!(result, r#"{"a":1,"z":2}"#);
  }

  #[test]
  fn test_recursive_objects() {
    let result = serialize(&json!({"a": {"c": 1, "b": 2}}));

    assert_eq!(result, r#"{"a":{"b":2,"c":1}}"#);
  }

  #[test]
  fn test_escape_values_and_keys() {
    let result = serialize(&json!({"key": "✓"}));
    assert_eq!(result, r#"{"key":"\\u2713"}"#);

    let result = serialize(&json!({"✓": 1}));
    assert_eq!(result, r#"{"\u2713":1}"#);
  }

  #[test]
  fn test_use_scientific_notation() {
    // https://searchfox.org/mozilla-central/source/toolkit/modules/tests/xpcshell/test_CanonicalJSON.js
    let inputs = vec![
      (0.01, "0.01"),
      (0.00099, "0.00099"),
      (0.000011, "0.000011"),
      (0.0000011, "0.0000011"),
      (0.000001, "0.000001"),
      (0.00000099, "9.9e-7"),
      (0.0000001, "1e-7"),
      (0.000000930258908, "9.30258908e-7"),
      (0.00000000000068272, "6.8272e-13"),
      (10f32.powi(20), "100000000000000000000"),
      (10f32.powi(21), "1e+21"),
      (10f32.powi(15) + 0.1, "1000000000000000.1"),
      (10f32.powi(16) * 1.1, "11000000000000000"),
    ];
    for (i, r) in inputs {
      let input = &json!({ "k": i });
      let result = format!(r#"{{"k":{}}}"#, r);
      assert_eq!(serialize(input), result);
    }
  }
}
