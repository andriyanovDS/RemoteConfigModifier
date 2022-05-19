use super::operator::{BinaryOperator, Operator, SetOperator};
use crate::io::InputReader;
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

pub async fn build_expression(app_ids: &[String]) -> Option<String> {
    let items = ExpressionListItem::into_enum_iter().map(Into::into);
    println!();
    let index =
        InputReader::request_select_item_in_list("Select condition:", items, None, true).await;
    match index {
        Some(index) => {
            let item = ExpressionListItem::into_enum_iter()
                .nth(index)
                .unwrap()
                .clone();
            Some(item.build(app_ids).await)
        }
        None => None,
    }
}

#[derive(IntoEnumIterator, Clone)]
enum ExpressionListItem {
    AppBuild,
    AppVersion,
    AppId,
    UserProperty,
    DeviceCountry,
    DeviceDateTime,
    DeviceLanguage,
    DeviceOS,
}

struct Expression<O: Operator> {
    name: &'static str,
    operator: O,
    value: O::Item,
}

impl ExpressionListItem {
    async fn build(self, app_ids: &[String]) -> String {
        match self {
            Self::AppId => build_app_id_expr(app_ids).await,
            Self::DeviceOS => {
                let (operator, value) = tokio::join!(
                    select_operator(&[BinaryOperator::Eq, BinaryOperator::BangEq]),
                    select_single_condition_value("device OS")
                );
                Expression {
                    name: "device.os",
                    operator,
                    value,
                }
                .to_string()
            }
            Self::DeviceDateTime => {
                let (operator, value) = tokio::join!(
                    select_operator(&[BinaryOperator::LessEq, BinaryOperator::More]),
                    select_single_condition_value("device date time")
                );
                Expression {
                    name: "device.dateTime",
                    operator,
                    value,
                }
                .to_string()
            }
            Self::DeviceCountry => {
                let value = select_multiple_condition_values("device device countries").await;
                Expression {
                    name: "device.country",
                    operator: SetOperator::In,
                    value,
                }
                .to_string()
            }
            Self::DeviceLanguage => {
                let value = select_multiple_condition_values("device device languages").await;
                Expression {
                    name: "device.language",
                    operator: SetOperator::In,
                    value,
                }
                .to_string()
            }
            Self::AppBuild => {
                let app_id_expr = build_app_id_expr(app_ids).await;
                let expression =
                    select_from_different_operators("app.build", "app build", "app builds").await;
                format!("{} && {}", app_id_expr, expression)
            }
            Self::AppVersion => {
                let app_id_expr = build_app_id_expr(app_ids).await;
                let expression =
                    select_from_different_operators("app.version", "app version", "app versions")
                        .await;
                format!("{} && {}", app_id_expr, expression)
            }
            Self::UserProperty => {
                let app_id_expr = build_app_id_expr(app_ids).await;
                let expression = select_from_different_operators(
                    "app.userProperty",
                    "user property",
                    "user properties",
                )
                .await;
                format!("{} && {}", app_id_expr, expression)
            }
        }
    }
}

impl<'a> Into<&'static str> for ExpressionListItem {
    fn into(self) -> &'static str {
        match self {
            ExpressionListItem::AppBuild => "App build",
            ExpressionListItem::AppVersion => "App version",
            ExpressionListItem::UserProperty => "User property",
            ExpressionListItem::AppId => "App ID",
            ExpressionListItem::DeviceCountry => "Device country",
            ExpressionListItem::DeviceLanguage => "Device language",
            ExpressionListItem::DeviceOS => "Device OS",
            ExpressionListItem::DeviceDateTime => "Device date time",
        }
    }
}

impl<O: Operator> ToString for Expression<O> {
    fn to_string(&self) -> String {
        self.operator.to_condition(&self.name, &self.value)
    }
}

async fn build_app_id_expr(app_ids: &[String]) -> String {
    let app_id = select_app_id(app_ids).await;
    format!("app.id == '{}'", app_id)
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

async fn select_from_different_operators(
    expression_name: &'static str,
    label_for_single_value: &'static str,
    label_for_multiple_values: &'static str,
) -> String where {
    let binary_items = ALL_BINARY_OPERATORS.iter().map(Into::into);
    let set_items = SET_OPERATORS_WITHOUT_IN.iter().map(Into::into);
    let operators_iter = binary_items.chain(set_items);
    println!();
    let operator_index =
        InputReader::request_select_item_in_list("Select operator:", operators_iter, None, false)
            .await
            .unwrap();
    if operator_index < ALL_BINARY_OPERATORS.len() {
        Expression {
            name: expression_name,
            operator: ALL_BINARY_OPERATORS[operator_index].clone(),
            value: select_single_condition_value(label_for_single_value).await,
        }
        .to_string()
    } else {
        Expression {
            name: expression_name,
            operator: SET_OPERATORS_WITHOUT_IN[operator_index - ALL_BINARY_OPERATORS.len()].clone(),
            value: select_multiple_condition_values(label_for_multiple_values).await,
        }
        .to_string()
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
    InputReader::request_user_input_string::<str>(&title)
        .await
        .unwrap()
}

async fn select_multiple_condition_values(label: &str) -> Vec<String> {
    let title = format!("Enter {} separated by the comma:", label);
    let input = InputReader::request_user_input_string::<str>(&title)
        .await
        .unwrap();
    input.split(",").map(|v| v.trim().to_string()).collect()
}
