use super::expression_builder::ExpressionBuilder;
use crate::editor::Editor;
use crate::error::{Error, Result};
use crate::io::{self, InputReader};
use crate::remote_config::{Condition, Parameter, ParameterValue, ParameterValueType, TagColor};
use color_eyre::owo_colors::colors::Green;
use color_eyre::owo_colors::{FgColorDisplay, OwoColorize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::string::String;
use tracing::{info, warn};

pub struct ParameterBuilder<'a, E: Editor> {
    parts: Parts,
    input_reader: &'a mut InputReader<E>,
    app_ids: &'a [String],
    conditions: &'a mut Vec<Condition>,
}

#[derive(Debug)]
struct Parts {
    name: String,
    description: Option<String>,
    default_value: ParameterValue,
    value_type: ParameterValueType,
    conditional_values: HashMap<String, ParameterValue>,
}

impl<'a, E: Editor> ParameterBuilder<'a, E> {
    pub fn new_from_parameter(
        name: String,
        parameter: &Parameter,
        input_reader: &'a mut InputReader<E>,
        app_ids: &'a [String],
        conditions: &'a mut Vec<Condition>,
    ) -> Self {
        let parts = Parts::new_from_parameter(name, parameter);
        Self {
            parts,
            input_reader,
            app_ids,
            conditions,
        }
    }

    pub fn start_flow(
        name: Option<String>,
        description: Option<String>,
        input_reader: &'a mut InputReader<E>,
        app_ids: &'a [String],
        conditions: &'a mut Vec<Condition>,
    ) -> (String, Parameter) {
        let name = name.map(Parts::validate_name).and_then(|result| {
            if let Err(message) = &result {
                warn!("{}", message.yellow());
            };
            result.ok()
        });
        let name = name.unwrap_or_else(|| Self::request_name(input_reader));
        let parts = Parts {
            name,
            description,
            default_value: Default::default(),
            value_type: Default::default(),
            conditional_values: HashMap::new(),
        };
        let mut builder = Self {
            parts,
            input_reader,
            app_ids,
            conditions,
        };
        if builder.parts.description.is_none() {
            builder.request_description();
        }
        builder
            .request_value_type()
            .request_default_value()
            .request_condition();

        builder.parts.parameter()
    }

    pub fn add_values<'b>(
        mut self,
        selected_conditions: impl Iterator<Item = &'b str>,
    ) -> Result<(String, Parameter)> {
        self.request_default_value();
        for condition in selected_conditions {
            self.request_value_for_condition(condition);
        }
        Ok(self.parts.parameter())
    }

    fn request_name(input_reader: &mut InputReader<E>) -> String {
        loop {
            let result = input_reader
                .request_user_input::<FgColorDisplay<Green, &str>>(&"Enter parameter name:".green())
                .map_err(|error| error.to_string())
                .and_then(|name| Parts::validate_name(name).map_err(|e| e.to_string()));

            match result {
                Ok(name) => {
                    return name;
                }
                Err(message) => {
                    warn!("{}", message.yellow())
                }
            }
        }
    }

    fn request_description(&mut self) {
        self.and_then("Enter description (Optional):", |parts, description| {
            parts.description = if description.is_empty() {
                None
            } else {
                Some(description)
            };
            Ok(())
        })
    }

    fn request_value_type(&mut self) -> &mut Self {
        let list = vec!["Boolean", "Number", "String", "JSON"];
        let values_iter = list.iter().copied();
        let label = "Select value type:".green().to_string();
        let index = io::request_select_item_in_list(&label, values_iter, None);
        match index {
            Some(index) => {
                let value_type = ParameterValueType::from(list[index]);
                self.parts.value_type = value_type;
            }
            None => self.request_description(),
        };
        self
    }

    fn request_default_value(&mut self) -> &mut Self {
        self.and_then("Enter default value:", |parts, value| {
            parts.set_default_value(value).map_err(Error::new)
        });
        self
    }

    fn request_condition(&mut self) -> &mut Self {
        let message = format!("{}", "Do you want to add conditional value? [Y,n]".green());
        if let Some(index) = self.select_condition(&message) {
            let condition_name = self.conditions[index].name.clone();
            self.request_value_for_condition(&condition_name);
            let message = format!(
                "{}",
                "Do you want to add additional conditional value? [Y,n]".green()
            );
            while let Some(selected_index) = self.select_condition(&message) {
                let condition_name = self.conditions[selected_index].name.clone();
                self.request_value_for_condition(&condition_name);
            }
        };
        self
    }

    fn request_value_for_condition(&mut self, condition_name: &str) {
        let message = format!("Enter value for {} condition:", &condition_name);
        let valid_value = loop {
            let result = self
                .input_reader
                .request_user_input::<FgColorDisplay<Green, String>>(&message.green())
                .map_err(|e| e.to_string())
                .and_then(|value| {
                    Parts::validate_value(value, &self.parts.value_type).map_err(|e| e.to_string())
                });

            match result {
                Ok(value) => {
                    break value;
                }
                Err(message) => warn!("{}", message.yellow()),
            }
        };
        self.parts.conditional_values.insert(
            condition_name.to_string(),
            ParameterValue::Value(valid_value),
        );
    }

    fn select_condition(&mut self, message: &str) -> Option<usize> {
        if !self.input_reader.ask_confirmation(message) {
            return None;
        }
        loop {
            let condition_names = self.conditions.iter().map(|cond| cond.name.as_str());
            let custom_option = Some("Create a new condition");
            let label = "Select one of available conditions:";
            let index = io::request_select_item_in_list(label, condition_names, custom_option);
            if index
                .map(|index| index < self.conditions.len())
                .unwrap_or(true)
            {
                return index;
            }
            if let Some(condition) = self.make_new_condition() {
                self.conditions.push(condition);
                return Some(self.conditions.len() - 1);
            }
        }
    }

    fn make_new_condition(&mut self) -> Option<Condition> {
        let label = "Write condition name:".green();
        loop {
            let name = loop {
                let name = self
                    .input_reader
                    .request_user_input::<FgColorDisplay<Green, &str>>(&label)
                    .unwrap();
                if self.conditions.iter().any(|cond| cond.name == name) {
                    warn!("Condition with name {} already exists.", name);
                } else {
                    break name;
                }
            };
            let mut expression_builder =
                ExpressionBuilder::new(self.input_reader, self.app_ids);
            let expression = expression_builder.build();
            if let Some(expression) = expression {
                info!(
                    "Condition '{}' with expression {} was added.",
                    name, expression
                );
                return Some(Condition {
                    name,
                    expression,
                    tag_color: TagColor::Green,
                });
            }
        }
    }

    fn and_then<F>(&mut self, request_msg: &'static str, parts_modifier: F)
    where
        F: Fn(&mut Parts, String) -> Result<()>,
    {
        loop {
            let result = self
                .input_reader
                .request_user_input::<FgColorDisplay<Green, &str>>(&request_msg.green())
                .and_then(|value| parts_modifier(&mut self.parts, value));
            match result {
                Ok(_) => {
                    return;
                }
                Err(error) => warn!("{}", error.message.yellow()),
            }
        }
    }
}

