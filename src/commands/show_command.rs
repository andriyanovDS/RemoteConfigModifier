use crate::error::Result;
use crate::network::NetworkService;
use crate::remote_config::RemoteConfig;
use crate::projects::Project;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

pub struct ShowCommand {
    network_service: NetworkService,
}

impl ShowCommand {
    pub fn new() -> Self {
        Self {
            network_service: NetworkService::new(),
        }
    }

    pub async fn start_flow(mut self) -> Result<()> {
        let project = Project::stub();
        let mut response = self.network_service.get_remote_config(&project).await?;
        let table = ShowCommand::build_table(&mut response.data);
        println!("{}", table.render());
        Ok(())
    }

    fn build_table(config: &mut RemoteConfig) -> Table {
        let mut table = Table::new();
        table.max_column_width = 40;
        table.style = TableStyle::simple();

        table.add_row(ShowCommand::make_title_row("Parameters"));
        config
            .parameters
            .iter()
            .map(|(name, parameter)| parameter.make_row(name, None))
            .flatten()
            .for_each(|row| table.add_row(row));

        config
            .parameter_groups
            .iter()
            .map(|(group_name, group)| {
                group
                    .parameters
                    .iter()
                    .map(|(name, parameter)| parameter.make_row(name, Some(group_name)))
            })
            .flatten()
            .flatten()
            .for_each(|row| table.add_row(row));

        if !config.conditions.is_empty() {
            table.add_row(ShowCommand::make_title_row("Conditions"));
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
