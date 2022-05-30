use super::operator::{BinaryOperator, Operator, SetOperator};
use crate::editor::Editor;
use crate::error::{Error, Result};
use crate::io::{self, InputReader};
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

pub struct ExpressionBuilder<'a, E: Editor> {
    input_reader: &'a mut InputReader<E>,
    app_ids: &'a [String],
}

impl<'a, E: Editor> ExpressionBuilder<'a, E> {
    pub fn new(input_reader: &'a mut InputReader<E>, app_ids: &'a [String]) -> Self {
        Self {
            input_reader,
            app_ids,
        }
    }

    pub fn build(&mut self) -> Option<String> {
        loop {
            let items = ExpressionListItem::into_enum_iter().map(Into::into);
            let index = io::request_select_item_in_list("Select condition:", items, None)?;
            let expression_item = ExpressionListItem::into_enum_iter().nth(index).unwrap();
            let expression = self.build_for_item(&expression_item);
            if expression.is_some() {
                return expression;
            }
        }
    }

    fn build_for_item(&mut self, item: &ExpressionListItem) -> Option<String> {
        match item {
            ExpressionListItem::AppId => Self::build_app_id_expr(self.app_ids),
            ExpressionListItem::DeviceOS => {
                let operator =
                    Self::select_operator(&[BinaryOperator::Eq, BinaryOperator::BangEq])?;
                let expression = Expression {
                    name: "device.os",
                    operator,
                    value: self.select_single_condition_value("device OS"),
                };
                Some(expression.to_string())
            }
            ExpressionListItem::DeviceDateTime => {
                let operator =
                    Self::select_operator(&[BinaryOperator::LessEq, BinaryOperator::More])?;
                let expression = Expression {
                    name: "device.dateTime",
                    operator,
                    value: self.select_single_condition_value("device date time"),
                };
                Some(expression.to_string())
            }
            ExpressionListItem::DeviceCountry => {
                let expression = Expression {
                    name: "device.country",
                    operator: SetOperator::In,
                    value: self.select_multiple_condition_values("device device countries"),
                };
                Some(expression.to_string())
            }
            ExpressionListItem::DeviceLanguage => {
                let expression = Expression {
                    name: "device.language",
                    operator: SetOperator::In,
                    value: self.select_multiple_condition_values("device device languages"),
                };
                Some(expression.to_string())
            }
            ExpressionListItem::AppBuild => {
                let app_id_expr = Self::build_app_id_expr(self.app_ids)?;
                let expression =
                    self.select_from_different_operators("app.build", "app build", "app builds")?;
                Some(format!("{} && {}", app_id_expr, expression.to_string()))
            }
            ExpressionListItem::AppVersion => {
                let app_id_expr = Self::build_app_id_expr(self.app_ids)?;
                let expression = self.select_from_different_operators(
                    "app.version",
                    "app version",
                    "app versions",
                )?;
                Some(format!("{} && {}", app_id_expr, expression.to_string()))
            }
            ExpressionListItem::UserProperty => {
                let app_id_expr = Self::build_app_id_expr(self.app_ids)?;
                let expression = self.select_from_different_operators(
                    "app.userProperty",
                    "user property",
                    "user properties",
                )?;
                Some(format!("{} && {}", app_id_expr, expression.to_string()))
            }
        }
    }

    fn select_from_different_operators(
        &mut self,
        expression_name: &'static str,
        label_for_single_value: &'static str,
        label_for_multiple_values: &'static str,
    ) -> Option<Expression<SetOperator>> where {
        let operators = ALL_SET_OPERATORS_EXCEPT_IN.iter().map(Into::into);
        let operator_index = io::request_select_item_in_list("Select operator:", operators, None)?;
        let operator = ALL_SET_OPERATORS_EXCEPT_IN[operator_index].clone();
        let value = match operator {
            SetOperator::Binary(_) => {
                vec![self.select_single_condition_value(label_for_single_value)]
            }
            _ => self.select_multiple_condition_values(label_for_multiple_values),
        };
        Some(Expression {
            name: expression_name,
            operator,
            value,
        })
    }

    fn select_single_condition_value(&mut self, label: &str) -> String {
        let title = format!("Enter {}:", label.green());
        self.input_reader.request_user_input::<str>(&title).unwrap()
    }

    fn select_multiple_condition_values(&mut self, label: &str) -> Vec<String> {
        let title = format!("Enter {} separated by the comma:", label.green());
        self.input_reader
            .request_user_input::<str>(&title)
            .unwrap()
            .split(',')
            .map(|v| v.trim().to_string())
            .collect()
    }

    fn build_app_id_expr(app_ids: &[String]) -> Option<String> {
        if app_ids.len() == 1 {
            return Some(app_ids[0].clone());
        }
        let app_ids_iter = app_ids.iter().map(|id| id.split(':').nth(2).unwrap());
        io::request_select_item_in_list("Select App ID:", app_ids_iter, None)
            .map(|index| format!("app.id == '{}'", app_ids[index]))
    }

    fn select_operator<T>(operators: &'static [T]) -> Option<T>
    where
        for<'b> &'b T: Into<&'static str>,
        T: Clone,
    {
        let items = operators.iter().map(Into::into);
        io::request_select_item_in_list("Select operator:", items, None)
            .map(|index| operators[index].clone())
    }
}

pub fn replace_app_id(expression: &mut String, app_ids: &[String]) -> Result<()> {
    let search_str = "app.id == '";
    let index = expression.find(search_str);

    if index.is_none() {
        return Ok(());
    }
    let app_id_start_index = index.unwrap() + search_str.len();
    let app_id_end_index = expression[app_id_start_index..].find('\'').map(|i| i - 1);
    if app_id_end_index.is_none() {
        return Ok(());
    }
    let app_id_end_index = app_id_end_index.unwrap() + app_id_start_index;
    let platform = expression[app_id_start_index..app_id_end_index]
        .split(':')
        .nth(2)
        .unwrap();
    let replacement = app_ids
        .iter()
        .find(|app_id| app_id.split(':').nth(2).unwrap() == platform);
    if replacement.is_none() {
        let message =
            format!("App ID for compatible platform {platform} was not found for this project");
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

impl<'a> From<ExpressionListItem> for &'static str {
    fn from(item: ExpressionListItem) -> &'static str {
        match item {
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
        self.operator.to_condition(self.name, &self.value)
    }
}
