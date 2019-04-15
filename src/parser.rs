use crate::nom::*;
use crate::op_codes::*;
use crate::token::*;

named!(parse_opcode <&str, u8>, alt!(
        tag!("LMEM") => { |_| LMEM } |
        tag!("LCON") => { |_| LCON } |
        tag!("STOR") => { |_| STOR } |
        tag!("MOVE") => { |_| MOVE } |
        tag!("ADDR") => { |_| ADDR } |
        tag!("OR") => { |_| OR } |
        tag!("AND") => { |_| AND } |
        tag!("XOR") => { |_| XOR } |
        tag!("ROT") => { |_| ROT } |
        tag!("JUMP") => { |_| JUMP } |
        tag!("HALT") => { |_| HALT } |
        tag!("CHAR") => { |_| CHAR } |
        tag!("LESS") => { |_| LESS } |
        tag!("DISP") => { |_| DISP }
));

named!(parse_quoted <&str, &str>, delimited!(
        tag!("\""),
        take_until!("\""),
        tag!("\"")
));

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    nom::is_hex_digit(c as u8)
}

named!(parse_hexcode<&str, u8>,
       map_res!(take_while_m_n!(1, 2, is_hex_digit), from_hex)
);

named!(parse_token_value<&str, TokenValue>,
       alt!(
           parse_quoted => { |value| TokenValue::Label(value) } |
           parse_hexcode => { |value| TokenValue::Constant(value) }
       )
);

named!(additional_label_arg<&str, &str>, ws!(do_parse!(
                tag!(":") >>
                quoted: parse_quoted >> (quoted)
            )));

named!(pub parse_instruction<&str, Vec<Token>>,
       ws!(do_parse!(
               opcode: parse_opcode >>
               register: parse_hexcode >>
               args: opt!(parse_token_value) >>
               extra_1: opt!(additional_label_arg) >>
               extra_2: opt!(additional_label_arg) >>
               (
                   vec![
                        Token {
                            value: TokenValue::Constant(opcode + register),
                            label: extra_1,
                        },
                        Token {
                            value: args.unwrap_or(TokenValue::Constant(0x00)),
                            label: extra_2,
                        }
                   ]
               ))
       )
);

named!(pub parse_datasegment<&str, Vec<Token>>,
       ws!(do_parse!(
               tag!("DATA") >>
               quoted: parse_quoted >>
               extra_1: opt!(additional_label_arg) >>
               extra_2: opt!(additional_label_arg) >>
               ({
                   let mut tokens: Vec<Token> = quoted
                       .chars()
                       .map(|x| Token::from_const(x as u8))
                       .collect();
                   tokens.first_mut().unwrap().label = extra_1;
                   tokens.last_mut().unwrap().label = extra_2;
                   if tokens.len() % 2 != 0 { tokens.push(Token::from_const(0x00)) }
                   tokens
               })
               )));

named!(pub parse_token<&str, Vec<Token>>, alt!(
        parse_datasegment | parse_instruction
        ));

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Context::Code;
    use nom::Err::Error;
    use nom::Err::Incomplete;
    use nom::ErrorKind::{Alt, Tag, TakeWhileMN};
    use nom::Needed::Size;

    #[test]
    fn test_parse_opcode() {
        assert_eq!(parse_opcode("LMEM"), Ok(("", LMEM)));
        assert_eq!(parse_opcode("LESS "), Ok((" ", LESS)));
        assert_eq!(parse_opcode(" LESS "), Err(Error(Code(" LESS ", Alt))));
        assert_eq!(parse_opcode(""), Err(Incomplete(Size(4))));
        assert_eq!(parse_opcode(" "), Err(Error(Code(" ", Alt))));
    }

    #[test]
    fn test_parse_quoted() {
        assert_eq!(parse_quoted("\"test1\""), Ok(("", "test1")));
        assert_eq!(parse_quoted("\"test 2 test\""), Ok(("", "test 2 test")));
        assert_eq!(parse_quoted("\"2 "), Err(Incomplete(Size(1))));
        assert_eq!(parse_quoted("2 "), Err(Error(Code("2 ", Tag))));
    }

    #[test]
    fn test_parse_hexcode() {
        assert_eq!(parse_hexcode("02"), Ok(("", 0x02)));
        assert_eq!(parse_hexcode("3a"), Ok(("", 0x3A)));
        assert_eq!(parse_hexcode("3q"), Ok(("q", 0x03)));
        assert_eq!(parse_hexcode("q"), Err(Error(Code("q", TakeWhileMN))));
    }

    #[test]
    fn test_parse_token_value() {
        assert_eq!(
            parse_token_value("\"label\""),
            Ok(("", TokenValue::Label("label")))
        );
        assert_eq!(
            parse_token_value("0A"),
            Ok(("", TokenValue::Constant(0x0A)))
        );
    }

    #[test]
    fn test_parse_instruction() {
        assert_eq!(
            parse_instruction("LMEM 3 w"),
            Ok(("w", [Token::from_const(0x13), Token::from_const(0x00)]))
        );

        assert_eq!(
            parse_instruction("LCON 3 \"what\" w"),
            Ok((
                    "w",
                    [Token::from_const(0x23), Token::from_label("what")]
            ))
        );
    }
}
