use crate::parse::regex;

pub mod parse;
pub mod regex;


pub fn find(haystack: &str, pattern: &str) -> bool {
    regex(pattern).unwrap().exact(haystack).is_some()
}

#[cfg(test)]
mod test {
    use crate::find;

    #[test]
    fn literal_char_match() {
        assert!(find("apple", "a"))
    }

    #[test]
    fn literal_char_no_match() {
        assert!(!find("dog", "a"))
    }

    #[test]
    fn digit_match() {
        assert!(find("apple123", "\\d"))
    }

    #[test]
    fn digit_no_match() {
        assert!(!find("apple", "\\d"))
    }

    #[test]
    fn alphanumeric_match() {
        assert!(find("123apple123", "\\w"))
    }

    #[test]
    fn alphanumeric_no_match() {
        assert!(!find("$!?", "\\w"))
    }

    #[test]
    fn char_group_match() {
        assert!(find("apple", "[abc]"))
    }

    #[test]
    fn char_group_no_match() {
        assert!(!find("dog", "[abc]"))
    }

    #[test]
    fn negative_char_group_match() {
        assert!(find("dog", "[^abc]"))
    }

    #[test]
    fn negative_char_group_no_match() {
        assert!(!find("cab", "[^abc]"))
    }

    #[test]
    fn combining_1() {
        assert!(find("1 apple", "\\d apple"));
        assert!(!find("1 orange", "\\d apple"));
    }

    #[test]
    fn combining_2() {
        assert!(find("100 apple", "\\d\\d\\d apple"));
        assert!(!find("1 apple", "\\d\\d\\d apple"));
    }

    #[test]
    fn combining_3() {
        assert!(find("3 dogs", "\\d \\w\\w\\ws"));
        assert!(find("5 cats", "\\d \\w\\w\\ws"));
        assert!(!find("1 cat", "\\d \\w\\w\\ws"));
    }

    #[test]
    fn starts() {
        assert!(find("logs", "^log"));
        assert!(!find("slog", "^log"))
    }

    #[test]
    fn ends() {
        assert!(find("slog", "log$"));
        assert!(!find("logs", "log$"))
    }

    #[test]
    fn starts_ends() {
        assert!(find("log", "^log$"));
        assert!(!find("logs", "^log$"));
        assert!(!find("slog", "^log$"))
    }

    #[test]
    fn counters() {
        // ?
        assert!(find("", "^\\w?$"));
        assert!(find("a", "^\\w?$"));
        assert!(!find("aaaaa", "^\\w?$"));

        // +
        assert!(!find("", "^\\w+$"));
        assert!(find("a", "^\\w+$"));
        assert!(find("aaaaa", "^\\w+$"));

        // *
        assert!(find("", "^\\w*$"));
        assert!(find("a", "^\\w*$"));
        assert!(find("aaaaa", "^\\w*$"));
    }
}
