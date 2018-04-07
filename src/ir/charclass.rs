pub fn match_length<F>(s: &str, predicate: F) -> usize
where
    F: Fn(&char) -> bool,
{
    s.chars()
        .take_while(predicate)
        .fold(0, |a, c| a + c.len_utf8())
}

pub fn is_whitespace(c: &char) -> bool {
    c.is_whitespace()
}

pub fn is_identifier_start(c: &char) -> bool {
    c.is_alphabetic() || *c == '_'
}

pub fn is_identifier(c: &char) -> bool {
    c.is_alphanumeric() || *c == '_'
}

pub fn is_decimal_digit(c: &char) -> bool {
    c.is_digit(10)
}
