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

/// Creates a `HashMap` from a list of key-value pairs.
///
/// # Examples
///
/// ```
/// use hashbrown::HashMap;
/// use pizza_common::hashmap;
///
/// let map = pizza_common::hashmap! { "a" => 1, "b" => 2 };
/// let mut expected = HashMap::new();
/// expected.insert("a", 1);
/// expected.insert("b", 2);
/// assert_eq!(map, expected);
/// ```

#[macro_export]
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map =  hashbrown::HashMap::with_capacity(_cap);
            $(
                _map.insert($key, $value);
            )*
            _map
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hashmap_macro() {
        let map = hashmap! { "a" => 1, "b" => 2 };
        let mut expected = hashbrown::HashMap::new();
        expected.insert("a", 1);
        expected.insert("b", 2);
        assert_eq!(map, expected);
    }
}
