use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let mut agent = ai_client::client::Agent::new().await;

    loop {
        let mut input = String::new();

        print!("User> ");
        io::stdout().flush().unwrap(); // Flush stdout to print the prompt before the input

        match io::stdin().read_line(&mut input) {
            Ok(_n) => {
                let input = input.trim(); // Remove trailing newline
                if input == "exit" {
                    break;
                } else {
                    let response = agent.msg(input).await;
                    match response {
                        Ok(response) => {
                            println!("Bot> {}", response);
                            continue;
                        }
                        Err(error) => {
                            println!("Error: {}", error);
                            continue;
                        }
                    };

                }
            }
            Err(error) => {
                println!("Failed to read line: {}", error);
            }
        }
    }
}
