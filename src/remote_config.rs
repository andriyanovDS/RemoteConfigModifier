use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfig {
    pub conditions: Vec<Condition>,
    pub parameters: HashMap<String, Parameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_groups: Option<HashMap<String, ParameterGroup>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub name: String,
    expression: String,
    tag_color: TagColor,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TagColor {
    #[serde(rename = "CONDITION_DISPLAY_COLOR_UNSPECIFIED")]
    Unspecified,
    Blue,
    Brown,
    Cyan,
    DeepOrange,
    Green,
    Indigo,
    Lime,
    Orange,
    Pink,
    Purple,
    Teal,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ParameterGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: HashMap<String, Parameter>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<ParameterValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_values: Option<HashMap<String, ParameterValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub value_type: ParameterValueType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParameterValueType {
    #[serde(rename = "PARAMETER_VALUE_TYPE_UNSPECIFIED")]
    Unspecified,
    Boolean,
    String,
    Number,
    Json,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ParameterValue {
    Value(String),
    UseInAppDefault(bool),
}

impl RemoteConfig {
    pub fn get_map_for_existing_parameter(
        &mut self,
        name: &str,
    ) -> Option<&mut HashMap<String, Parameter>> {
        if self.parameters.contains_key(name) {
            Some(&mut self.parameters)
        } else if let Some(groups) = &mut self.parameter_groups {
            groups.iter_mut().find_map(|(_, group)| {
                if group.parameters.contains_key(name) {
                    Some(&mut group.parameters)
                } else {
                    None
                }
            })
        } else {
            None
        }
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let entries = [
            ("name", Debug::fmt(&self.name, f)?),
            ("expression", Display::fmt(&self.expression, f)?),
            ("tag_color", Debug::fmt(&self.tag_color, f)?),
        ];
        f.debug_map().entries(entries).finish()
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let new_line = if f.alternate() { '\n' } else { ' ' };
        write!(f, "{{ {new_line}")?;
        write!(f, "  value_type: {:?},{new_line}", self.value_type)?;
        if let Some(value) = &self.default_value {
            write!(f, "  default_value: {value:?},{new_line}")?;
        }
        if let Some(value) = &self.description {
            write!(f, "  description: {value},{new_line}")?;
        }
        if let Some(value) = &self.conditional_values {
            write!(f, "  conditional_values: {value:#?},{new_line}")?;
        }
        write!(f, "}}")
    }
}

impl Display for ParameterValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        let map = match self {
            ParameterValue::Value(value) => map.entries([("value", value)]),
            ParameterValue::UseInAppDefault(use_default) => {
                map.entries([("useInAppDefault", use_default)])
            }
        };
        map.finish()
    }
}

impl Debug for ParameterValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterValue::Value(value) => write!(f, "{value}"),
            ParameterValue::UseInAppDefault(use_default) => write!(f, "{use_default}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Error;

    #[test]
    fn serialization() {
        let parameter = Parameter {
            default_value: Some(ParameterValue::Value("false".to_string())),
            conditional_values: None,
            description: Some("some desc".to_string()),
            value_type: ParameterValueType::Boolean,
        };
        let condition = Condition {
            name: "Platform".to_string(),
            expression: "device.os == 'ios'".to_string(),
            tag_color: TagColor::Brown,
        };
        let mut parameters = HashMap::new();
        parameters.insert("uploadLogs".to_string(), parameter);
        let remote_config = RemoteConfig {
            conditions: vec![condition],
            parameters,
            parameter_groups: None,
        };
        let result = serde_json::to_string(&remote_config).unwrap();
        assert_eq!(result, "{\
            \"conditions\":[{\"name\":\"Platform\",\"expression\":\"device.os == 'ios'\",\"tagColor\":\"BROWN\"}],\
            \"parameters\":{\"uploadLogs\":{\"defaultValue\":{\"value\":\"false\"},\"description\":\"some desc\",\"valueType\":\"BOOL\"}}\
        }")
    }

    #[test]
    fn deserialization() {
        let json = "{\
          \"conditions\": [{\
            \"name\": \"Platform\",\
            \"expression\": \"device.os == 'ios'\",\
            \"tagColor\": \"BLUE\"\
          }],\
          \"parameters\": {\
            \"maxCameraResolutions\": {\
              \"defaultValue\": {\
                \"value\": \"{\"iPhone13,2\":\"720x480\"}\"\
              },\
              \"conditionalValues\": {\
                \"Platform\": {\
                  \"value\": \"{\"iPhone13,2\":\"1280x720\"}\"\
                }\
              },\
              \"description\": \"Maximum camera resolutions map for iOS devices\",\
              \"valueType\": \"JSON\"\
            }\
          }\
        }";
        println!("{:?}", &json);
        let bytes = json.as_bytes();

        let result: Result<RemoteConfig, Error> = serde_json::from_slice(bytes);
        if let Err(error) = &result {
            println!("error: {error}");
        }

        let received_remote_config: RemoteConfig = result.unwrap();

        let parameters = {
            let mut map = HashMap::new();
            let conditional_values = {
                let mut map = HashMap::new();
                map.insert(
                    "Platform".to_string(),
                    ParameterValue::Value("{\"iPhone13,2\":\"1280x720\"}".to_string()),
                );
                map
            };
            let parameter = Parameter {
                default_value: Some(ParameterValue::Value(
                    "{\"iPhone13,2\":\"720x480\"}".to_string(),
                )),
                conditional_values: Some(conditional_values),
                description: Some("Maximum camera resolutions map for iOS devices".to_string()),
                value_type: ParameterValueType::Json,
            };
            map.insert("maxCameraResolutions".to_string(), parameter);
            map
        };

        let expected_config = RemoteConfig {
            conditions: vec![],
            parameters,
            parameter_groups: None,
        };
        assert_eq!(received_remote_config, expected_config)
    }
}
