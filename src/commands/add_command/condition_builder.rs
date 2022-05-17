use super::conditions::AppBuildCondition;
use super::conditions::{BinaryOperator, SetOperator};
use super::conditions::{
    AppIdCondition, AppVersionCondition, DeviceCountryCondition, DeviceDateTimeCondition,
    DeviceLanguageCondition, DeviceOSCondition, UserPropertyCondition,
};
use crate::io::{InputReader, InputString};
use enum_iterator::IntoEnumIterator;

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

#[derive(IntoEnumIterator, Clone)]
pub enum ConditionListItem {
    AppBuild,
    AppVersion,
    AppId,
    UserProperty,
    DeviceCountry,
    DeviceDateTime,
    DeviceLanguage,
    DeviceOS,
}

impl ConditionListItem {
    pub async fn select_condition(app_ids: &[String]) -> Option<String> {
        let items = ConditionListItem::into_enum_iter().map(Into::into);
        println!();
        let index =
            InputReader::request_select_item_in_list("Select condition:", items, None, true).await;
        match index {
            Some(index) => {
                let item = ConditionListItem::into_enum_iter()
                    .nth(index)
                    .unwrap()
                    .clone();
                Some(item.build(app_ids).await)
            }
            None => None,
        }
    }

    pub async fn build(self, app_ids: &[String]) -> String {
        match self {
            Self::AppId => {
                let value = select_app_id(app_ids).await;
                AppIdCondition { value }.to_string()
            }
            Self::DeviceOS => {
                let (condition, value) = tokio::join!(
                    select_operator(&[BinaryOperator::Eq, BinaryOperator::BangEq]),
                    select_single_condition_value("device OS")
                );
                let condition = DeviceOSCondition {
                    value,
                    is_equal: condition == BinaryOperator::Eq,
                };
                condition.to_string()
            }
            Self::DeviceDateTime => {
                let (condition, value) = tokio::join!(
                    select_operator(&[BinaryOperator::LessEq, BinaryOperator::More]),
                    select_single_condition_value("device date time")
                );
                let condition = DeviceDateTimeCondition {
                    value,
                    is_more: condition == BinaryOperator::More,
                };
                condition.to_string()
            }
            Self::DeviceCountry => {
                let value = select_multiple_condition_values("device device countries").await;
                let condition = DeviceCountryCondition { value };
                condition.to_string()
            }
            Self::DeviceLanguage => {
                let value = select_multiple_condition_values("device device languages").await;
                let condition = DeviceLanguageCondition { value };
                condition.to_string()
            }
            Self::AppBuild => {
                let app_id = select_app_id(app_ids).await;
                select_from_different_operators(
                    &ALL_BINARY_OPERATORS,
                    &SET_OPERATORS_WITHOUT_IN,
                    "app build",
                    "app builds",
                    app_id,
                    |operator, value, app_id| {
                        let condition = AppBuildCondition {
                            operator,
                            value,
                            app_id_expression: AppIdCondition { value: app_id },
                        };
                        condition.to_string()
                    },
                    |operator, value, app_id| {
                        let condition = AppBuildCondition {
                            operator,
                            value,
                            app_id_expression: AppIdCondition { value: app_id },
                        };
                        condition.to_string()
                    },
                )
                .await
            }
            Self::AppVersion => {
                let app_id = select_app_id(app_ids).await;
                select_from_different_operators(
                    &ALL_BINARY_OPERATORS,
                    &SET_OPERATORS_WITHOUT_IN,
                    "app version",
                    "app versions",
                    app_id,
                    |operator, value, app_id| {
                        let condition = AppVersionCondition {
                            operator,
                            value,
                            app_id_expression: AppIdCondition { value: app_id },
                        };
                        condition.to_string()
                    },
                    |operator, value, app_id| {
                        let condition = AppVersionCondition {
                            operator,
                            value,
                            app_id_expression: AppIdCondition { value: app_id },
                        };
                        condition.to_string()
                    },
                )
                .await
            }
            Self::UserProperty => {
                select_from_different_operators(
                    &ALL_BINARY_OPERATORS,
                    &SET_OPERATORS_WITHOUT_IN,
                    "user property",
                    "user properties",
                    String::new(),
                    |operator, value, _| {
                        let condition = UserPropertyCondition { operator, value };
                        condition.to_string()
                    },
                    |operator, value, _| {
                        let condition = UserPropertyCondition { operator, value };
                        condition.to_string()
                    },
                )
                .await
            }
        }
    }
}

impl<'a> Into<&'static str> for ConditionListItem {
    fn into(self) -> &'static str {
        match self {
            ConditionListItem::AppBuild => "App build",
            ConditionListItem::AppVersion => "App version",
            ConditionListItem::UserProperty => "User property",
            ConditionListItem::AppId => "App ID",
            ConditionListItem::DeviceCountry => "Device country",
            ConditionListItem::DeviceLanguage => "Device language",
            ConditionListItem::DeviceOS => "Device OS",
            ConditionListItem::DeviceDateTime => "Device date time",
        }
    }
}

async fn select_app_id(app_ids: &[String]) -> String {
    if app_ids.len() == 1 {
        return app_ids[0].clone();
    }
    let app_ids_iter = app_ids.iter().map(|id| id.as_str());
    println!();
    let index =
        InputReader::request_select_item_in_list("Select App ID:", app_ids_iter, None, false)
            .await
            .unwrap();
    app_ids[index].clone()
}

async fn select_from_different_operators<BF, SF, R>(
    binary_operators: &'static [BinaryOperator],
    set_operators: &'static [SetOperator],
    label_for_single_value: &'static str,
    label_for_multiple_values: &'static str,
    app_id: String,
    binary_condition_factory: BF,
    set_condition_factory: SF,
) -> R
where
    BF: FnOnce(BinaryOperator, String, String) -> R,
    SF: FnOnce(SetOperator, Vec<String>, String) -> R,
{
    let binary_items = binary_operators.iter().map(Into::into);
    let set_items = set_operators.iter().map(Into::into);
    let operators_iter = binary_items.chain(set_items);
    println!();
    let operator_index =
        InputReader::request_select_item_in_list("Select operator:", operators_iter, None, false)
            .await
            .unwrap();
    if operator_index < binary_operators.len() {
        let value = select_single_condition_value(label_for_single_value).await;
        binary_condition_factory(binary_operators[operator_index].clone(), value, app_id)
    } else {
        let value = select_multiple_condition_values(label_for_multiple_values).await;
        set_condition_factory(
            set_operators[operator_index - binary_operators.len()].clone(),
            value,
            app_id,
        )
    }
}

async fn select_operator<T>(operators: &'static [T]) -> T
where
    for<'a> &'a T: Into<&'static str>,
    T: Clone,
{
    let items = operators.iter().map(Into::into);
    let operator_index =
        InputReader::request_select_item_in_list("Select operator:", items, None, false)
            .await
            .unwrap();
    operators[operator_index].clone()
}

async fn select_single_condition_value(label: &str) -> String {
    let title = format!("Enter {}:", label);
    InputReader::request_user_input::<InputString, str>(&title)
        .await
        .unwrap()
        .0
}

async fn select_multiple_condition_values(label: &str) -> Vec<String> {
    let title = format!("Enter {}:", label);
    let input = InputReader::request_user_input::<InputString, str>(&title)
        .await
        .unwrap()
        .0;
    input.split(",").map(|v| v.trim().to_string()).collect()
}
