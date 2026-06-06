use ropey::Rope;

pub enum Motion {
    WordForward,
    WordBackward,
    WordEnd,
    LineEnd,
    LineStart,
    ParagraphForward,
    ParagraphBackward,
}

pub fn find_motion_range(rope: &Rope, pos: usize, motion: Motion) -> std::ops::Range<usize> {
    match motion {
        Motion::WordForward => {
            let mut end = pos;
            let len = rope.len_chars();
            
            // Se estivermos em uma palavra, pula até o fim dela
            if end < len && is_word_char(rope.char(end)) {
                while end < len && is_word_char(rope.char(end)) {
                    end += 1;
                }
            }
            
            // Pula espaços em branco (ou outros caracteres que não são de palavra)
            while end < len && !is_word_char(rope.char(end)) {
                end += 1;
            }
            
            pos..end
        }
        Motion::WordBackward => {
            let mut start = pos;
            
            // Se estivermos no início de uma palavra, ou em um espaço, 
            // primeiro voltamos para o final da palavra anterior.
            if start > 0 && !is_word_char(rope.char(start - 1)) {
                while start > 0 && !is_word_char(rope.char(start - 1)) {
                    start -= 1;
                }
            } else if start > 0 && is_word_char(rope.char(start - 1)) {
                // Se já estivermos dentro de uma palavra, voltamos até o início dela.
                // Mas se já estivermos no primeiro caractere da palavra, queremos a palavra anterior.
                if start > 1 && !is_word_char(rope.char(start - 2)) {
                    start -= 1;
                    while start > 0 && !is_word_char(rope.char(start - 1)) {
                        start -= 1;
                    }
                }
            }
            
            // Agora voltamos até o início da palavra atual
            while start > 0 && is_word_char(rope.char(start - 1)) {
                start -= 1;
            }
            
            start..pos
        }
        Motion::WordEnd => {
            let mut end = pos;
            let len = rope.len_chars();
            
            // Se já estivermos no fim de uma palavra, ou em um espaço, 
            // primeiro avançamos para o início da próxima palavra.
            if end < len && !is_word_char(rope.char(end)) {
                while end < len && !is_word_char(rope.char(end)) {
                    end += 1;
                }
            } else if end + 1 < len && !is_word_char(rope.char(end + 1)) {
                // Se estivermos no último caractere de uma palavra, pula para a próxima
                end += 1;
                while end < len && !is_word_char(rope.char(end)) {
                    end += 1;
                }
            }
            
            // Agora avançamos até o final da palavra atual (ou da próxima, se pulamos)
            if end < len {
                while end + 1 < len && is_word_char(rope.char(end + 1)) {
                    end += 1;
                }
            }
            
            pos..end
        }
        Motion::LineEnd => {
            let line = rope.char_to_line(pos);
            let end = rope.line_to_char(line + 1).saturating_sub(1);
            pos..end
        }
        Motion::LineStart => {
            let line = rope.char_to_line(pos);
            let start = rope.line_to_char(line);
            start..pos
        }
        _ => pos..pos,
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    #[test]
    fn test_word_forward() {
        let rope = Rope::from_str("hello   world  test");
        // De 'h' (0) para início de 'world' (8)
        assert_eq!(find_motion_range(&rope, 0, Motion::WordForward).end, 8);
        // De meio da palavra para início da próxima
        assert_eq!(find_motion_range(&rope, 2, Motion::WordForward).end, 8);
    }

    #[test]
    fn test_word_backward() {
        let rope = Rope::from_str("hello   world  test");
        // De 'w' (8) para início de 'hello' (0)
        assert_eq!(find_motion_range(&rope, 8, Motion::WordBackward).start, 0);
        // De meio da palavra para início da mesma
        assert_eq!(find_motion_range(&rope, 10, Motion::WordBackward).start, 8);
    }

    #[test]
    fn test_word_end() {
        let rope = Rope::from_str("hello   world  test");
        // De 'h' (0) para fim de 'hello' (4)
        assert_eq!(find_motion_range(&rope, 0, Motion::WordEnd).end, 4);
        // De 'e' (1) para fim de 'hello' (4)
        assert_eq!(find_motion_range(&rope, 1, Motion::WordEnd).end, 4);
        // De 'o' (4) para fim de 'world' (12)
        assert_eq!(find_motion_range(&rope, 4, Motion::WordEnd).end, 12);
        // De espaço (5) para fim de 'world' (12)
        assert_eq!(find_motion_range(&rope, 5, Motion::WordEnd).end, 12);
    }
}
