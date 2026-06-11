use crate::*;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar);

use lalrpop_util::ParseError;

pub fn parse(s: &str) -> AST {
    let mut ast = AST {
        fns: Vec::new(),
        main_fn: usize::MAX,
    };
    match grammar::ASTParser::new().parse(&mut ast, s) {
        Ok(()) => ast,
        Err(err) => print_parse_error(s, err),
    }
}

fn print_parse_error(input: &str, err: ParseError<usize, grammar::Token, &str>) -> ! {
    let (start, _) = match err {
        ParseError::InvalidToken { location } => (location, location + 1),
        ParseError::UnrecognizedEof { location, .. } => (location, location + 1),
        ParseError::UnrecognizedToken { token, .. } => (token.0, token.2),
        ParseError::ExtraToken { token } => (token.0, token.2),
        ParseError::User { .. } => (0, 0),
    };

    let (line, col, linestr) = location_to_line_col(input, start);

    println!("Error at line {line}, col {col}");
    println!("ERR: {linestr}");

    let mut buf = String::new();
    for _ in 0..col {
        buf.push(' ');
    }
    buf.push('^');
    println!("ERR: {buf}");
    panic!("parse error");
}

fn location_to_line_col(input: &str, offset: usize) -> (usize, usize, &str) {
    let mut line = 1;
    let mut col = 1;

    let mut linestr = "";

    for (i, c) in input.char_indices() {
        if col == 1 {
            linestr = &input[i..];
            linestr = linestr.split("\n").next().unwrap();
        }

        if i == offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
            
        } else {
            col += 1;
        }
    }
    (line, col, linestr)
}

