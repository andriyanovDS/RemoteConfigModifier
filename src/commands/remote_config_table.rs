use crate::remote_config::{
    Condition, Parameter, ParameterValue, ParameterValueType, RemoteConfig,
};
use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use term_table::{Table, TableStyle};

impl RemoteConfig {
    pub fn build_table<'a, 'b>(&'a self, project_name: &'b str) -> Table<'a> {
        let mut table = Table::new();
        table.max_column_width = 25;
        table.style = TableStyle::simple();

        let title = format!("{} parameters", project_name);
        table.add_row(Self::make_title_row(title));
        self.parameters
            .iter()
            .flat_map(|(name, parameter)| parameter.make_row(name, None))
            .for_each(|row| table.add_row(row));

        self.parameter_groups
            .iter()
            .flat_map(|(group_name, group)| {
                group
                    .parameters
                    .iter()
                    .flat_map(|(name, parameter)| parameter.make_row(name, Some(group_name)))
            })
            .for_each(|row| table.add_row(row));

        if !self.conditions.is_empty() {
            table.add_row(Self::make_title_row("Conditions".to_string()));
            self.conditions
                .iter()
                .map(|condition| condition.make_row())
                .for_each(|row| table.add_row(row))
        }
        table
    }

    fn make_title_row(title: String) -> Row<'static> {
        Row::new(vec![TableCell::new_with_alignment(
            title,
            5,
            Alignment::Center,
        )])
    }
}

impl Parameter {
    pub fn make_row<N: ToString>(&self, name: N, group_name: Option<&str>) -> Vec<Row> {
        let rows_count = self.conditional_values.len() + 1;
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
        self.conditional_values
            .iter()
            .map(|(name, value)| {
                Row::new(vec![
                    TableCell::new(""),
                    TableCell::new(name),
                    TableCell::new(""),
                    TableCell::new(value.cell_content()),
                ])
            })
            .for_each(|row| rows.push(row));

        rows
    }

    pub fn preview(&self, name: &str, title: &str, group_name: Option<&str>) {
        let mut table = Table::new();
        table.style = TableStyle::simple();
        table.max_column_width = 25;

        let title_label = Row::new(vec![TableCell::new_with_alignment(
            title,
            5,
            Alignment::Center,
        )]);
        table.add_row(title_label);

        self.make_row(name, group_name)
            .into_iter()
            .for_each(|row| table.add_row(row));

        println!("{}", table.render());
    }
}

impl Condition {
    pub fn make_row(&self) -> Row {
        let expression = self.expression.replace("&& ", "\n && ");
        Row::new(vec![
            TableCell::new(&self.name),
            TableCell::new_with_col_span(expression, 4),
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
            Self::Value(string) => string,
            Self::UseInAppDefault(_) => "Use in app default",
        }
    }
}
