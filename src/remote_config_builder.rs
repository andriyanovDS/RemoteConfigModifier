use tokio::io::AsyncBufReadExt;
use colored::Colorize;
use serde_json::Value;
use std::string::String;
use crate::remote_config::{Parameter, ParameterValue, ParameterValueType};

#[derive(Debug)]
pub struct RemoteConfigBuilder {
    inner: Result<Parts>,
}

#[derive(Debug)]
pub struct Error {
    pub message: &'static str,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Parts {
    name: String,
    description: Option<String>,
    default_value: ParameterValue,
    value_type: ParameterValueType,
}

struct ParameterString(String);

impl RemoteConfigBuilder {
    pub async fn start_flow() -> std::result::Result<(String, Parameter), &'static str> {
        Self::request_name()
            .await
            .request_value_type()
            .await
            .request_default_value()
            .await
            .request_description()
            .await
            .inner
            .map(|parts| parts.parameter())
            .map_err(|error| error.message)
    }

    async fn request_name() -> Self {
        let result: Result<Parts> = Self::request_user_input::<ParameterString>(&"Enter parameter name:")
            .await
            .and_then(|name| {
                Parts::validate_name(name.0).map_err(Error::new)
            })
            .map(Parts::new);

        RemoteConfigBuilder {
            inner: result
        }
    }

    async fn request_value_type(self) -> Self {
        let message = "Enter value type. It can be one of the following: \
            Boolean [b], \
            Number [n], \
            JSON [j], \
            String [s]: ";
        self.and_then(message, |mut parts, value_type| {
            parts.value_type = value_type;
            Ok(parts)
        }).await
    }

    async fn request_default_value(self) -> Self {
        self.and_then(&"Enter default value:", |mut parts, value: ParameterString| {
            parts.set_default_value(value.0).map_err(Error::new)
        }).await
    }

    async fn request_description(self) -> Self {
        self.and_then(&"Enter description (Optional):", |mut parts, value: ParameterString| {
            let description = value.0;
            parts.description = if description.is_empty() {
                None
            } else {
                Some(description)
            };
            Ok(parts)
        }).await
    }

    async fn and_then<F, P>(
        self,
        request_msg: &'static str,
        parts_modifier: F
    ) -> Self where F: FnOnce(Parts, P) -> Result<Parts>, P: TryFrom<String, Error=Error> {
        let inner = match self.inner {
            Ok(parts) => {
                Self::request_user_input::<P>(request_msg).await.and_then(move |value| {
                    parts_modifier(parts, value)
                })
            }
            Err(error) => Err(error)
        };
        Self { inner }
    }

    async fn request_user_input<R>(request_msg: &'static str) -> Result<R> where R: TryFrom<String, Error=Error> {
        println!("{}", format!("{}", request_msg.green()));
        let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
        let mut buffer = String::new();
        reader.read_line(&mut buffer).await
            .map_err(|_| Error::new("Failed to read input"))
            .map(move |_| {
                buffer.pop();
                buffer
            })
            .and_then(R::try_from)
    }
}

impl Parts {
    fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            default_value: ParameterValue::Value(String::new()),
            value_type: ParameterValueType::String,
        }
    }

    fn validate_name(name: String) -> std::result::Result<String, &'static str> {
        if name.is_empty() {
            return Err("Name must contain at least one character");
        }
        let mut characters = name.chars().into_iter();
        let first_char = characters.next().unwrap();
        if first_char.is_ascii_alphabetic() == false && first_char != '_' {
            return Err("Parameter keys must start with an underscore or English letter character [A-Z, a-z]");
        }
        if characters.all(|char| char.is_ascii_alphanumeric() || char == '_') {
            Ok(name)
        } else {
            Err("Parameter keys can only include English letter characters, numbers and underscore")
        }
    }

    fn set_default_value(mut self, value: String) -> std::result::Result<Self, &'static str> {
        let mut parts = match &self.value_type {
            ParameterValueType::Boolean => value.parse::<bool>().map(move|_| self).map_err(|_| "Value must boolean"),
            ParameterValueType::Number => {
                if value.chars().into_iter().all(|char| char.is_numeric()) {
                    Ok(self)
                } else {
                    Err("Value must numeric")
                }
            }
            ParameterValueType::String => Ok(self),
            ParameterValueType::Json => {
                serde_json::from_str::<Value>(&value)
                    .map_err(|_| "Invalid JSON")
                    .map(move|_| self)
            }
            ParameterValueType::Unspecified => panic!("Unsupported value type")
        }?;
        parts.default_value = ParameterValue::Value(value);
        Ok(parts)
    }

    fn parameter(self) -> (String, Parameter) {
        let parameter = Parameter {
            default_value: Some(self.default_value),
            conditional_values: None,
            description: self.description,
            value_type: self.value_type,
        };
        (self.name, parameter)
    }
}

impl TryFrom<String> for ParameterString {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        String::try_from(value)
            .map_err(|_| Error::new("Failed to parse input string"))
            .map(|name| Self(name))
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

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Ok(Self::Value(value))
    }
}

impl Error {
    fn new(message: &'static str) -> Self {
        Self { message }
    }
}