use anyhow::anyhow;

use crate::regex::Regex;
use crate::regex::Regex::{Somewhere, Starts};

fn or(input: &str) -> anyhow::Result<(&str, Regex)> {
    let (left_input, right_input) = input
        .split_once('|')
        .ok_or_else(|| anyhow!("Expected | but couldn't find it {input}"))?;
    let (_, left) = regex_inner(left_input)?;

    let (right_input, remaining_input) = right_input
        .split_once(')')
        .ok_or_else(|| anyhow!("Expected ) but couldn't find it in {right_input}"))?;
    let (_, right) = regex_inner(right_input)?;

    let result = Regex::Or(Box::new(left), Box::new(right));
    Ok((remaining_input, result))
}

fn any(input: &str) -> anyhow::Result<(&str, Regex)> {
    let (negate, input) = input
        .strip_prefix('^')
        .map(|rest| (true, rest))
        .unwrap_or((false, input));

    let (input, remaining_input) = input
        .split_once(']')
        .ok_or_else(|| anyhow!("Expected ] but couldn't find it in {input}"))?;

    let collected: Vec<char> = input.chars().collect();
    let result = if negate {
        Regex::NotAny { cs: collected }
    } else {
        Regex::Any { cs: collected }
    };

    Ok((remaining_input, result))
}

fn escaped(input: &str) -> anyhow::Result<(&str, Regex)> {
    let mut chars = input.chars();
    let reg = match chars.next() {
        Some('d') => Regex::Digit,
        Some('w') => Regex::Alphanumeric,
        Some('t') => Regex::Whitespace,
        any => {
            return Err(anyhow!("Expected 'd' or 'w' or 't' after '\\' but got {any:?}"));
        }
    };
    Ok((chars.as_str(), reg))
}

fn regex_inner(input: &str) -> anyhow::Result<(&str, Regex)> {
    let mut chars = input.chars();
    let (first, reminder) = if let Some(first) = chars.next() {
        (first, chars.as_str())
    } else {
        return Ok((input, Regex::Noop));
    };

    let (reminder, reg) = match first {
        '*' => (reminder, Regex::Wildcard),
        '$' if reminder.is_empty() => (reminder, Regex::Ends),
        '$' => {
            return Err(anyhow!("found $ not in the end of the line"));
        }
        '\\' => escaped(reminder)?,
        '[' => any(reminder)?,
        '(' => or(reminder)?,
        c => (reminder, Regex::Tag { c }),
    };

    let (reminder, reg) = if let Some(stripped) = reminder.strip_prefix('+') {
        let (i, n) = regex_inner(stripped)?;
        (i, Regex::AtLeastOne(Box::new(reg), Box::new(n)))
    } else if let Some(stripped) = reminder.strip_prefix('?') {
        let (i, n) = regex_inner(stripped)?;
        (i, Regex::ZeroOrOne(Box::new(reg), Box::new(n)))
    } else {
        let (i, n) = regex_inner(reminder)?;
        (i, Regex::And(Box::new(reg), Box::new(n)))
    };

    Ok((reminder, reg))
}

pub fn regex(input: &str) -> anyhow::Result<Regex> {
    let (starts, input) = input
        .strip_prefix('^')
        .map(|rest| (true, rest))
        .unwrap_or((false, input));

    let (_input, res) = regex_inner(input)?;
    if starts {
        Ok(Starts(Box::new(res)))
    } else {
        Ok(Somewhere(Box::new(res)))
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::regex;

    #[test]
    fn t() {
        let regi = regex("a").unwrap();
        assert_eq!(regi.exact("a"), Some(""));
    }

    #[test]
    fn t1() {
        let regi = regex("\\d").unwrap();
        assert_eq!(regi.exact("2a"), Some("a"));
    }

    #[test]
    fn t2() {
        let regi = regex("\\t").unwrap();
        assert_eq!(regi.exact(" a"), Some("a"));
    }

    #[test]
    fn t3() {
        let regi = regex("\\w").unwrap();
        assert_eq!(regi.exact("b a"), Some(" a"));
    }

    #[test]
    fn t4() {
        let regi = regex("ab").unwrap();
        assert!(regi.exact("abcde").is_some());
    }

    #[test]
    fn t5() {
        let regi = regex(r"[abcd]").unwrap();
        assert!(regi.exact("a").is_some());
    }

    #[test]
    fn t6() {
        let regi = regex(r"(abcd|\w+1)+").unwrap();
        println!("{regi:?}");
        assert_eq!(regi.exact("abcdabcdwwwwwww1"), Some(""));
    }
}
