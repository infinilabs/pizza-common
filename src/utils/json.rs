// MIT License
//
// Copyright (C) INFINI Labs & INFINI LIMITED. <hello@infini.ltd>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use serde_json::Value;

/// Compares two JSON strings for equality.
///
/// # Arguments
///
/// * `json1` - The first JSON string to compare.
/// * `json2` - The second JSON string to compare.
///
/// # Returns
///
/// Returns `true` if the JSON strings are equal, and `false` otherwise.
///
/// # Examples
///
/// ```
/// use pizza_common::utils::json::compare_json;
/// let json1 = r#"{"name":"John","age":30}"#;
/// let json2 = r#"{"age":30,"name":"John"}"#;
///
/// assert_eq!(compare_json(json1, json2), true);
/// ```
pub fn compare_json(json1: &str, json2: &str) -> bool {
    let value1: Value = serde_json::from_str(json1).unwrap();
    let value2: Value = serde_json::from_str(json2).unwrap();
    value1 == value2
}

#[cfg(test)]
mod test {
    use crate::utils::json::compare_json;

    #[test]
    fn test_compare_json_equal() {
        let json1 = r#"{"name":"John","age":30}"#;
        let json2 = r#"{"age":30,"name":"John"}"#;
        assert_eq!(compare_json(json1, json2), true);
    }

    #[test]
    fn test_compare_json_not_equal() {
        let json1 = r#"{"name":"John","age":30}"#;
        let json2 = r#"{"name":"Jane","age":25}"#;
        assert_eq!(compare_json(json1, json2), false);
    }
}
