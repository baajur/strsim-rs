
//! This library implements string similarity metrics. Currently includes
//! Hamming, Levenshtein, Jaro, and Jaro-Winkler.

#![feature(test, core, collections)]

extern crate test;

use std::char;
use std::cmp::{max, min};
use std::collections::bit_vec::BitVec;

#[derive(Debug, PartialEq)]
pub enum StrSimError {
    DifferentLengthArgs
}

pub type HammingResult = Result<usize, StrSimError>;

/// Calculates the number of positions in the two strings where the characters
/// differ. Returns an error if the strings have different lengths.
///
/// ```
/// use strsim::hamming;
///
/// match hamming("hamming", "hammers") {
///     Ok(distance) => assert_eq!(3, distance),
///     Err(why) => panic!("{:?}", why)
/// }
/// ```
pub fn hamming(a: &str, b: &str) -> HammingResult {
    if a.len() != b.len() {
        Err(StrSimError::DifferentLengthArgs)
    } else {
        Ok(a.chars()
            .zip(b.chars())
            .filter(|&(a_char, b_char)| a_char != b_char)
            .count())
    }
}

/// Calculates the Jaro similarity between two strings. The returned value
/// is between 0.0 and 1.0 (higher value means more similar).
///
/// ```
/// use strsim::jaro;
/// use std::num::Float;
///
/// assert!((0.392 - jaro("Friedrich Nietzsche", "Jean-Paul Sartre")).abs() <
///         0.001);
/// ```
pub fn jaro(a: &str, b: &str) -> f64 {
    if a == b { return 1.0; }
    if a.len() == 0 || b.len() == 0 { return 0.0; }

    let search_range = max(0, (max(a.len(), b.len()) / 2) - 1);

    let mut b_consumed = BitVec::from_elem(b.len(), false);
    let mut matches = 0.0;

    let mut transpositions = 0.0;
    let mut b_match_index = 0;

    for (i, a_char) in a.chars().enumerate() {
        let min_bound =
            // prevent integer wrapping
            if i > search_range {
                max(0, i - search_range)
            } else {
                0
            };

        let max_bound = min(b.len() - 1, i + search_range);

        if min_bound > max_bound {
            continue;
        }

        for (j, b_char) in b.slice_chars(min_bound, max_bound + 1)
                            .chars()
                            .enumerate() {
            let index = j + min_bound;
            if a_char == b_char && !b_consumed[index] {
                b_consumed.set(index, true);
                matches += 1.0;

                if index < b_match_index {
                    transpositions += 1.0;
                }
                b_match_index = index;

                break;
            }
        }
    }

    if matches == 0.0 {
        0.0
    } else {
        (1.0 / 3.0) * ((matches / a.len() as f64) +
                       (matches / b.len() as f64) +
                       ((matches - transpositions) / matches))
    }
}

/// Like Jaro but gives a boost to strings that have a common prefix.
///
/// ```
/// use strsim::jaro_winkler;
/// use std::num::Float;
///
/// assert!((0.911 - jaro_winkler("cheeseburger", "cheese fries")).abs() <
///         0.001);
/// ```
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    let jaro_distance = jaro(a, b);

    // Don't limit the length of the common prefix
    let prefix_length = a.chars()
                         .zip(b.chars())
                         .take_while(|&(a_char, b_char)| a_char == b_char)
                         .count();

    let jaro_winkler_distance =
        jaro_distance + (0.1 * prefix_length as f64 * (1.0 - jaro_distance));

    if jaro_winkler_distance <= 1.0 {
        jaro_winkler_distance
    } else {
        1.0
    }
}

/// Calculates the minimum number of insertions, deletions, and substitutions
/// required to change one string into the other.
///
/// ```
/// use strsim::levenshtein;
///
/// assert_eq!(3, levenshtein("kitten", "sitting"));
/// ```
pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b { return 0; }
    else if a.len() == 0 { return b.len(); }
    else if b.len() == 0 { return a.len(); }

    let mut prev_distances: Vec<usize> = Vec::with_capacity(b.len() + 1);
    let mut curr_distances: Vec<usize> = Vec::with_capacity(b.len() + 1);

    for i in 0..(b.len() + 1) {
        prev_distances.push(i);
        curr_distances.push(0);
    }

    for (i, a_char) in a.chars().enumerate() {
        curr_distances[0] = i + 1;

        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            curr_distances[j + 1] = min(curr_distances[j] + 1,
                                        min(prev_distances[j + 1] + 1,
                                            prev_distances[j] + cost));
        }

        prev_distances.clone_from(&curr_distances);
    }

    curr_distances[b.len()]
}

