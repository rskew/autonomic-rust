pub fn remove_punctuation(s: &str) -> String {
    s.chars().filter(|x| match *x {
        '.'|','|';'|':'|'!'|'?'|'\''|'\"'|'\\'|'/'|'<'|'>'|'~'|'('|')'|'{'|'}'|'['|']' => false,
        _ => true,
    }).collect::<String>()
}
