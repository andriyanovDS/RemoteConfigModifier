use crate::error::Result;
use crate::network::NetworkService;
use crate::remote_config::{
    Condition, Parameter, ParameterValue, ParameterValueType, RemoteConfig,
};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

pub struct ShowConfigFlow {
    network_service: NetworkService,
}

impl ShowConfigFlow {
    pub fn new() -> Self {
        Self {
            network_service: NetworkService::new(),
        }
    }

    pub async fn start_flow(mut self) -> Result<()> {
        let mut response = self.network_service.get_remote_config().await?;
        let table = ShowConfigFlow::build_table(&mut response.data);
        println!("{}", table.render());
        Ok(())
    }

    fn build_table(config: &mut RemoteConfig) -> Table {
        let mut table = Table::new();
        table.max_column_width = 40;
        table.style = TableStyle::simple();

        table.add_row(ShowConfigFlow::make_title_row("Parameters"));
        config
            .parameters
            .iter()
            .map(|(name, parameter)| parameter.make_row(name, None))
            .flatten()
            .for_each(|row| table.add_row(row));

        if let Some(groups) = config.parameter_groups.as_ref() {
            groups
                .iter()
                .flat_map(|(group_name, group)| {
                    group.parameters.as_ref().map(|parameters| {
                        parameters
                            .iter()
                            .map(|(name, parameter)| parameter.make_row(name, Some(group_name)))
                    })
                })
                .flatten()
                .flatten()
                .for_each(|row| table.add_row(row))
        }

        if !config.conditions.is_empty() {
            table.add_row(ShowConfigFlow::make_title_row("Conditions"));
            config
                .conditions
                .iter_mut()
                .map(|condition| condition.make_row())
                .for_each(|row| table.add_row(row))
        }

        table
    }

    fn make_title_row(title: &str) -> Row {
        Row::new(vec![TableCell::new_with_alignment(
            title,
            5,
            Alignment::Center,
        )])
    }
}

impl Parameter {
    fn make_row(&self, name: &str, group_name: Option<&str>) -> Vec<Row> {
        let rows_count = self
            .conditional_values
            .as_ref()
            .map(|cond| cond.len() + 1)
            .unwrap_or(1);
        let mut rows = Vec::with_capacity(rows_count);
        let default_row = Row::new(vec![
            TableCell::new(name),
            TableCell::new(""),
            TableCell::new(self.value_type.cell_content()),
            TableCell::new(
                self.default_value
                    .as_ref()
                    .map(|value| value.cell_content())
                    .unwrap_or(""),
            ),
            TableCell::new(group_name.unwrap_or("")),
        ]);
        rows.push(default_row);
        if let Some(conditional_values) = self.conditional_values.as_ref() {
            conditional_values
                .iter()
                .map(|(name, value)| {
                    Row::new(vec![
                        TableCell::new(""),
                        TableCell::new(name),
                        TableCell::new(""),
                        TableCell::new(value.cell_content()),
                    ])
                })
                .for_each(|row| rows.push(row))
        }
        rows
    }
}

impl Condition {
    fn make_row(&mut self) -> Row {
        self.expression = self.expression.replace("&& ", "\n && ");
        Row::new(vec![
            TableCell::new(&self.name),
            TableCell::new_with_col_span(&self.expression, 4),
        ])
    }
}

impl ParameterValueType {
    fn cell_content(&self) -> &str {
        match self {
            Self::String => "String",
            Self::Boolean => "Bool",
            Self::Number => "Number",
            Self::Json => "JSON",
            Self::Unspecified => "Unspecified",
        }
    }
}

impl ParameterValue {
    fn cell_content(&self) -> &str {
        match self {
            Self::Value(string) => &string,
            Self::UseInAppDefault(_) => "Use in app default",
        }
    }
}
