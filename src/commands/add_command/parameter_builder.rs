use crate::error::{Error, Result};
use crate::io::{InputReader, InputString};
use crate::remote_config::{Condition, Parameter, ParameterValue, ParameterValueType};
use color_eyre::owo_colors::colors::Green;
use color_eyre::owo_colors::{FgColorDisplay, OwoColorize};
use serde_json::Value;
use std::collections::HashMap;
use std::string::String;
use tracing::warn;

#[derive(Debug)]
pub struct ParameterBuilder {
    parts: Parts,
}

#[derive(Debug)]
struct Parts {
    name: String,
    description: Option<String>,
    default_value: ParameterValue,
    value_type: ParameterValueType,
    conditional_values: HashMap<String, ParameterValue>,
}

impl ParameterBuilder {
    pub fn new(name: String, parameter: &Parameter) -> Self {
        let parts = Parts {
            name,
            description: parameter.description.clone(),
            default_value: ParameterValue::Value(String::new()),
            value_type: parameter.value_type.clone(),
            conditional_values: HashMap::with_capacity(parameter.conditional_values.len()),
        };
        Self { parts }
    }

    pub async fn start_flow(
        name: Option<String>,
        description: Option<String>,
        conditions: &[Condition],
    ) -> (String, Parameter) {
        let name = match name.map(Parts::validate_name).transpose() {
            Err(message) => {
                warn!("{}", message.yellow());
                None
            }
            Ok(name) => name,
        };
        let builder: ParameterBuilder = match (name, description) {
            (Some(name), Some(description)) => {
                let mut parts = Parts::new(name);
                parts.description = Some(description);
                ParameterBuilder { parts }
            }
            (Some(name), None) => {
                let parts = Parts::new(name);
                let builder = ParameterBuilder { parts };
                builder.request_description().await
            }
            (None, Some(description)) => {
                let mut parts = Parts::new(Self::request_name().await);
                parts.description = Some(description);
                ParameterBuilder { parts }
            }
            (None, None) => {
                let parts = Parts::new(Self::request_name().await);
                let builder = ParameterBuilder { parts };
                builder.request_description().await
            }
        };
        builder
            .request_value_type()
            .await
            .request_default_value()
            .await
            .request_condition(conditions)
            .await
            .parts
            .parameter()
    }

    pub async fn add_values(
        self,
        selected_conditions: impl Iterator<Item = &str>,
    ) -> Result<(String, Parameter)> {
        let mut builder = self.request_default_value().await;
        for condition in selected_conditions {
            builder = builder.request_value_for_condition(condition).await;
        }
        Ok(builder.parts.parameter())
    }

    async fn request_name() -> String {
        loop {
            let result =
                InputReader::request_user_input::<InputString, FgColorDisplay<Green, &str>>(
                    &"Enter parameter name:".green(),
                )
                .await
                .map_err(|error| error.to_string())
                .and_then(|name| Parts::validate_name(name.0).map_err(|e| e.to_string()));

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

    async fn request_description(self) -> ParameterBuilder {
        self.and_then(
            "Enter description (Optional):",
            |parts, value: InputString| {
                let description = value.0;
                parts.description = if description.is_empty() {
                    None
                } else {
                    Some(description)
                };
                Ok(())
            },
        )
        .await
    }

    async fn request_value_type(self) -> ParameterBuilder {
        let message = "Enter value type. It can be one of the following: \
            Boolean [b], \
            Number [n], \
            JSON [j], \
            String [s]: ";
        self.and_then(message, |parts, value_type| {
            parts.value_type = value_type;
            Ok(())
        })
        .await
    }

    async fn request_default_value(self) -> ParameterBuilder {
        self.and_then("Enter default value:", |parts, value: InputString| {
            parts.set_default_value(value.0).map_err(Error::new)
        })
        .await
    }

    async fn request_condition(self, conditions: &[Condition]) -> ParameterBuilder {
        if conditions.is_empty() {
            return self;
        }
        let message = format!("{}", "Do you want to add conditional value? [Y,n]".green());
        match self.request_select_condition(&message, conditions).await {
            None => self,
            Some(index) => {
                let condition_name = &conditions[index].name;
                let mut builder = self.request_value_for_condition(condition_name).await;
                let message = format!(
                    "{}",
                    "Do you want to add additional conditional value? [Y,n]".green()
                );
                while let Some(selected_index) =
                    builder.request_select_condition(&message, conditions).await
                {
                    let condition_name = &conditions[selected_index].name;
                    builder = builder.request_value_for_condition(condition_name).await;
                }
                builder
            }
        }
    }

    async fn request_value_for_condition(self, condition_name: &str) -> ParameterBuilder {
        let message = format!("Enter value for {} condition:", &condition_name);
        let valid_value = loop {
            let result = InputReader::request_user_input::<
                InputString,
                FgColorDisplay<Green, String>,
            >(&message.green())
            .await
            .map_err(|e| e.to_string())
            .and_then(|value| {
                Parts::validate_value(value.0, &self.parts.value_type).map_err(|e| e.to_string())
            });

            match result {
                Ok(value) => {
                    break value;
                }
                Err(message) => warn!("{}", message.yellow()),
            }
        };
        let mut parts = self.parts;
        parts.conditional_values.insert(
            condition_name.to_string(),
            ParameterValue::Value(valid_value),
        );
        ParameterBuilder { parts }
    }

    async fn request_select_condition(
        &self,
        message: &str,
        conditions: &[Condition],
    ) -> Option<usize> {
        if !InputReader::ask_confirmation(message).await {
            return None;
        }
        let condition_names = conditions.iter().map(|cond| cond.name.as_str());
        let label = "Select one of available conditions:";
        InputReader::request_select_item_in_list(label, condition_names, None).await
    }

    async fn and_then<F, P>(self, request_msg: &'static str, parts_modifier: F) -> ParameterBuilder
    where
        F: Fn(&mut Parts, P) -> Result<()>,
        P: TryFrom<String, Error = Error>,
    {
        let mut parts = self.parts;
        loop {
            let result = InputReader::request_user_input::<P, FgColorDisplay<Green, &str>>(
                &request_msg.green(),
            )
            .await
            .and_then(|value| parts_modifier(&mut parts, value));
            match result {
                Ok(_) => {
                    return ParameterBuilder { parts };
                }
                Err(error) => warn!("{}", error.message.yellow()),
            }
        }
    }
}

impl Parts {
    fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            default_value: ParameterValue::Value(String::new()),
            value_type: ParameterValueType::String,
            conditional_values: HashMap::new(),
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

impl TryFrom<String> for ParameterValueType {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match &value.to_lowercase().as_ref() {
            &"b" | &"boolean" => Ok(Self::Boolean),
            &"j" | &"json" => Ok(Self::Json),
            &"n" | &"number" => Ok(Self::Number),
            &"s" | &"string" => Ok(Self::String),
            _ => Err(Error::new("Unexpected value. It can be one of the following: Boolean [b], Number [n], JSON [j], String [s]"))
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
