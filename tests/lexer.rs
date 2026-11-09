use riscv_rv32i_interpreter::compiler::lexer::{tokenize, LexError, Position, TokenKind};
use riscv_rv32i_interpreter::compiler::lexer::TokenKind::*;


fn kinds(source: &str) -> Vec<TokenKind> {
    tokenize(source).unwrap().into_iter().map(|t| t.kind).collect()
}

fn var(name: &str) -> TokenKind {
    VarName(name.to_string())
}

fn class(name: &str) -> TokenKind {
    ClassName(name.to_string())
}

fn at(line: u32, column: u32) -> Position {
    Position { line, column }
}

#[test]
fn empty_source() {
    assert_eq!(kinds(""), []);
}

#[test]
fn whitespace_only() {
    assert_eq!(kinds(" \t\n\r\n  "), []);
}

#[test]
fn keywords() {
    assert_eq!(kinds("class int void if else while func return"), [Class, Int, Void, If, Else, While, Func, Return]);
}

#[test]
fn keywords_are_reserved() {
    // a keyword is never a VarName, so the parser rejects `int else = 1;`
    assert_eq!(kinds("int else = 1;"), [Int, Else, Assign, IntNum(1), Semicolon]);
    assert_eq!(kinds("return(x)"), [Return, LParen, var("x"), RParen]);
    assert_eq!(kinds("int void;"), [Int, Void, Semicolon]);
}

#[test]
fn keyword_prefixes_are_names() {
    assert_eq!(kinds("integer voids iff elsewhere whiles classy funcs returned"), [
        var("integer"), var("voids"), var("iff"), var("elsewhere"), var("whiles"), var("classy"), var("funcs"), var("returned"),
    ]);
}

#[test]
fn capitalized_keywords_are_class_names() {
    assert_eq!(kinds("Class If Int"), [class("Class"), class("If"), class("Int")]);
}

#[test]
fn class_names() {
    assert_eq!(kinds("Main A My_Class2"), [class("Main"), class("A"), class("My_Class2")]);
}

#[test]
fn var_names() {
    assert_eq!(kinds("x camelCase snake_case x1 a_B_9"), [
        var("x"), var("camelCase"), var("snake_case"), var("x1"), var("a_B_9"),
    ]);
}

#[test]
fn names_can_contain_dash() {
    // the grammar allows '-' in names, so there is no subtraction here
    assert_eq!(kinds("a-b My-Class"), [var("a-b"), class("My-Class")]);
    assert_eq!(kinds("a - b"), [var("a"), Minus, var("b")]);
    assert_eq!(kinds("a -b"), [var("a"), Minus, var("b")]);
    assert_eq!(kinds("step-size-2"), [var("step-size-2")]);
}

#[test]
fn names_can_end_with_dash() {
    // the dash belongs to the name as long as it directly follows it
    assert_eq!(kinds("a- b"), [var("a-"), var("b")]);
    assert_eq!(kinds("a--1"), [var("a--1")]);
}

#[test]
fn unary_minus() {
    // the lexer only produces Minus; the parser decides if it is unary or binary
    assert_eq!(kinds("-5"), [Minus, IntNum(5)]);
    assert_eq!(kinds("- -x"), [Minus, Minus, var("x")]);
    assert_eq!(kinds("--x"), [Minus, Minus, var("x")]);
    assert_eq!(kinds("a - -b"), [var("a"), Minus, Minus, var("b")]);
    assert_eq!(kinds("-(a+1)"), [Minus, LParen, var("a"), Plus, IntNum(1), RParen]);
}

#[test]
fn parenthesized_conditions() {
    assert_eq!(kinds("(a<1||b<2)&&c<3"), [
        LParen, var("a"), Lt, IntNum(1), Or, var("b"), Lt, IntNum(2), RParen, And, var("c"), Lt, IntNum(3),
    ]);
}

