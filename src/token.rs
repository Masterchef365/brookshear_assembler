#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue<'a> {
    Constant(u8),
    Label(&'a str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub value: TokenValue<'a>,
    pub label: Option<&'a str>,
}

impl<'a> Token<'a> {
    pub fn from_const(constant: u8) -> Self {
        Self {
            value: TokenValue::Constant(constant),
            label: None,
        }
    }

    #[allow(dead_code)]
    pub fn from_label(label: &'a str) -> Self {
        Self {
            value: TokenValue::Label(label),
            label: None,
        }
    }

    #[allow(dead_code)]
    pub fn empty_label(value: TokenValue<'a>) -> Self {
        Self { value, label: None }
    }
}
