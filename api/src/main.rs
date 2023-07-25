#[tokio::main]
async fn main() {
    let weather = ai_client::getw().await;
    match weather {
        Ok(weather_data) => println!("Weather: {:?}", weather_data),
        Err(e) => println!("An error occurred: {:?}", e),
    }
}
