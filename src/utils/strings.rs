// MIT License
//
// Copyright (C) INFINI Labs & INFINI LIMITED.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

/// Removes the last occurrence of a specified character or substring from the input string.
///
/// If the input string contains the specified character or substring, this function removes
/// the last occurrence of it from the string. If the specified character or substring is not
/// found, the function returns the original string unchanged.
///
/// # Arguments
///
/// * `input_string` - The input string from which to remove the last occurrence of the specified character or substring.
/// * `find` - The character or substring to remove from the input string.
///
/// # Returns
///
/// A new `String` with the last occurrence of the specified character or substring removed, or the original string
/// if the specified character or substring is not found.
pub fn remove_suffix_str(input_string: &str, find: &str) -> String {
    // Find the index of the last occurrence of the specified character or substring
    if let Some(last_index) = input_string.rfind(find) {
        // If found, remove it using slicing
        let (left, right) = input_string.split_at(last_index);
        format!("{}{}", left, &right[find.len()..])
    } else {
        // If not found, return the original string
        input_string.to_string()
    }
}

/// Removes the specified prefix from the input string.
///
/// If the input string starts with the specified prefix, this function removes the prefix
/// from the string. If the input string does not start with the prefix, the function returns
/// the original string unchanged.
///
/// # Arguments
///
/// * `input_string` - The input string from which to remove the prefix.
/// * `prefix` - The prefix to remove from the input string.
///
/// # Returns
///
/// A new `String` with the prefix removed if it was present, or the original string
/// if the prefix was not found.
pub fn remove_prefix_str(input_string: &str, prefix: &str) -> String {
    if input_string.starts_with(prefix) {
        input_string[prefix.len()..].to_string()
    } else {
        input_string.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_suffix_str() {
        // Test case: string with '*' at the end
        let input_string = "abc*def*ghi*";
        let result = remove_suffix_str(input_string, "*");
        assert_eq!(result, "abc*def*ghi");

        // Test case: string with '*' in the middle
        let input_string = "abc*def*ghi";
        let result = remove_suffix_str(input_string, "*");
        assert_eq!(result, "abc*defghi");

        // Test case: string with no '*'
        let input_string = "abcdefghi";
        let result = remove_suffix_str(input_string, "*");
        assert_eq!(result, "abcdefghi");

        // Test case: empty string
        let input_string = "";
        let result = remove_suffix_str(input_string, "*");
        assert_eq!(result, "");
    }

    #[test]
    fn test_remove_prefix_str() {
        let s = "hello world";
        let result = remove_prefix_str(&s, "hello ");
        assert_eq!(result, "world");

        let s = "hello world";
        let result = remove_prefix_str(&s, "world");
        assert_eq!(result, "hello world");

        let s = "hello world";
        let result = remove_prefix_str(&s, "");
        assert_eq!(result, "hello world");
    }
}
