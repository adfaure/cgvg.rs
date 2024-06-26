use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use std::iter;

/// Find the number of digits of a number.
pub fn number_of_digits<T>(number: &T) -> u32
where
    T: Into<u64> + Copy,
{
    let num = (*number).into();
    if num == 0 as u64 {
        1 as u32
    } else {
        (num as f64).log10().floor() as u32 + 1
    }
}

/// Iter over a colored string (it reads the `\u{1b}` codes).
pub fn iter_colored(string: &str) -> impl Iterator<Item = String> + '_ {
    string.chars().batching(|it| {
        match it.next() {
            Some(t) => {
                if t == '\u{1b}' {
                    let s: String = it.take_while(|c| *c != 'm').collect();
                    // Since take_while consumes the first false, we mannually add the 'm'
                    Some(format!("{t}{s}m"))
                } else {
                    Some(t.to_string())
                }
            }
            None => None,
        }
    })
}

/// Pad a number with white spaces (if needed) to be printed with the `max_size` len.
///
/// panics if the number of digits of `number` is greater than `max_size`.
pub fn pad_number(number: u32, max_size: u32) -> String {
    let nb_digits = number_of_digits(&number);
    assert!(
        nb_digits <= max_size,
        "pad_number wrong arguments number of digits of {nb_digits} > {max_size}"
    );

    if nb_digits < max_size {
        let diff = max_size - nb_digits;
        let padding = std::iter::repeat(" ")
            .take(diff as usize)
            .collect::<String>();
        format!("{}{}", number, padding)
    } else {
        format!("{}", number)
    }
}

/// Wrap text with support for colored string.
///
/// - replace tabs with a number of whitespace to ensure that the printed line stays in the
/// delimited space.
/// - if `fill_end` is true, then empty spaces are added at the end of each wrapped line.
pub fn wrap_text<'a>(
    text: &'a str,
    max_length: &'a u32,
    tab_size: &'a u32,
    fill_end: bool,
) -> impl Iterator<Item = String> + 'a {
    let mut memory: Vec<String> = vec![];

    iter_colored(text)
        .map(|c| {
            if c == "\t" {
                return iter::repeat(" ".to_string()).take(*tab_size as usize);
            } else {
                return iter::repeat(c).take(1);
            }
        })
        .flatten()
        .batching(move |it| {
            let (len, temp) = it
                .fold_while((0, memory.clone()), |(length, mut acc), c| {
                    let mut new_length = length;

                    acc.push(c.clone());

                    if !c.starts_with('\u{1b}') {
                        new_length += 1;
                    } else {
                        match c.as_str() {
                            "\u{1b}[0m" => {
                                memory.clear();
                            }
                            _ => {
                                memory.push(c.clone());
                            }
                        }
                    }

                    if new_length == *max_length {
                        if !memory.is_empty() {
                            memory = memory.clone();
                            acc.push("\u{1b}[0m".to_string());
                        }

                        Done((new_length, acc))
                    } else {
                        Continue((new_length, acc))
                    }
                })
                .into_inner();

            let line: String = temp.into_iter().collect();

            // If len is 0 then the string contains remaining style
            // harder to clean than to ignore, and I think ignoring wont change the style
            if len == 0 {
                None
            } else if fill_end && &len < max_length {
                let padding = std::iter::repeat(" ")
                    .take((max_length - len) as usize)
                    .collect::<String>();
                Some(format!("{line}{padding}"))
            } else {
                Some(line)
            }
        })
        .map(|array| String::from(array))
    // .collect_vec();
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use colored::Colorize;

    #[test]
    fn test_wrap_text() {
        let tab_size = 8;
        // Simple cases
        let res = wrap_text("1234567890abc", &5, &tab_size, false).collect_vec();
        assert_eq!(vec!["12345", "67890", "abc"], res);

        let res = wrap_text("1234567890abc", &15, &tab_size, false).collect_vec();
        assert_eq!(vec!["1234567890abc"], res);

        // Got coloring
        let blue = format!("aaaaabbbbbzzzzz").blue().to_string();

        let res = wrap_text(&blue, &5, &tab_size, false).collect_vec();
        assert_eq!(
            vec![
                "\u{1b}[34maaaaa\u{1b}[0m",
                "\u{1b}[34mbbbbb\u{1b}[0m",
                "\u{1b}[34mzzzzz\u{1b}[0m"
            ],
            res
        );

        let blue_bold_underline = format!("aaaaabbbbbzzzzz")
            .blue()
            .bold()
            .underline()
            .to_string();

        let res = wrap_text(&blue_bold_underline, &5, &tab_size, false).collect_vec();

        assert_eq!(
            vec![
                "\u{1b}[1;4;34maaaaa\u{1b}[0m",
                "\u{1b}[1;4;34mbbbbb\u{1b}[0m",
                "\u{1b}[1;4;34mzzzzz\u{1b}[0m"
            ],
            res
        );

        let begin = format!("{}", "aaaaa".to_string().blue());
        let middle = format!("{}", "bbbbb".to_string().white());
        let end = format!("{}", "zzzzz".to_string().red());

        let blue_bold_underline = format!("{begin}{middle}{end}").underline().to_string();

        let res = wrap_text(&blue_bold_underline, &5, &tab_size, false).collect_vec();
        assert_eq!(
            vec![
                "\u{1b}[4m\u{1b}[34maaaaa\u{1b}[0m",
                "\u{1b}[4m\u{1b}[34m\u{1b}[0m\u{1b}[4m\u{1b}[37mbbbbb\u{1b}[0m",
                "\u{1b}[4m\u{1b}[37m\u{1b}[0m\u{1b}[4m\u{1b}[31mzzzzz\u{1b}[0m"
            ],
            res
        );

        let res = wrap_text(&"\taaaaaaaabbbbbbbb".to_string(), &8, &tab_size, false).collect_vec();

        println!("{res:?}");

        println!("{}", "aaaaaaaabbbbbbbb");
        println!("{}", "\taaaaaaaabbbbbbbb");

        assert_eq!(vec!["        ", "aaaaaaaa", "bbbbbbbb"], res);
    }

    #[test]
    fn test_iter_colored() {
        let blue_bold_underline = format!("abz").blue().bold().underline().to_string();

        assert_eq!(
            vec!["\u{1b}[1;4;34m", "a", "b", "z", "\u{1b}[0m"],
            iter_colored(&blue_bold_underline).collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_number_of_digits() {
        let zero = 0_u32;
        assert_eq!(1, number_of_digits(&zero));

        let cinq = 5_u32;
        assert_eq!(1, number_of_digits(&cinq));

        let dix = 11_u32;
        assert_eq!(2, number_of_digits(&dix));

        let cinquante = 100_u32;
        assert_eq!(3, number_of_digits(&cinquante));
    }

    #[test]
    #[should_panic(expected = "pad_number wrong arguments number of digits of 3 > 0")]
    fn test_pad_number_assert() {
        assert_eq!("100  ", pad_number(100, 5));
        assert_eq!("100", pad_number(100, 3));

        // Last one should assert
        assert_eq!("100  ", pad_number(100, 0));
    }
}
