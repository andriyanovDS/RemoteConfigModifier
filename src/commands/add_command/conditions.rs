use std::fmt::{Display, Formatter};

const ALL_BINARY_OPERATORS: [BinaryOperator; 6] = [
    BinaryOperator::Less,
    BinaryOperator::LessEq,
    BinaryOperator::Eq,
    BinaryOperator::BangEq,
    BinaryOperator::More,
    BinaryOperator::MoreEq,
];
const SET_OPERATORS_WITHOUT_IN: [SetOperator; 4] = [
    SetOperator::Contains,
    SetOperator::NotContains,
    SetOperator::Matches,
    SetOperator::ExactlyMatches,
];
const IN_OPERATOR: [SetOperator; 1] = [SetOperator::In];

pub(super) trait Condition: ToString {
    fn available_binary_operators() -> &'static [BinaryOperator];
    fn available_set_operators() -> &'static [SetOperator];
    fn list_representation() -> &'static str;
}

pub(super) struct AppBuildCondition<O: Operator> {
    app_id_condition: AppIdCondition,
    value: O::Item,
    operator: O,
}

pub(super) struct AppVersionCondition<O: Operator> {
    app_id_condition: AppIdCondition,
    value: O::Item,
    operator: O,
}

pub(super) struct AppIdCondition {
    value: String
}

pub(super) struct UserPropertyCondition<O: Operator> {
    value: O::Item,
    operator: O,
}

pub(super) struct DeviceCountryCondition {
    value: Vec<String>,
}

pub(super) struct DeviceDateTimeCondition {
    value: String,
    is_more: bool,
}

pub(super) struct DeviceLanguageCondition {
    value: Vec<String>,
}

pub(super) struct DeviceOSCondition {
    value: String,
    is_equal: bool,
}

impl<O: Operator> ToString for AppBuildCondition<O> {
    fn to_string(&self) -> String {
        format!("{} && {}", self.app_id_condition.to_string(), self.operator.to_condition("app.build", &self.value))
    }
}

impl<O: Operator> Condition for AppBuildCondition<O> {
    fn available_binary_operators() -> &'static [BinaryOperator] {
        &ALL_BINARY_OPERATORS
    }
    fn available_set_operators() -> &'static [SetOperator] {
        &SET_OPERATORS_WITHOUT_IN
    }
    fn list_representation() -> &'static str {
        "App build"
    }
}

impl<O: Operator> ToString for AppVersionCondition<O> {
    fn to_string(&self) -> String {
        format!("{} && {}", self.app_id_condition.to_string(), self.operator.to_condition("app.version", &self.value))
    }
}

impl<O: Operator> Condition for AppVersionCondition<O> {
    fn available_binary_operators() -> &'static [BinaryOperator] {
        &ALL_BINARY_OPERATORS
    }
    fn available_set_operators() -> &'static [SetOperator] {
        &SET_OPERATORS_WITHOUT_IN
    }
    fn list_representation() -> &'static str {
        "App version"
    }
}

impl ToString for AppIdCondition {
    fn to_string(&self) -> String {
        format!("app.id == {}", self.value)
    }
}

impl Condition for AppIdCondition {
    fn available_binary_operators() -> &'static [BinaryOperator] {
        &[BinaryOperator::Eq]
    }
    fn available_set_operators() -> &'static [SetOperator] {
        &[]
    }
    fn list_representation() -> &'static str {
        "App ID"
    }
}

impl<O: Operator> ToString for UserPropertyCondition<O> {
    fn to_string(&self) -> String {
        self.operator.to_condition("app.userProperty", &self.value)
    }
}

impl<O: Operator> Condition for UserPropertyCondition<O> {
    fn available_binary_operators() -> &'static [BinaryOperator] {
        &ALL_BINARY_OPERATORS
    }
    fn available_set_operators() -> &'static [SetOperator] {
        &SET_OPERATORS_WITHOUT_IN
    }
    fn list_representation() -> &'static str {
        "User condition"
    }
}

impl ToString for DeviceCountryCondition {
    fn to_string(&self) -> String {
        format!("device.country in {:?}", self.value)
    }
}

impl Condition for DeviceCountryCondition {
    fn available_binary_operators() -> &'static [BinaryOperator] {
        &[]
    }
    fn available_set_operators() -> &'static [SetOperator] {
        &IN_OPERATOR
    }
    fn list_representation() -> &'static str {
        "Device country"
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

impl Condition for DeviceDateTimeCondition {
    fn available_binary_operators() -> &'static [BinaryOperator] {
        &[BinaryOperator::LessEq, BinaryOperator::More]
    }
    fn available_set_operators() -> &'static [SetOperator] {
        &[]
    }
    fn list_representation() -> &'static str {
        "Device date time"
    }
}

impl ToString for DeviceLanguageCondition {
    fn to_string(&self) -> String {
        format!("device.language in {:?}", self.value)
    }
}

impl Condition for DeviceLanguageCondition {
    fn available_binary_operators() -> &'static [BinaryOperator] {
        &[]
    }
    fn available_set_operators() -> &'static [SetOperator] {
        &IN_OPERATOR
    }
    fn list_representation() -> &'static str {
        "Device language"
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

impl Condition for DeviceOSCondition {
    fn available_binary_operators() -> &'static [BinaryOperator] {
        &[BinaryOperator::Eq, BinaryOperator::BangEq]
    }
    fn available_set_operators() -> &'static [SetOperator] {
        &[]
    }
    fn list_representation() -> &'static str {
        "Device OS"
    }
}

pub(super) trait Operator: Display {
    type Item;
    fn to_condition(&self, condition_name: &str, value: &Self::Item) -> String;
}

pub(super) enum BinaryOperator {
    Less,
    LessEq,
    Eq,
    More,
    MoreEq,
    BangEq,
}

pub(super) enum SetOperator {
    Contains,
    NotContains,
    Matches,
    ExactlyMatches,
    In,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let operator = match self {
            Self::Less => "<",
            Self::LessEq => "<=",
            Self::Eq => "==",
            Self::BangEq => "!=",
            Self::More => ">",
            Self::MoreEq => ">="
        };
        write!(f, "{}", operator)
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
        let operator = match self {
            Self::Contains => "contains",
            Self::NotContains => "notContains",
            Self::Matches => "matches",
            Self::ExactlyMatches => "exactlyMatches",
            Self::In => "in",
        };
        write!(f, "{}", operator)
    }
}

impl Operator for SetOperator {
    type Item = Vec<String>;
    fn to_condition(&self, condition_name: &str, value: &Self::Item) -> String {
        match self {
            Self::In => format!("{} {} {:?}", condition_name, self, value),
            _ => format!("{} {}.({:?})", condition_name, self, value)
        }
    }
}
