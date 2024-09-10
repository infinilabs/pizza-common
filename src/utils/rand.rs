// MIT License
//
// Copyright (C) INFINI Labs & INFINI LIMITED.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
use alloc::borrow::ToOwned;
use alloc::string::String;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rand_core::RngCore;
use rand_core::SeedableRng;
// random name seeds
const HERO_NAMES: [&str; 40] = [
    "Spider-Man",
    "Iron Man",
    "Captain America",
    "Thor",
    "Hulk",
    "Black Widow",
    "Doctor Strange",
    "Black Panther",
    "Wolverine",
    "Captain Marvel",
    "Ant-Man",
    "Deadpool",
    "Scarlet Witch",
    "Vision",
    "Hawkeye",
    "Falcon",
    "Winter Soldier",
    "Star-Lord",
    "Gamora",
    "Drax the Destroyer",
    "Spider-Woman",
    "Iron Fist",
    "Luke Cage",
    "Daredevil",
    "Jessica Jones",
    "Punisher",
    "Cable",
    "Jean Grey",
    "Cyclops",
    "Storm",
    "Rogue",
    "Nightcrawler",
    "Professor X",
    "Beast",
    "Iceman",
    "Ghost Rider",
    "Blade",
    "Silver Surfer",
    "Quicksilver",
    "Scarlet Spider",
];

/// Generate random names
pub fn generate_name() -> &'static str {
    let mut rng = ChaCha8Rng::seed_from_u64(1234);
    HERO_NAMES.choose(&mut rng).unwrap_or(&"Unknown")
}

/// Generate uuid
pub fn generate_uuid() -> String {
    super::uuid::Uuid::new().encode_with(ToOwned::to_owned)
}

pub fn generate_random_u32(min: u32, max: u32) -> u32 {
    let mut rng = ChaCha8Rng::seed_from_u64(1234);
    rng.next_u32() % (max - min) + min
}

/// Generate a random string with space-separated words of random lengths.
///
/// # Parameters
/// - `word_count_range`: A tuple representing the range of the number of words.
/// - `word_length_range`: A tuple representing the range of the length of each word.
///
/// # Returns
/// A random string with space-separated words.
///
/// # Parameters
/// - `word_count_range`: A tuple representing the range of the number of words.
/// - `word_length_range`: A tuple representing the range of the length of each word.
///
/// # Returns
/// A random string with space-separated words.
pub fn generate_random_string(
    word_count_range: (usize, usize),
    word_length_range: (usize, usize),
) -> String {
    let mut rng = ChaCha8Rng::seed_from_u64(1234);

    // Generate random word count
    let word_count = word_count_range.0
        + (rng.next_u32() as usize % (word_count_range.1 - word_count_range.0 + 1));

    // Preallocate the space for the final string
    let mut result = String::with_capacity(word_count * word_length_range.1);

    for i in 0..word_count {
        let word_length = word_length_range.0
            + (rng.next_u32() as usize % (word_length_range.1 - word_length_range.0 + 1));
        let word: String = (0..word_length)
            .map(|_| {
                let rand_char = rng.next_u32() as u8 % 26 + b'a'; // Generate a random character between 'a' and 'z'
                rand_char as char
            })
            .collect();

        if i > 0 {
            result.push(' '); // Add space between words
        }
        result.push_str(&word);
    }

    result
}