/// Same as Levenshtein but allows for adjacent transpositions.
///
///
/// ```
/// use strsim::damerau_levenshtein;
///
/// assert_eq!(3, damerau_levenshtein("damerau", "aderua"));
/// ```
pub fn damerau_levenshtein(a: &str, b: &str) -> usize {
    if a == b { return 0; }
    else if a.len() == 0 { return b.len(); }
    else if b.len() == 0 { return a.len(); }

    let mut prev_two_distances: Vec<usize> = Vec::with_capacity(b.len() + 1);
    let mut prev_distances: Vec<usize> = Vec::with_capacity(b.len() + 1);
    let mut curr_distances: Vec<usize> = Vec::with_capacity(b.len() + 1);

    let mut prev_a_char = char::MAX;
    let mut prev_b_char = char::MAX;

    for i in 0..(b.len() + 1) {
        prev_two_distances.push(i);
        prev_distances.push(i);
        curr_distances.push(0);
    }

    for (i, a_char) in a.chars().enumerate() {
        curr_distances[0] = i + 1;

        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            curr_distances[j + 1] = min(curr_distances[j] + 1,
                                        min(prev_distances[j + 1] + 1,
                                            prev_distances[j] + cost));
            if i > 0 && j > 0 && a_char != b_char &&
               a_char == prev_b_char && b_char == prev_a_char {
                curr_distances[j + 1] = min(curr_distances[j + 1],
                                            prev_two_distances[j - 1] + 1);
            }

            prev_b_char = b_char;
        }

        prev_two_distances.clone_from(&prev_distances);
        prev_distances.clone_from(&curr_distances);
        prev_a_char = a_char;
    }

    curr_distances[b.len()]
 }

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use std::num::Float;

    #[test]
    fn hamming_empty() {
        match hamming("", "") {
            Ok(distance) => { assert_eq!(0, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn hamming_same() {
        match hamming("hamming", "hamming") {
            Ok(distance) => { assert_eq!(0, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn hamming_diff() {
        match hamming("hamming", "hammers") {
            Ok(distance) => { assert_eq!(3, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn hamming_unequal_length() {
        match hamming("ham", "hamming") {
            Ok(_) => { panic!(); },
            Err(why) => { assert_eq!(why, StrSimError::DifferentLengthArgs); }
        }
    }

    #[test]
    fn hamming_names() {
        match hamming("Friedrich Nietzs", "Jean-Paul Sartre") {
            Ok(distance) => { assert_eq!(14, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn jaro_both_empty() {
       assert_eq!(1.0, jaro("", ""));
    }

    #[test]
    fn jaro_first_empty() {
        assert_eq!(0.0, jaro("", "jaro"));
    }

    #[test]
    fn jaro_second_empty() {
        assert_eq!(0.0, jaro("distance", ""));
    }

    #[test]
    fn jaro_same() {
        assert_eq!(1.0, jaro("jaro", "jaro"));
    }

    #[test]
    fn jaro_diff_short() {
        assert!((0.767 - jaro("dixon", "dicksonx")).abs() < 0.001);
    }

    #[test]
    fn jaro_diff_no_transposition() {
        assert!((0.822 - jaro("dwayne", "duane")).abs() < 0.001);
    }

    #[test]
    fn jaro_diff_with_transposition() {
        assert!((0.944 - jaro("martha", "marhta")).abs() < 0.001);
    }

    #[test]
    fn jaro_names() {
        assert!((0.392 - jaro("Friedrich Nietzsche",
                              "Jean-Paul Sartre")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_both_empty() {
        assert_eq!(1.0, jaro_winkler("", ""));
    }

    #[test]
    fn jaro_winkler_first_empty() {
        assert_eq!(0.0, jaro_winkler("", "jaro-winkler"));
    }

    #[test]
    fn jaro_winkler_second_empty() {
        assert_eq!(0.0, jaro_winkler("distance", ""));
    }

    #[test]
    fn jaro_winkler_same() {
        assert_eq!(1.0, jaro_winkler("Jaro-Winkler", "Jaro-Winkler"));
    }

    #[test]
    fn jaro_winkler_diff_short() {
        assert!((0.813 - jaro_winkler("dixon", "dicksonx")).abs() < 0.001);
        assert!((0.813 - jaro_winkler("dicksonx", "dixon")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_diff_no_transposition() {
        assert!((0.840 - jaro_winkler("dwayne", "duane")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_diff_with_transposition() {
        assert!((0.961 - jaro_winkler("martha", "marhta")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_names() {
        assert!((0.562 - jaro_winkler("Friedrich Nietzsche",
                                      "Fran-Paul Sartre")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_long_prefix() {
        assert!((0.911 - jaro_winkler("cheeseburger", "cheese fries")).abs() <
                0.001);
    }

    #[test]
    fn jaro_winkler_more_names() {
        assert!((0.868 - jaro_winkler("Thorkel", "Thorgier")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_length_of_one() {
        assert!((0.738 - jaro_winkler("Dinsdale", "D")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_very_long_prefix() {
        assert!((1.0 - jaro_winkler("thequickbrownfoxjumpedoverx",
                                    "thequickbrownfoxjumpedovery")).abs() <
                0.001);
    }

    #[test]
    fn levenshtein_empty() {
        assert_eq!(0, levenshtein("", ""));
    }

    #[test]
    fn levenshtein_same() {
        assert_eq!(0, levenshtein("levenshtein", "levenshtein"));
    }

    #[test]
    fn levenshtein_diff_short() {
        assert_eq!(3, levenshtein("kitten", "sitting"));
    }

    #[test]
    fn levenshtein_diff_with_space() {
        assert_eq!(5, levenshtein("hello, world", "bye, world"));
    }

    #[test]
    fn levenshtein_diff_longer() {
        let a = "The quick brown fox jumped over the angry dog.";
        let b = "Lorem ipsum dolor sit amet, dicta latine an eam.";
        assert_eq!(37, levenshtein(a, b));
    }

    #[test]
    fn levenshtein_first_empty() {
        assert_eq!(7, levenshtein("", "sitting"));
    }

    #[test]
    fn levenshtein_second_empty() {
        assert_eq!(6, levenshtein("kitten", ""));
    }

    #[test]
    fn damerau_levenshtein_empty() {
        assert_eq!(0, damerau_levenshtein("", ""));
    }

    #[test]
    fn damerau_levenshtein_same() {
        assert_eq!(0, damerau_levenshtein("damerau", "damerau"));
    }

    #[test]
    fn damerau_levenshtein_first_empty() {
        assert_eq!(7, damerau_levenshtein("", "damerau"));
    }

    #[test]
    fn damerau_levenshtein_second_empty() {
        assert_eq!(7, damerau_levenshtein("damerau", ""));
    }

    #[test]
    fn damerau_levenshtein_diff_short() {
        assert_eq!(3, damerau_levenshtein("damerau", "aderua"));
    }

    #[test]
    fn damerau_levenshtein_diff_reversed() {
        assert_eq!(3, damerau_levenshtein("aderua", "damerau"));
    }

    #[test]
    fn damerau_levenshtein_diff_unequal_length() {
        assert_eq!(6, damerau_levenshtein("damerau", "aderuaxyz"));
    }

    #[test]
    fn damerau_levenshtein_diff_unequal_length_reversed() {
        assert_eq!(6, damerau_levenshtein("aderuaxyz", "damerau"));
    }

    #[test]
    fn damerau_levenshtein_diff_comedians() {
        assert_eq!(5, damerau_levenshtein("Stewart", "Colbert"));
    }

    #[test]
    fn damerau_levenshtein_many_transpositions() {
        assert_eq!(4, damerau_levenshtein("abcdefghijkl", "bacedfgihjlk"));
    }

    #[test]
    fn damerau_levenshtein_diff_longer() {
        let a = "The quick brown fox jumped over the angry dog.";
        let b = "Lehem ipsum dolor sit amet, dicta latine an eam.";
        assert_eq!(36, damerau_levenshtein(a, b));
    }

    #[test]
    fn damerau_levenshtein_beginning_transposition() {
        assert_eq!(1, damerau_levenshtein("foobar", "ofobar"));
    }

    #[test]
    fn damerau_levenshtein_end_transposition() {
        assert_eq!(1, damerau_levenshtein("specter", "spectre"));
    }

    #[bench]
    fn bench_hamming(b: &mut Bencher) {
        b.iter(|| hamming("Friedrich Nietzs", "Jean-Paul Sartre"));
    }

    #[bench]
    fn bench_levenshtein(b: &mut Bencher) {
        b.iter(|| levenshtein("Friedrich Nietzsche", "Jean-Paul Sartre"));
    }

    #[bench]
    fn bench_damerau_levenshtein(b: &mut Bencher) {
        b.iter(|| damerau_levenshtein("Friedrich Nietzsche",
                                      "Jirn-Paul Satere"));
    }

    #[bench]
    fn bench_jaro(b: &mut Bencher) {
        b.iter(|| jaro("Friedrich Nietzsche", "Jean-Paul Sartre"));
    }

    #[bench]
    fn bench_jaro_winkler(b: &mut Bencher) {
        b.iter(|| jaro_winkler("Friedrich Nietzsche", "Fran-Paul Sartre"));
    }
}
