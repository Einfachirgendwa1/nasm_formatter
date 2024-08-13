use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[cold]
fn probably_not() {}

#[inline]
fn unlikely(cond: bool) -> bool {
    if cond {
        probably_not();
    }
    cond
}

pub enum Indentation {
    Tabs { charlen: u32 }, // Like in https://projectacrn.github.io/latest/developer-guides/asm_coding_guidelines.html#asm-cs-06-tabs-shall-be-8-characters-wide
    Spaces { amount: u32 },
}

// TODO: Implement missing options
pub struct Settings {
    indentation: Indentation,
    inline_labels: bool,
    lowercase_instructions: bool,
    lowercase_registers: bool,
    lowercase_custom_names: bool,
    line_length_limit: Option<u32>,
    allow_multiline_instructions: bool,
    align_assembler_directives: bool,
    align_labels_to_start_of_line: bool,
    align_instruction_statements: bool,
    allow_double_slash_comments: bool,
}

impl Default for Settings {
    // Mostly following https://projectacrn.github.io/latest/developer-guides/asm_coding_guidelines.html
    fn default() -> Self {
        Self {
            indentation: Indentation::Tabs { charlen: 4 },
            inline_labels: false,
            lowercase_instructions: true,
            lowercase_registers: true,
            lowercase_custom_names: true,
            line_length_limit: Some(120),
            allow_multiline_instructions: false,
            align_assembler_directives: true,
            align_labels_to_start_of_line: true,
            align_instruction_statements: true,
            allow_double_slash_comments: true,
        }
    }
}

macro_rules! log {
    ($buffer:expr, $($e:expr), *) => {
        println!($($e, )*);
        $buffer.push_str(&format!($($e, )*));
    };
}

fn main() {}

#[inline]
pub fn format(file: File, log_buffer: &mut String, settings: &Settings) -> String {
    let mut indent_amount = 0;
    let reader = BufReader::new(file);
    let mut output = String::new();

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line.trim().to_string(),
            Err(err) => {
                log!(log_buffer, "Error reading line from file: {}", err);
                continue;
            }
        };

        parse_str(
            line.as_str(),
            &mut output,
            log_buffer,
            &mut indent_amount,
            &settings,
        );
    }
    output
}

#[inline]
fn parse_str(
    value: &str,
    output: &mut String,
    log_buffer: &mut String,
    indent_amount: &mut u32,
    settings: &Settings,
) {
    let mut chars = value.chars();
    let first_char = match chars.next() {
        Some(c) => c,
        None => {
            output.push('\n');
            return;
        }
    };

    if unlikely(first_char == ':') {
        log!(
            log_buffer,
            "Parsing Error: Colon without an identifier in front of it."
        );
        write_indent(&settings.indentation, *indent_amount, output);
        output.push(':');
        output.push('\n');
        return;
    }

    let mut instruction = String::from(first_char);

    while let Some(next) = chars.next() {
        instruction.push(next);
        if next == ':' || instruction == "section " {
            *indent_amount = 1;
            output.push_str(instruction.as_str());

            if next != ':' {
                while let Some(c) = chars.next() {
                    output.push(c);
                    if c == '\n' {
                        break;
                    }
                }
            }
            let the_rest = chars.collect::<String>().trim().to_string();
            output.push('\n');
            if !the_rest.is_empty() {
                parse_str(&the_rest, output, log_buffer, indent_amount, settings);
            }
            return;
        }
    }
    write_indent(&settings.indentation, *indent_amount, output);
    output.push_str(instruction.as_str());
    output.push('\n');
}

#[inline]
fn write_indent(indentation: &Indentation, indent_amount: u32, output: &mut String) {
    match indentation {
        Indentation::Tabs { charlen: _ } => {
            for _ in 0..indent_amount {
                output.push('\t');
            }
        }
        Indentation::Spaces { amount } => {
            for _ in 0..amount * indent_amount {
                output.push(' ');
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    macro_rules! test_format {
        ($name:ident, $original:expr, $expected:expr, $settings:expr) => {
            #[test]
            fn $name() {
                let original = File::open($original).unwrap();
                let expected = File::open($expected).unwrap();

                let expected: String = BufReader::new(expected)
                    .lines()
                    .map(|r| r.unwrap())
                    .map(|r| r + "\n")
                    .collect();

                let mut log_buffer = String::new();

                assert_eq!(format(original, &mut log_buffer, &$settings), expected);
            }
        };
    }

    test_format!(
        simple_exit_tabs,
        "test_data/original/simple_exit_tabs.asm",
        "test_data/expected/simple_exit_tabs.asm",
        Settings::default()
    );

    test_format!(
        simple_exit_spaces,
        "test_data/original/simple_exit_spaces.asm",
        "test_data/expected/simple_exit_spaces.asm",
        Settings {
            indentation: Indentation::Spaces { amount: 2 },
            ..Default::default()
        }
    );

    // TODO: Test more thoroughly
}
