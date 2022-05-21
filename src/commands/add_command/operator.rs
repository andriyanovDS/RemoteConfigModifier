use std::fmt::{Display, Formatter, Write};

pub trait Operator: Display {
    type Item;
    fn to_condition(&self, condition_name: &str, value: &Self::Item) -> String;
}

#[derive(Clone, PartialEq)]
pub enum BinaryOperator {
    Less,
    LessEq,
    Eq,
    More,
    MoreEq,
    BangEq,
}

#[derive(Clone, PartialEq)]
pub enum SetOperator {
    Contains,
    NotContains,
    Matches,
    ExactlyMatches,
    In,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(self))
    }
}

impl Into<&'static str> for &BinaryOperator {
    fn into(self) -> &'static str {
        match self {
            BinaryOperator::Less => "<",
            BinaryOperator::LessEq => "<=",
            BinaryOperator::Eq => "==",
            BinaryOperator::BangEq => "!=",
            BinaryOperator::More => ">",
            BinaryOperator::MoreEq => ">=",
        }
    }
}

impl Operator for BinaryOperator {
    type Item = String;
    fn to_condition(&self, condition_name: &str, value: &Self::Item) -> String {
        format!("{}.{}(['{}'])", condition_name, self, value)
    }
}

impl Display for SetOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(self))
    }
}

impl Into<&'static str> for &SetOperator {
    fn into(self) -> &'static str {
        match self {
            SetOperator::Contains => "contains",
            SetOperator::NotContains => "notContains",
            SetOperator::Matches => "matches",
            SetOperator::ExactlyMatches => "exactlyMatches",
            SetOperator::In => "in",
        }
    }
}

impl Operator for SetOperator {
    type Item = Vec<String>;
    fn to_condition(&self, condition_name: &str, value: &Self::Item) -> String {
        let value = match value.first() {
            None => String::new(),
            Some(first) => {
                let mut result = String::with_capacity(value.len() * first.len() + value.len() * 3);
                write!(&mut result, "'{}'", first).unwrap();
                value.iter().skip(1).for_each(|item| {
                    result.push_str(",");
                    write!(&mut result, "'{}'", item).unwrap();
                });
                result
            }
        };
        match self {
            Self::In => format!("{} {} [{}]", condition_name, self, value),
            _ => format!("{}.{}([{}])", condition_name, self, value),
        }
    }
}
