/// Calculates levenshtein distance between two strings, returns the {{{1
/// distance as usize.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let length_a = a.chars().count();
    let length_b = b.chars().count();
    let mut result = 0;

    // Shortcut optimizations / degenerate cases.
    if a == b {
        return result;
    } else if length_a == 0 {
        return length_b;
    } else if length_b == 0 {
        return length_a;
    }

    // Initialize the vector.
    // This is why it’s fast, normally a matrix is used,
    // here we use a single vector.
    let mut cache: Vec<usize> = vec![0; length_a];
    let mut index_a = 0;
    let mut distance_a;
    let mut distance_b;

    while index_a < length_a {
        index_a += 1;
        cache[index_a - 1] = index_a;
    }

    for (index_b, code_b) in b.chars().enumerate() {
        result = index_b;
        distance_a = index_b;

        for (index_a, code_a) in a.chars().enumerate() {
            distance_b = if code_a == code_b {
                distance_a
            } else {
                distance_a + 1
            };

            distance_a = cache[index_a];

            result = if distance_a > result {
                if distance_b > result {
                    result + 1
                } else {
                    distance_b
                }
            } else {
                if distance_b > distance_a {
                    distance_a + 1
                } else {
                    distance_b
                }
            };

            cache[index_a] = result;
        }
    }

    result
}

/// Calculates levenshtein distance, but ignores case and some non-alphanumeric {{{1
/// characters, such as apostrophes, dashes, and underscores.
pub fn levenshtein_insensitive(a: &str, b: &str) -> usize {
    let a_fixed = a.to_lowercase()
        .replace("'", "")
        .replace("-", " ")
        .replace("_", " ");
    let b_fixed = b.to_lowercase()
        .replace("'", "")
        .replace("-", " ")
        .replace("_", " ");

    levenshtein(&a_fixed, &b_fixed)
}

// Tests {{{1
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_zero_distance() {
        let result = levenshtein("test", "test");
        assert_eq!(result, 0);
    }

    #[test]
    fn levenshtein_distance_calc() {
        let result = levenshtein("some string", "completely different");
        assert_eq!(result, 14);
    }

    #[test]
    fn levenshtein_insensitive_exact() {
        let result = levenshtein_insensitive("some string", "so'me_stRIng");
        assert_eq!(result, 0);
    }

    #[test]
    fn levenshtein_insensitive_calc() {
        let result = levenshtein_insensitive("some str'ing", "com'pleTELY-DIFf'erent");
        assert_eq!(result, 14);
    }
}