impl Parts {
    fn new_from_parameter(name: String, parameter: &Parameter) -> Self {
        Self {
            name,
            description: parameter.description.clone(),
            default_value: Default::default(),
            value_type: parameter.value_type.clone(),
            conditional_values: HashMap::with_capacity(parameter.conditional_values.len()),
        }
    }

    fn validate_name(name: String) -> std::result::Result<String, &'static str> {
        if name.is_empty() {
            return Err("Name must contain at least one character");
        }
        let mut characters = name.chars();
        let first_char = characters.next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err("Parameter name must start with an underscore or English letter character [A-Z, a-z]");
        }
        if characters.all(|char| char.is_ascii_alphanumeric() || char == '_') {
            Ok(name)
        } else {
            Err("Parameter name can only include English letter characters, numbers and underscore")
        }
    }

    fn validate_value(
        value: String,
        value_type: &ParameterValueType,
    ) -> std::result::Result<String, &'static str> {
        match value_type {
            ParameterValueType::Boolean => value
                .parse::<bool>()
                .map(|_| value)
                .map_err(|_| "Value must be a boolean"),
            ParameterValueType::Number => value
                .parse::<f32>()
                .map(|_| value)
                .map_err(|_| "Value must be numeric"),
            ParameterValueType::String => Ok(value),
            ParameterValueType::Json => serde_json::from_str::<Value>(&value)
                .map_err(|_| "Invalid JSON")
                .map(|_| value),
            ParameterValueType::Unspecified => panic!("Unsupported value type"),
        }
    }

    fn set_default_value(&mut self, value: String) -> std::result::Result<(), &'static str> {
        let value = Self::validate_value(value, &self.value_type)?;
        self.default_value = ParameterValue::Value(value);
        Ok(())
    }

    fn parameter(self) -> (String, Parameter) {
        let parameter = Parameter {
            default_value: Some(self.default_value),
            conditional_values: self.conditional_values,
            description: self.description,
            value_type: self.value_type,
        };
        (self.name, parameter)
    }
}

impl From<&str> for ParameterValueType {
    fn from(value: &str) -> Self {
        match value {
            "Boolean" => Self::Boolean,
            "JSON" => Self::Json,
            "Number" => Self::Number,
            "String" => Self::String,
            _ => panic!("Unexpected value"),
        }
    }
}

impl TryFrom<String> for ParameterValue {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Ok(Self::Value(value))
    }
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        error.message
    }
}

impl Default for ParameterValueType {
    fn default() -> Self {
        Self::String
    }
}

impl Default for ParameterValue {
    fn default() -> Self {
        Self::Value(String::new())
    }
}
