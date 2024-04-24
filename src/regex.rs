#[derive(Debug, Eq, PartialEq)]
pub enum Regex {
    Noop,
    Digit,
    Alphanumeric,
    Wildcard,
    Whitespace,
    Tag { c: char },
    Any { cs: Vec<char> },
    NotAny { cs: Vec<char> },
    AtLeastOne(Box<Regex>, Box<Regex>),
    ZeroOrOne(Box<Regex>, Box<Regex>),
    ZeroOrMany(Box<Regex>, Box<Regex>),
    Quantifier(usize, usize, Box<Regex>, Box<Regex>),
    Or(Box<Regex>, Box<Regex>),
    And(Box<Regex>, Box<Regex>),
    Somewhere(Box<Regex>),
    Starts(Box<Regex>),
    Ends,
}

impl Regex {
    pub fn exact<'a>(&self, haystack: &'a str) -> Option<&'a str> {
        match self {
            Regex::Noop => Some(haystack),
            Regex::Digit => Self::match_char(|c| c.is_ascii_digit(), haystack),
            Regex::Alphanumeric => Self::match_char(|c| c.is_alphanumeric(), haystack),
            Regex::Wildcard => Self::match_char(|_| true, haystack),
            Regex::Whitespace => Self::match_char(|c| c.is_whitespace(), haystack),
            Regex::Tag { c } => Self::match_char(move |x| x == *c, haystack),
            Regex::Any { cs } => Self::match_char(move |x| cs.contains(&x), haystack),
            Regex::NotAny { cs } => Self::match_char(move |x| !cs.contains(&x), haystack),
            Regex::AtLeastOne(inner, follow) => self.match_at_least_one(haystack, inner, follow),
            Regex::ZeroOrOne(inner, follow) => Self::match_zero_or_one(haystack, inner, follow),
            Regex::ZeroOrMany(inner, follow) => self.match_at_zero_or_many(haystack, inner, follow),
            Regex::Quantifier(min, max, inner, follow) => {
                todo!("implement quantifier")
            }
            Regex::Or(left, right) => Self::match_or(haystack, left, right),
            Regex::And(left, right) => Self::match_and(haystack, left, right),
            Regex::Ends => Self::match_ends(haystack),
            // todo: should be methods?
            Regex::Somewhere(inner) => self.match_somewhere(haystack, inner),
            Regex::Starts(inner) => inner.exact(haystack),
        }
    }

    fn match_at_least_one<'a>(
        &self,
        haystack: &'a str,
        inner: &Regex,
        follow: &Regex,
    ) -> Option<&'a str> {
        match inner.exact(haystack) {
            None => None,
            Some(next) => match self.exact(next) {
                None => follow.exact(next),
                Some(next) => Some(next),
            },
        }
    }

    fn match_at_zero_or_many<'a>(
        &self,
        haystack: &'a str,
        inner: &Regex,
        follow: &Regex,
    ) -> Option<&'a str> {
        match inner.exact(haystack) {
            None => follow.exact(haystack),
            Some(next) => self.exact(next)
        }
    }

    fn match_zero_or_one<'a>(haystack: &'a str, inner: &Regex, follow: &Regex) -> Option<&'a str> {
        match inner.exact(haystack) {
            None => follow.exact(haystack),
            Some(next) => match follow.exact(next) {
                None => follow.exact(haystack),
                Some(next) => Some(next),
            },
        }
    }

    fn match_ends(haystack: &str) -> Option<&str> {
        haystack.is_empty().then_some(haystack)
    }

    fn match_somewhere<'a>(&self, haystack: &'a str, inner: &Regex) -> Option<&'a str> {
        inner.exact(haystack).or_else(|| {
            if haystack.is_empty() {
                None
            } else {
                self.exact(&haystack[1..])
            }
        })
    }

    fn match_and<'a>(haystack: &'a str, left: &Regex, right: &Regex) -> Option<&'a str> {
        left.exact(haystack).and_then(|next| right.exact(next))
    }

    fn match_or<'a>(haystack: &'a str, left: &Regex, right: &Regex) -> Option<&'a str> {
        left.exact(haystack).or_else(|| right.exact(haystack))
    }

    fn match_char<F>(condition: F, haystack: &str) -> Option<&str>
        where
            F: Fn(char) -> bool,
    {
        haystack
            .chars()
            .next()
            .and_then(|c| condition(c).then_some(&haystack[1..]))
    }
}

#[cfg(test)]
mod tests {
    use crate::regex::Regex;
    use crate::regex::Regex::Somewhere;

    #[test]
    fn test_digit_matcher() {
        let digit = Regex::Digit;
        assert_eq!(digit.exact("123"), Some("23"));
        assert_eq!(digit.exact("abc"), None);
    }

