use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RemoteConfig {
    conditions: Vec<Condition>,
    parameters: HashMap<String, Parameter>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Condition {
    name: String,
    expression: String,
    tag_color: TagColor,
}

impl Display for Condition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n  name: {},\n  expression: {},\n  tag_color: {:?}\n }}",
            self.name, self.expression, self.tag_color
        )
    }
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
#[serde(rename_all = "camelCase")]
struct Parameter {
    #[serde(skip_serializing_if = "Option::is_none")]
    default_value: Option<ParameterValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conditional_values: Option<HashMap<String, ParameterValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    value_type: ParameterValueType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
enum ParameterValueType {
    #[serde(rename = "PARAMETER_VALUE_TYPE_UNSPECIFIED")]
    Unspecified,
    Boolean,
    String,
    Number,
    Json,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
enum ParameterValue {
    Value(String),
    UseInAppDefault(bool),
}

impl Display for ParameterValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterValue::Value(value) => {
                write!(f, "{{\n  value: {}\n}}", value)
            }
            ParameterValue::UseInAppDefault(use_default) => {
                write!(f, "{{\n  useInAppDefault: {}\n}}", use_default)
            }
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
        };
        assert_eq!(received_remote_config, expected_config)
    }
}
