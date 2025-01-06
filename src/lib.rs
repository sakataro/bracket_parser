use std::fmt::{self, Display};

#[derive(Debug)]
pub enum ParseError {
    HasNoClosing(usize),
}

#[derive(Debug)]
pub enum AST {
    Text(String),          // normal text
    Parenthesis(Box<AST>), // ()
    Curly(Box<AST>),       // {}
    Square(Box<AST>),      // []
    Tokens(Vec<AST>),      // normal text and {parenthesis (maybe nest) or not} or not
}

impl Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AST::Text(inner) => {
                write!(f, "AST::Text({})", inner)
            }
            AST::Parenthesis(inner) => {
                write!(f, "AST::Parenthesis({})", inner)
            }
            AST::Curly(inner) => {
                write!(f, "AST::Curly({})", inner)
            }
            AST::Square(inner) => {
                write!(f, "AST::Square({})", inner)
            }
            AST::Tokens(tokens) => {
                write!(f, "AST::Tokens([ ")?;
                for token in tokens {
                    write!(f, "{}, ", token)?;
                }
                write!(f, "]")
            }
        }
    }
}

pub fn parse(origin: &str) -> Result<AST, ParseError> {
    if origin.len() == 0 {
        return Ok(AST::Text(String::from("")));
    }
    let mut tokens: Vec<AST> = Vec::new();
    let mut last_string_start: usize = 0;
    let mut index: usize = 0;
    loop {
        let ch = match origin.chars().nth(index) {
            Some(c) => c,
            None => {
                break;
            }
        };
        match ch {
            '(' | '{' | '[' => {
                if last_string_start != index {
                    tokens.push(AST::Text(origin[last_string_start..index].to_string()));
                }
                let b: Bracket;
                if ch == '(' {
                    b = Bracket::Paren;
                } else if ch == '{' {
                    b = Bracket::Curly;
                } else {
                    b = Bracket::Square;
                }

                // NOTE: サーチとパースで2回走査するので多分遅い
                let end_index = match search_end_bracket(&origin[index..], &b) {
                    Some(i) => i,
                    None => {
                        return Err(ParseError::HasNoClosing(index));
                    }
                };

                let parsed_inner = match parse(&origin[(index + 1)..(index + end_index)]) {
                    Ok(ast) => ast,
                    Err(err) => {
                        return Err(err);
                    }
                };

                match b {
                    Bracket::Paren => tokens.push(AST::Parenthesis(Box::new(parsed_inner))),
                    Bracket::Curly => tokens.push(AST::Curly(Box::new(parsed_inner))),
                    Bracket::Square => tokens.push(AST::Square(Box::new(parsed_inner))),
                }

                last_string_start = index + end_index + 1;
                index = index + end_index + 1;
            }
            _ => {
                index += 1;
            }
        }
    }

    if last_string_start < origin.len() - 1 || origin.len() == 1 {
        tokens.push(AST::Text(origin[last_string_start..].to_string()));
    }

    if tokens.len() == 1 {
        return Ok(tokens.pop().unwrap());
    }

    Ok(AST::Tokens(tokens))
}

enum Bracket {
    Paren,
    Curly,
    Square,
}

// 渡された文字列から閉じ括弧の場所(文字列のindex)を検出する
// ↓の場合6を返す
// origin -> (12345).....
// 適切な閉じ括弧がない場合はNoneを返す
fn search_end_bracket(origin: &str, bracket: &Bracket) -> Option<usize> {
    let targets: (char, char);
    let mut target_count = 0;
    match bracket {
        Bracket::Paren => {
            targets = ('(', ')');
        }
        Bracket::Curly => {
            targets = ('{', '}');
        }
        Bracket::Square => {
            targets = ('[', ']');
        }
    }
    for (index, ch) in origin.chars().enumerate() {
        match ch {
            '(' | '{' | '[' => {
                if targets.0 == ch {
                    target_count = target_count + 1;
                }
            }
            ')' | '}' | ']' => {
                if targets.1 == ch {
                    target_count = target_count - 1;
                    if target_count == 0 {
                        return Some(index);
                    }
                }
            }
            _ => {}
        }
    }
    //最後まで見つからなかったのでNoneを返す
    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn search_end_bracket_paranthesis() {
        let origin = "(123456)texttext";
        assert_eq!(7, search_end_bracket(origin, &Bracket::Paren).unwrap());

        let double = "(123456(89))texttext";
        assert_eq!(11, search_end_bracket(double, &Bracket::Paren).unwrap());
    }
    #[test]
    fn search_end_bracket_curly() {
        let origin = "{123456}texttext";
        assert_eq!(7, search_end_bracket(origin, &Bracket::Curly).unwrap());

        let double = "{123456{89}}texttext";
        assert_eq!(11, search_end_bracket(double, &Bracket::Curly).unwrap());
    }
    #[test]
    fn search_end_bracket_square() {
        let origin = "[123456]texttext";
        assert_eq!(7, search_end_bracket(origin, &Bracket::Square).unwrap());

        let double = "[123456[89]]texttext";
        assert_eq!(11, search_end_bracket(double, &Bracket::Square).unwrap());
    }
    #[test]
    fn search_end_bracket_none() {
        let origin = "(123456texttext";
        assert_eq!(None, search_end_bracket(origin, &Bracket::Square));
    }

    #[test]
    fn parse_single_text() {
        let ast = parse("t").unwrap();
        if let AST::Text(str) = ast {
            assert_eq!(str, "t");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn parse_single_bracket() {
        let ast = parse("text(a)").unwrap();
        if let AST::Tokens(tokens) = ast {
            assert_eq!(tokens.len(), 2);
            assert!(matches!(tokens[0], AST::Text(_)));
            if let AST::Parenthesis(a) = &tokens[1] {
                assert!(matches!(**a, AST::Text(_)))
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        };
    }

    #[test]
    fn parse_single_curly() {
        let ast = parse("text{aaa}test").unwrap();
        if let AST::Tokens(tokens) = ast {
            assert_eq!(tokens.len(), 3);
            assert!(matches!(tokens[0], AST::Text(_)));
            if let AST::Curly(a) = &tokens[1] {
                assert!(matches!(**a, AST::Text(_)))
            } else {
                assert!(false)
            }
            assert!(matches!(tokens[2], AST::Text(_)));
        } else {
            assert!(false);
        };
    }

    #[test]
    fn parse_single_square() {
        let ast = parse("text[aaa]test").unwrap();
        if let AST::Tokens(tokens) = ast {
            assert_eq!(tokens.len(), 3);
            assert!(matches!(tokens[0], AST::Text(_)));
            if let AST::Square(a) = &tokens[1] {
                assert!(matches!(**a, AST::Text(_)))
            } else {
                assert!(false)
            }
            assert!(matches!(tokens[2], AST::Text(_)));
        } else {
            assert!(false);
        };
    }

    #[test]
    fn parse_not_closing_bracket() {
        match parse("text(aaa]test") {
            Err(e) => assert!(matches!(e, ParseError::HasNoClosing(4))),
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_nest() {
        let ast = parse("test inner(par{curly[square]curly}par)").unwrap();
        println!("{:?}", ast);
        // TODO:: 真面目に書く
        assert!(true)
    }
}
