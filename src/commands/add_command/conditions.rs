use std::fmt::{Display, Formatter};

pub struct AppBuildCondition<O: Operator> {
    pub app_id_expression: AppIdCondition,
    pub value: O::Item,
    pub operator: O,
}

pub struct AppVersionCondition<O: Operator> {
    pub app_id_expression: AppIdCondition,
    pub value: O::Item,
    pub operator: O,
}

pub struct UserPropertyCondition<O: Operator> {
    pub value: O::Item,
    pub operator: O,
}

pub struct AppIdCondition {
    pub value: String,
}

pub struct DeviceCountryCondition {
    pub value: Vec<String>,
}

pub struct DeviceDateTimeCondition {
    pub value: String,
    pub is_more: bool,
}

pub struct DeviceLanguageCondition {
    pub value: Vec<String>,
}

pub struct DeviceOSCondition {
    pub value: String,
    pub is_equal: bool,
}

impl<O: Operator> ToString for AppBuildCondition<O> {
    fn to_string(&self) -> String {
        format!(
            "{} && {}",
            self.app_id_expression.to_string(),
            self.operator.to_condition("app.build", &self.value)
        )
    }
}

impl<O: Operator> ToString for AppVersionCondition<O> {
    fn to_string(&self) -> String {
        format!(
            "{} && {}",
            self.app_id_expression.to_string(),
            self.operator.to_condition("app.version", &self.value)
        )
    }
}

impl ToString for AppIdCondition {
    fn to_string(&self) -> String {
        format!("app.id == {}", self.value)
    }
}

impl<O: Operator> ToString for UserPropertyCondition<O> {
    fn to_string(&self) -> String {
        self.operator.to_condition("app.userProperty", &self.value)
    }
}

impl ToString for DeviceCountryCondition {
    fn to_string(&self) -> String {
        format!("device.country in {:?}", self.value)
    }
}

impl ToString for DeviceDateTimeCondition {
    fn to_string(&self) -> String {
        let operator = if self.is_more {
            BinaryOperator::More
        } else {
            BinaryOperator::LessEq
        };
        operator.to_condition("device.dateTime", &self.value)
    }
}

impl ToString for DeviceLanguageCondition {
    fn to_string(&self) -> String {
        format!("device.language in {:?}", self.value)
    }
}

impl ToString for DeviceOSCondition {
    fn to_string(&self) -> String {
        let operator = if self.is_equal {
            BinaryOperator::Eq
        } else {
            BinaryOperator::BangEq
        };
        operator.to_condition("device.os", &self.value)
    }
}

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
        format!("{} {} {}", condition_name, self, value)
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
        match self {
            Self::In => format!("{} {} {:?}", condition_name, self, value),
            _ => format!("{}.{}({:?})", condition_name, self, value),
        }
    }
}
