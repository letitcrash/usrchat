#[tokio::main]
async fn main() {
    let _response = ai_client::getw().await;
    // match weather {
    //     Ok(weather_data) => println!("Weather: {:?}", weather_data),
    //     Err(e) => println!("An error occurred: {:?}", e),
    // }
}