#[test]
fn names_cannot_start_with_digit_underscore_or_dash() {
    assert_eq!(kinds("1x"), [IntNum(1), var("x")]);
    assert_eq!(tokenize("_x"), Err(LexError::UnexpectedChar('_', at(1, 1))));
    assert_eq!(kinds("-x"), [Minus, var("x")]);
}

#[test]
fn characters_between_z_and_a_are_not_name_characters() {
    // [ \ ] ^ ` lie between 'Z' and 'a' in ASCII
    for c in ['[', '\\', ']', '^', '`'] {
        assert_eq!(tokenize(&format!("a{c}b")), Err(LexError::UnexpectedChar(c, at(1, 2))));
    }
}

#[test]
fn integers() {
    assert_eq!(kinds("0 7 42 007 2147483647"), [IntNum(0), IntNum(7), IntNum(42), IntNum(7), IntNum(i32::MAX)]);
}

#[test]
fn integer_too_large() {
    assert_eq!(tokenize("x = 2147483648"), Err(LexError::IntegerTooLarge(at(1, 5))));
    assert_eq!(tokenize("99999999999999999999"), Err(LexError::IntegerTooLarge(at(1, 1))));
}

#[test]
fn arithmetic_operators() {
    assert_eq!(kinds("+ - * /"), [Plus, Minus, Mult, Div]);
}

#[test]
fn comparison_operators() {
    assert_eq!(kinds("== != < > >= <="), [Eq, Ne, Lt, Gt, Ge, Le]);
}

#[test]
fn logical_operators() {
    assert_eq!(kinds("&& ||"), [And, Or]);
}

#[test]
fn punctuation() {
    assert_eq!(kinds("= ; , ( ) { }"), [Assign, Semicolon, Comma, LParen, RParen, LBrace, RBrace]);
}

#[test]
fn two_char_operators_take_longest_match() {
    assert_eq!(kinds("==="), [Eq, Assign]);
    assert_eq!(kinds("<=="), [Le, Assign]);
    assert_eq!(kinds("<<"), [Lt, Lt]);
}

#[test]
fn whitespace_splits_two_char_operators() {
    assert_eq!(kinds("= ="), [Assign, Assign]);
    assert_eq!(kinds("< ="), [Lt, Assign]);
    assert_eq!(kinds("> ="), [Gt, Assign]);
}

#[test]
fn lone_halves_of_operators_are_errors() {
    assert_eq!(tokenize("!"), Err(LexError::UnexpectedChar('!', at(1, 1))));
    assert_eq!(tokenize("a ! = b"), Err(LexError::UnexpectedChar('!', at(1, 3))));
    assert_eq!(tokenize("a & b"), Err(LexError::UnexpectedChar('&', at(1, 3))));
    assert_eq!(tokenize("a | b"), Err(LexError::UnexpectedChar('|', at(1, 3))));
}

#[test]
fn unexpected_characters() {
    for c in ['.', '@', '#', '"', '%', 'ä'] {
        assert_eq!(tokenize(&c.to_string()), Err(LexError::UnexpectedChar(c, at(1, 1))));
    }
}

#[test]
fn whitespace_does_not_matter() {
    let spaced = kinds("int  x\t=\n 1 +\r\n y ;");
    let compact = kinds("int x=1+y;");
    assert_eq!(spaced, compact);
    assert_eq!(compact, [Int, var("x"), Assign, IntNum(1), Plus, var("y"), Semicolon]);
}

#[test]
fn whitespace_needed_between_words() {
    assert_eq!(kinds("intx"), [var("intx")]);
    assert_eq!(kinds("int x"), [Int, var("x")]);
}

#[test]
fn positions() {
    let tokens = tokenize("class Main {\n  int x = 10;\n}").unwrap();
    let positions: Vec<_> = tokens.iter().map(|t| t.position).collect();
    assert_eq!(positions, [
        at(1, 1), at(1, 7), at(1, 12),                       // class Main {
        at(2, 3), at(2, 7), at(2, 9), at(2, 11), at(2, 13),  //   int x = 10;
        at(3, 1),                                            // }
    ]);
}