    #[test]
    fn test_alphanumeric_matcher() {
        let alphanumeric = Regex::Alphanumeric;
        assert_eq!(alphanumeric.exact("abc123"), Some("bc123"));
        assert_eq!(alphanumeric.exact("!@#"), None);
    }

    #[test]
    fn test_tag_matcher() {
        let tag_a = Regex::Tag { c: 'a' };
        assert_eq!(tag_a.exact("abc"), Some("bc"));
        assert_eq!(tag_a.exact("123"), None);
    }

    #[test]
    fn test_wildcard_matcher() {
        let wildcard = Regex::Wildcard;
        assert_eq!(wildcard.exact("abc123"), Some("bc123"));
        assert_eq!(wildcard.exact(""), None);
    }

    #[test]
    fn test_any_matcher() {
        let any = Regex::Any {
            cs: vec!['a', 'b', 'c'],
        };
        assert_eq!(any.exact("abc123"), Some("bc123"));
        assert_eq!(any.exact("123"), None);
    }

    #[test]
    fn test_not_any_matcher() {
        let not_any = Regex::NotAny {
            cs: vec!['x', 'y', 'z'],
        };
        assert_eq!(not_any.exact("abc123"), Some("bc123"));
        assert_eq!(not_any.exact("xyz"), None);
    }

    #[test]
    fn test_at_least_one_matcher() {
        let at_least_one = Regex::AtLeastOne(Box::new(Regex::Digit), Box::new(Regex::Noop));
        assert_eq!(at_least_one.exact("123abc"), Some("abc"));
        assert_eq!(at_least_one.exact("abc123"), None);
    }

    #[test]
    fn test_at_least_one_and_matcher() {
        let at_least_one =
            Regex::AtLeastOne(Box::new(Regex::Digit), Box::new(Regex::Tag { c: '1' }));
        assert_eq!(at_least_one.exact("12345671"), Some(""));
        assert_eq!(at_least_one.exact("1"), None);
    }

    #[test]
    fn test_zero_or_one_matcher() {
        let zero_or_one = Regex::ZeroOrOne(Box::new(Regex::Digit), Box::new(Regex::Noop));
        assert_eq!(zero_or_one.exact("123abc"), Some("23abc"));
        assert_eq!(zero_or_one.exact("abc123"), Some("abc123"));
    }

    #[test]
    fn test_or_matcher() {
        let or = Regex::Or(Box::new(Regex::Digit), Box::new(Regex::Tag { c: 'a' }));
        assert_eq!(or.exact("123abc"), Some("23abc"));
        assert_eq!(or.exact("abc123"), Some("bc123"));
        assert_eq!(or.exact("xyz"), None);
    }

    #[test]
    fn test_and_matcher() {
        let and = Regex::And(Box::new(Regex::Digit), Box::new(Regex::Tag { c: 'a' }));
        assert_eq!(and.exact("a123"), None);
        assert_eq!(and.exact("123abc"), None);
        assert_eq!(and.exact("abc"), None);
        assert_eq!(and.exact("1abc"), Some("bc"));
    }

    #[test]
    fn test_ends_matcher() {
        let ends = Regex::Ends;
        assert_eq!(ends.exact(""), Some(""));
        assert_eq!(ends.exact("abc"), None);
    }

    #[test]
    fn test_somewhere_matcher() {
        let somewhere = Regex::Somewhere(Box::new(Regex::Tag { c: 'a' }));
        assert_eq!(somewhere.exact("abc"), Some("bc"));
        assert_eq!(somewhere.exact("xyz"), None);
    }

    #[test]
    fn test_starts_matcher() {
        let starts = Regex::Starts(Box::new(Regex::Tag { c: 'a' }));

        assert_eq!(starts.exact("abc"), Some("bc"));
        assert_eq!(starts.exact("xyz"), None);
    }

    //
    #[test]
    fn test_complex() {
        let complex = Somewhere(Box::new(Regex::And(
            Box::new(Regex::Digit),
            Box::new(Regex::AtLeastOne(
                Box::new(Regex::Alphanumeric),
                Box::new(Regex::And(
                    Box::new(Regex::Or(
                        Box::new(Regex::Tag { c: '!' }),
                        Box::new(Regex::Tag { c: '?' }),
                    )),
                    Box::new(Regex::Ends),
                )),
            )),
        )));

        println!("{complex:?}");

        assert!(complex.exact("1aaaa!").is_some());
        assert!(complex.exact("blabla1aaaa!").is_some());
        assert!(complex.exact("blabla1aaaa?").is_some());
        assert!(complex.exact("11a?").is_some());
        assert!(complex.exact("1?").is_none());
        assert!(complex.exact("1!").is_none());
    }
}
