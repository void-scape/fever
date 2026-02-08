use bevy::prelude::*;
use rand::{Rng, seq::IndexedRandom};
use std::{alloc::Layout, collections::HashMap, fmt::Write, slice, str};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(markov("assets/dc.txt"));
    });
}

#[derive(Component)]
pub struct Markov(HashMap<[Token; 3], usize>);

impl Markov {
    pub fn write_text(&self, w: &mut impl Write, chars: usize) {
        let mut rng = rand::rng();
        let output = generate_tokens(&mut rng, &self.0, chars);
        pretty_print_tokens(w, &output);
    }
}

fn markov(path: &str) -> Markov {
    let input = std::fs::read_to_string(path).unwrap();

    // arena is leaked and never unallocated
    let mut arena = Arena::new(1024);
    let tokens = tokenize_input(&mut arena, &input);

    let mut sorted_tokens = tokens.iter().collect::<Vec<_>>();
    sorted_tokens.sort_by_key(|(_, c)| *c);

    Markov(tokens)
}

struct Arena {
    alloc: *mut u8,
    capacity: usize,
    len: usize,
}

impl Arena {
    fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 1).unwrap();
        let alloc = unsafe { std::alloc::alloc(layout) };
        assert!(!alloc.is_null());

        Self {
            alloc,
            capacity,
            len: 0,
        }
    }

    fn alloc_lowercase(&mut self, str: &str) -> &'static str {
        assert!(!str.is_empty());

        if self.len + str.len() > self.capacity {
            let layout = Layout::from_size_align(self.capacity * 2, 1).unwrap();
            self.alloc = unsafe { std::alloc::alloc(layout) };
            assert!(!self.alloc.is_null());
            self.capacity *= 2;
            self.len = 0;
            assert!(str.len() <= self.capacity);
        }

        unsafe {
            let start = self.alloc.add(self.len);
            self.len += str.len();
            let mut ptr = start;
            for byte in str.bytes() {
                *ptr = byte;
                ptr = ptr.add(1);
            }
            let str = str::from_utf8_unchecked_mut(slice::from_raw_parts_mut(start, str.len()));
            str.make_ascii_lowercase();
            str
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Token {
    Str(&'static str),
    Puncuation(char),
}

impl Token {
    fn len(&self) -> usize {
        match self {
            Self::Str(str) => str.len(),
            Self::Puncuation(_) => 1,
        }
    }

    fn should_strip(&self) -> bool {
        let delimiters = ['(', ')', '[', ']', '<', '>', '{', '}', '\"'];
        match self {
            Self::Str(str) => str.is_empty() || str.chars().any(|c| delimiters.contains(&c)),
            Self::Puncuation(c) => delimiters.contains(c),
        }
    }
}

fn tokenize_input(arena: &mut Arena, input: &str) -> HashMap<[Token; 3], usize> {
    let mut result = Vec::new();
    let mut current = String::new();

    for c in input.chars() {
        if c.is_ascii_punctuation() {
            if !current.is_empty() {
                result.push(Token::Str(arena.alloc_lowercase(&current)));
                current.clear();
            }
            result.push(Token::Puncuation(c));
        } else if c.is_whitespace() {
            if !current.is_empty() {
                result.push(Token::Str(arena.alloc_lowercase(&current)));
                current.clear();
            }
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        result.push(Token::Str(arena.alloc_lowercase(&current)));
    }

    let mut tokens = HashMap::with_capacity(result.len());
    for window in result.windows(3) {
        if window.iter().all(|t| !t.should_strip()) {
            *tokens.entry([window[0], window[1], window[2]]).or_default() += 1;
        }
    }
    tokens
}

fn generate_tokens(
    rng: &mut impl Rng,
    tokens: &HashMap<[Token; 3], usize>,
    chars: usize,
) -> Vec<Token> {
    let mut chars = chars as isize;

    let mut key = HashMap::<[Token; 2], Vec<(Token, usize)>>::new();
    for (k, v) in tokens.iter() {
        key.entry([k[0], k[1]]).or_default().push((k[2], *v));
    }

    let mut output = Vec::new();
    loop {
        // grab random entry to start
        let (start, ends) = key.iter().nth(rng.random_range(0..key.len())).unwrap();
        let end = ends.choose_weighted(rng, |(_, c)| *c).unwrap().0;

        chars -= start[0].len() as isize;
        chars -= start[1].len() as isize;
        chars -= end.len() as isize;
        output.extend([start[0], start[1], end]);
        if chars <= 0 {
            if output
                .last()
                .is_none_or(|p| !matches!(p, Token::Puncuation('.')))
            {
                if output
                    .last()
                    .is_some_and(|p| matches!(p, Token::Puncuation(_)))
                {
                    output.pop();
                }
                output.push(Token::Puncuation('.'));
            }
            return output;
        }

        let mut first = start[1];
        let mut second = end;

        while let Some(ends) = key.get(&[first, second]) {
            let end = ends.choose_weighted(rng, |(_, c)| *c).unwrap().0;
            chars -= end.len() as isize;
            if chars <= 0 {
                if output
                    .last()
                    .is_none_or(|p| !matches!(p, Token::Puncuation('.')))
                {
                    if output
                        .last()
                        .is_some_and(|p| matches!(p, Token::Puncuation(_)))
                    {
                        output.pop();
                    }
                    output.push(Token::Puncuation('.'));
                }
                return output;
            }
            output.push(end);
            first = second;
            second = end;
        }
    }
}

fn pretty_print_tokens(w: &mut impl Write, output: &[Token]) {
    let mut first = true;
    let mut skip_pad = false;
    let mut capitalize = true;
    for token in output.iter() {
        match token {
            Token::Str(str) => {
                if !first && !skip_pad {
                    w.write_char(' ').unwrap();
                }
                first = false;
                skip_pad = false;

                if capitalize && str.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
                    let c = str.chars().next().unwrap();
                    w.write_str(&format!("{}", c.to_ascii_uppercase())).unwrap();
                    w.write_str(&str[c.len_utf8()..]).unwrap();
                } else if *str == "i" {
                    w.write_char('I').unwrap();
                } else {
                    w.write_str(str).unwrap();
                }
                capitalize = false;
            }
            Token::Puncuation(c) => {
                capitalize = *c == '.' || *c == '!' || *c == '?';
                skip_pad = *c == '\'' || *c == '-';
                w.write_char(*c).unwrap();
            }
        }
    }
    if !capitalize {
        w.write_char('.').unwrap();
    }
}
