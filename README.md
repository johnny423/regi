# Regi

Regi is a lightweight Rust library for exact pattern matching within strings using regular expressions.

## Features

- Search for the exact occurrence of a pattern within a given haystack string.
- Supports simple regular expressions include
  - wildcards - like '.', '\d', '\w', '\t'.
  - any and not any - '[abc]', '[^abc]'.
  - at-least-one and zero-or-one matchers - 'a+', 'a?'
  - or - (a|b)
  - starts and ends - '^abc', 'abc$'



