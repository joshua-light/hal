//! Hotkeys by key, not by letter: a Cyrillic letter typed where hal reads
//! commands becomes the Latin letter on the same key of a QWERTY board, so
//! `j`/`k`/`q` work with the Ukrainian (or Russian) layout switched on.
//! Fields that take text keep what was typed; [`crate::app::App::types_text`]
//! says which those are.
//!
//! Only letters are mapped. Punctuation exists on both layouts (`.` is `/`
//! on the Ukrainian board and itself on the English one), so it can't say
//! which key was pressed.

use crossterm::event::{KeyCode, KeyEvent};

/// ЙЦУКЕН rows against QWERTY, lowercase; Ukrainian first, then the
/// Russian letters that sit on other keys.
const LAYOUT: &[(char, char)] = &[
    ('й', 'q'),
    ('ц', 'w'),
    ('у', 'e'),
    ('к', 'r'),
    ('е', 't'),
    ('н', 'y'),
    ('г', 'u'),
    ('ш', 'i'),
    ('щ', 'o'),
    ('з', 'p'),
    ('х', '['),
    ('ї', ']'),
    ('ф', 'a'),
    ('і', 's'),
    ('в', 'd'),
    ('а', 'f'),
    ('п', 'g'),
    ('р', 'h'),
    ('о', 'j'),
    ('л', 'k'),
    ('д', 'l'),
    ('ж', ';'),
    ('є', '\''),
    ('ґ', '`'),
    ('я', 'z'),
    ('ч', 'x'),
    ('с', 'c'),
    ('м', 'v'),
    ('и', 'b'),
    ('т', 'n'),
    ('ь', 'm'),
    ('б', ','),
    ('ю', '.'),
    ('ы', 's'),
    ('э', '\''),
    ('ъ', ']'),
    ('ё', '`'),
];

/// The QWERTY character on the key that types `c`, Shift kept: `о` → `j`,
/// `О` → `J`, `Б` → `<`. `None` when `c` is not a mapped Cyrillic letter.
pub fn latin(c: char) -> Option<char> {
    let lower = c.to_lowercase().next()?;
    let &(_, base) = LAYOUT.iter().find(|(cyr, _)| *cyr == lower)?;
    if lower == c {
        return Some(base);
    }
    Some(match base {
        '[' => '{',
        ']' => '}',
        ';' => ':',
        '\'' => '"',
        '`' => '~',
        ',' => '<',
        '.' => '>',
        letter => letter.to_ascii_uppercase(),
    })
}

/// `key` as the same key would read on the English layout.
pub fn to_latin(mut key: KeyEvent) -> KeyEvent {
    if let KeyCode::Char(c) = key.code {
        if let Some(l) = latin(c) {
            key.code = KeyCode::Char(l);
        }
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn a_cyrillic_letter_reads_as_its_key() {
        assert_eq!(latin('о'), Some('j'));
        assert_eq!(latin('л'), Some('k'));
        assert_eq!(latin('й'), Some('q'));
        assert_eq!(latin('і'), Some('s'));
        assert_eq!(latin('ы'), Some('s'));
        assert_eq!(latin('б'), Some(','));
    }

    #[test]
    fn shift_stays_on() {
        assert_eq!(latin('Н'), Some('Y'));
        assert_eq!(latin('Р'), Some('H'));
        assert_eq!(latin('Б'), Some('<'));
        assert_eq!(latin('Ж'), Some(':'));
    }

    #[test]
    fn latin_and_punctuation_pass_through() {
        assert_eq!(latin('j'), None);
        assert_eq!(latin('.'), None);
        assert_eq!(latin('1'), None);
    }

    #[test]
    fn a_ctrl_chord_keeps_its_modifier() {
        let key = to_latin(KeyEvent::new(KeyCode::Char('д'), KeyModifiers::CONTROL));
        assert_eq!(key.code, KeyCode::Char('l'));
        assert_eq!(key.modifiers, KeyModifiers::CONTROL);
    }
}
