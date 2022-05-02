use remote_config_modifier::network::NetworkService;

#[tokio::main]
async fn main() {
    let mut network_service = NetworkService::new();
    let result = network_service.get_remote_config().await;
    if let Err(error) = result {
        println!("error {:?}", error);
    }
}
