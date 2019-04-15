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

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    nom::is_hex_digit(c as u8)
}

named!(parse_hexcode<&str, u8>, 
       map_res!(take_while_m_n!(1, 2, is_hex_digit), from_hex));

named!(parse_quoted <&str, &str>, 
       delimited!(tag!("\""), take_until!("\""), tag!("\"")));

named!(parse_hexlist<&str, Vec<u8>>, 
       ws!(many0!(parse_hexcode)));

named!(parse_token_value<&str, TokenValue>,
       alt!(
           parse_quoted => { |value| TokenValue::Label(value) } |
           parse_hexcode => { |value| TokenValue::Constant(value) }
       )
);

named!(parse_instruction<&str, Vec<Token>>,
       ws!(do_parse!(
               opcode: parse_opcode >>
               register: parse_hexcode >>
               args: opt!(parse_token_value) >>
               opt!(tag!(":")) >>
               label_head: opt!(parse_quoted) >>
               opt!(tag!(":")) >>
               label_tail: opt!(parse_quoted) >>
               (
                   vec![
                   Token {
                       value: TokenValue::Constant(opcode + register),
                       label: label_head,
                   },
                   Token {
                       value: args.unwrap_or(TokenValue::Constant(0x00)),
                       label: label_tail,
                   }
                   ]
               )))
       );

#[derive(Debug, PartialEq)]
enum DataSegmentValue<'a> {
    String(&'a str),
    Data(Vec<u8>),
}

impl<'a> Into<Vec<Token<'a>>> for DataSegmentValue<'a> {
    fn into(self) -> Vec<Token<'a>> {
        match self {
            DataSegmentValue::String(string) => string.bytes().map(Token::from_const).collect(),
            DataSegmentValue::Data(data) => {
                data.iter().map(|value| Token::from_const(*value)).collect()
            }
        }
    }
}

named!(parse_datasegment_value<&str, DataSegmentValue>, alt!(
        parse_quoted => { |value| DataSegmentValue::String(value) } |
        parse_hexlist => { |value| DataSegmentValue::Data(value) }
));

named!(parse_datasegment<&str, Vec<Token>>,
       ws!(do_parse!(
               tag!("DATA") >>
               data: parse_datasegment_value >>
               opt!(tag!(":")) >>
               label_head: opt!(parse_quoted) >>
               opt!(tag!(":")) >>
               label_tail: opt!(parse_quoted) >>
               ({
                   let mut tokens: Vec<Token> = data.into();

                   // Head and tail labels
                   tokens.first_mut().unwrap().label = label_head;
                   tokens.last_mut().unwrap().label = label_tail;

                   // 2-byte alignment
                   if tokens.len() % 2 != 0 { tokens.push(Token::from_const(0x00)) }
                   tokens
               })
       )));

named!(pub parse_token<&str, Vec<Token>>, alt!( 
    parse_datasegment | parse_instruction | tag!("//") => { |_| vec![] }
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
            parse_instruction("LMEM 3 ;"),
            Ok((";", vec![Token::from_const(0x13), Token::from_const(0x00)]))
        );
        assert_eq!(
            parse_instruction("LCON 3 \"test\" ;"),
            Ok((
                ";",
                vec![Token::from_const(0x23), Token::from_label("test")]
            ))
        );
        assert_eq!(
            parse_instruction("LCON 3 03 ;"),
            Ok((";", vec![Token::from_const(0x23), Token::from_const(0x03)]))
        );
    }

    #[test]
    fn test_parse_hexlist() {
        assert_eq!(
            parse_hexlist("4A 56 32 w"),
            Ok(("w", vec![0x4Au8, 0x56u8, 0x32u8]))
        );
        assert_eq!(
            parse_hexlist(" 4A 56 32 w"),
            Ok(("w", vec![0x4Au8, 0x56u8, 0x32u8]))
        );
    }

    #[test]
    fn test_parse_datasegment_value() {
        assert_eq!(
            parse_datasegment_value("\"test\""),
            Ok(("", DataSegmentValue::String("test")))
        );
        assert_eq!(
            parse_datasegment_value("45 0 45 w"),
            Ok(("w", DataSegmentValue::Data(vec![0x45, 0x00, 0x45])))
        );
    }
}
