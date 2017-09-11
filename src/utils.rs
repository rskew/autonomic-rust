pub fn remove_punctuation(s: String) -> String {
    s.chars().filter(|x| match *x {
        '.'|','|';'|':'|'!'|'?'|'\''|'\"'|'\\'|'/'|'<'|'>'|'~'|'('|')'|'{'|'}'|'['|']' => false,
        _ => true,
    }).collect::<String>()
}