#[test]
fn error_position_on_later_line() {
    assert_eq!(tokenize("int x = 1;\nint y = 2 @ 3;"), Err(LexError::UnexpectedChar('@', at(2, 11))));
}

#[test]
fn parameter_lists() {
    assert_eq!(kinds("f()"), [var("f"), LParen, RParen]);
    assert_eq!(kinds("f(a,b , 1)"), [var("f"), LParen, var("a"), Comma, var("b"), Comma, IntNum(1), RParen]);
}

#[test]
fn full_program() {
    // the example from the README
    let source = "
        class Main {
            func int add(int a, int b) {
                return a + b;
            }

            func int abs(int n) {
                if (n < 0) {
                    return -n;
                }
                return n;
            }

            func void count-down(int n) {
                while (n > 0) {
                    if (n == 3) {
                        return;
                    }
                    n = n - 1;
                }
                return;
            }

            int x = 10;
            int step-size = -1;
            int sum = abs(-5 * 2) - 10;
            while ((x > 0 && sum != 100) || x == 10) {
                sum = add(sum, x * (2 - 1));
                x = x + step-size;
            }
            if (sum >= 55 && (sum != 0 || x == 0)) {
                sum = sum / 5;
            } else {
                sum = -(sum + 1);
            }
            count-down(sum);
        }
    ";
    assert_eq!(kinds(source), [
        Class, class("Main"), LBrace,
        Func, Int, var("add"), LParen, Int, var("a"), Comma, Int, var("b"), RParen, LBrace,
        Return, var("a"), Plus, var("b"), Semicolon,
        RBrace,
        Func, Int, var("abs"), LParen, Int, var("n"), RParen, LBrace,
        If, LParen, var("n"), Lt, IntNum(0), RParen, LBrace,
        Return, Minus, var("n"), Semicolon,
        RBrace,
        Return, var("n"), Semicolon,
        RBrace,
        Func, Void, var("count-down"), LParen, Int, var("n"), RParen, LBrace,
        While, LParen, var("n"), Gt, IntNum(0), RParen, LBrace,
        If, LParen, var("n"), Eq, IntNum(3), RParen, LBrace,
        Return, Semicolon,
        RBrace,
        var("n"), Assign, var("n"), Minus, IntNum(1), Semicolon,
        RBrace,
        Return, Semicolon,
        RBrace,
        Int, var("x"), Assign, IntNum(10), Semicolon,
        Int, var("step-size"), Assign, Minus, IntNum(1), Semicolon,
        Int, var("sum"), Assign, var("abs"), LParen, Minus, IntNum(5), Mult, IntNum(2), RParen, Minus, IntNum(10), Semicolon,
        While, LParen, LParen, var("x"), Gt, IntNum(0), And, var("sum"), Ne, IntNum(100), RParen, Or, var("x"), Eq, IntNum(10), RParen, LBrace,
        var("sum"), Assign, var("add"), LParen, var("sum"), Comma, var("x"), Mult, LParen, IntNum(2), Minus, IntNum(1), RParen, RParen, Semicolon,
        var("x"), Assign, var("x"), Plus, var("step-size"), Semicolon,
        RBrace,
        If, LParen, var("sum"), Ge, IntNum(55), And, LParen, var("sum"), Ne, IntNum(0), Or, var("x"), Eq, IntNum(0), RParen, RParen, LBrace,
        var("sum"), Assign, var("sum"), Div, IntNum(5), Semicolon,
        RBrace, Else, LBrace,
        var("sum"), Assign, Minus, LParen, var("sum"), Plus, IntNum(1), RParen, Semicolon,
        RBrace,
        var("count-down"), LParen, var("sum"), RParen, Semicolon,
        RBrace,
    ]);
}
