use super::operator::{BinaryOperator, Operator, SetOperator};
use crate::io::InputReader;
use crate::error::{Result, Error};
use color_eyre::owo_colors::OwoColorize;
use enum_iterator::IntoEnumIterator;

const ALL_SET_OPERATORS_EXCEPT_IN: [SetOperator; 10] = [
    SetOperator::Contains,
    SetOperator::NotContains,
    SetOperator::Matches,
    SetOperator::ExactlyMatches,
    SetOperator::Binary(BinaryOperator::Less),
    SetOperator::Binary(BinaryOperator::LessEq),
    SetOperator::Binary(BinaryOperator::Eq),
    SetOperator::Binary(BinaryOperator::BangEq),
    SetOperator::Binary(BinaryOperator::More),
    SetOperator::Binary(BinaryOperator::MoreEq),
];

pub async fn build_expression(app_ids: &[String]) -> Option<String> {
    loop {
        let items = ExpressionListItem::into_enum_iter().map(Into::into);
        let index =
            InputReader::request_select_item_in_list("Select condition:", items, None).await;
        if index.is_none() {
            return None;
        }
        let expression = ExpressionListItem::into_enum_iter()
            .nth(index.unwrap())
            .unwrap()
            .build(app_ids)
            .await;
        match expression {
            Some(expression) => {
                return Some(expression);
            }
            None => {
                continue;
            }
        }
    }
}

pub fn replace_app_id(expression: &mut String, app_ids: &Vec<String>) -> Result<()> {
    let search_str = "app.id == '";
    let index = expression.find(search_str);
    
    if index.is_none() {
        return Ok(());
    }
    let app_id_start_index = index.unwrap() + search_str.len();
    let app_id_end_index = expression[app_id_start_index..].find("'").map(|i| i - 1);
    if app_id_end_index.is_none() {
        return Ok(());
    }
    let app_id_end_index = app_id_end_index.unwrap() + app_id_start_index;
    let platform = expression[app_id_start_index..app_id_end_index].split(":").nth(2).unwrap();
    let replacement = app_ids.iter().find(|app_id| {
        app_id.split(":").nth(2).unwrap() == platform
    });
    if replacement.is_none() {
        let message = format!("App ID for compatible platform {platform} was not found for this project");
        return Err(Error { message });
    }
    expression.replace_range(app_id_start_index..=app_id_end_index, replacement.unwrap());
    Ok(())
}

#[derive(IntoEnumIterator)]
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
    async fn build(&self, app_ids: &[String]) -> Option<String> {
        match self {
            Self::AppId => build_app_id_expr(app_ids).await,
            Self::DeviceOS => {
                let operator =
                    select_operator(&[BinaryOperator::Eq, BinaryOperator::BangEq]).await?;
                let expression = Expression {
                    name: "device.os",
                    operator,
                    value: select_single_condition_value("device OS").await,
                };
                Some(expression.to_string())
            }
            Self::DeviceDateTime => {
                let operator =
                    select_operator(&[BinaryOperator::LessEq, BinaryOperator::More]).await?;
                let expression = Expression {
                    name: "device.dateTime",
                    operator,
                    value: select_single_condition_value("device date time").await,
                };
                Some(expression.to_string())
            }
            Self::DeviceCountry => {
                let expression = Expression {
                    name: "device.country",
                    operator: SetOperator::In,
                    value: select_multiple_condition_values("device device countries").await,
                };
                Some(expression.to_string())
            }
            Self::DeviceLanguage => {
                let expression = Expression {
                    name: "device.language",
                    operator: SetOperator::In,
                    value: select_multiple_condition_values("device device languages").await,
                };
                Some(expression.to_string())
            }
            Self::AppBuild => {
                let app_id_expr = build_app_id_expr(app_ids).await?;
                let expression =
                    select_from_different_operators("app.build", "app build", "app builds").await?;
                Some(format!("{} && {}", app_id_expr, expression.to_string()))
            }
            Self::AppVersion => {
                let app_id_expr = build_app_id_expr(app_ids).await?;
                let expression =
                    select_from_different_operators("app.version", "app version", "app versions")
                        .await?;
                Some(format!("{} && {}", app_id_expr, expression.to_string()))
            }
            Self::UserProperty => {
                let app_id_expr = build_app_id_expr(app_ids).await?;
                let expression = select_from_different_operators(
                    "app.userProperty",
                    "user property",
                    "user properties",
                )
                .await?;
                Some(format!("{} && {}", app_id_expr, expression.to_string()))
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

async fn build_app_id_expr(app_ids: &[String]) -> Option<String> {
    if app_ids.len() == 1 {
        return Some(app_ids[0].clone());
    }
    let app_ids_iter = app_ids.iter().map(|id| id.split(":").nth(2).unwrap());
    InputReader::request_select_item_in_list("Select App ID:", app_ids_iter, None)
        .await
        .map(|index| format!("app.id == '{}'", app_ids[index]))
}
async fn select_from_different_operators(
    expression_name: &'static str,
    label_for_single_value: &'static str,
    label_for_multiple_values: &'static str,
) -> Option<Expression<SetOperator>> where {
    let operators = ALL_SET_OPERATORS_EXCEPT_IN.iter().map(Into::into);
    let operator_index =
        InputReader::request_select_item_in_list("Select operator:", operators, None).await;
    if operator_index.is_none() {
        return None;
    }
    let operator = ALL_SET_OPERATORS_EXCEPT_IN[operator_index.unwrap()].clone();
    let value = match operator {
        SetOperator::Binary(_) => {
            vec![select_single_condition_value(label_for_single_value).await]
        }
        _ => select_multiple_condition_values(label_for_multiple_values).await,
    };
    Some(Expression {
        name: expression_name,
        operator,
        value,
    })
}

async fn select_operator<T>(operators: &'static [T]) -> Option<T>
where
    for<'a> &'a T: Into<&'static str>,
    T: Clone,
{
    let items = operators.iter().map(Into::into);
    InputReader::request_select_item_in_list("Select operator:", items, None)
        .await
        .map(|index| operators[index].clone())
}

async fn select_single_condition_value(label: &str) -> String {
    let title = format!("Enter {}:", label.green());
    InputReader::request_user_input_string::<str>(&title)
        .await
        .unwrap()
}

async fn select_multiple_condition_values(label: &str) -> Vec<String> {
    let title = format!("Enter {} separated by the comma:", label.green());
    InputReader::request_user_input_string::<str>(&title)
        .await
        .unwrap()
        .split(",")
        .map(|v| v.trim().to_string())
        .collect()
}
