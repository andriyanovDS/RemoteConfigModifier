use remote_config_modifier::add_parameter_flow::AddParameterFlow;

#[tokio::main]
async fn main() {
    let mut add_parameter_flow = AddParameterFlow::new();
    add_parameter_flow.start_flow().await;
}
