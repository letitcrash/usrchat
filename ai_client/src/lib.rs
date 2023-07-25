use async_openai::{
    types::{
        ChatCompletionFunctionsArgs, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, Role,
    },
    Client,
};
use serde_json::json;
use std::error::Error;
use std::{collections::HashMap, future::Future, pin::Pin};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

enum StoredFunction {
    Option0(Box<dyn Fn() -> ()>),
    Option1(Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = serde_json::Value> + Send>> + Send>),
    Option2(
        Box<
            dyn Fn(String, String) -> Pin<Box<dyn Future<Output = serde_json::Value> + Send>>
                + Send,
        >,
    ),
}

pub async fn getw() -> Result<(), Box<dyn Error>> {
    // This should come from env var outside the program
    std::env::set_var("RUST_LOG", "warn");

    // Setup tracing subscriber so that library can log the rate limited message
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo-0613")
        .messages([ChatCompletionRequestMessageArgs::default()
            .role(Role::User)
            .content("What's the weather like in Boston?")
            .build()?])
        .functions([ChatCompletionFunctionsArgs::default()
            .name("get_current_weather")
            .description("Get the current weather in a given location")
            .parameters(json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA",
                    },
                    "unit": { "type": "string", "enum": ["celsius", "fahrenheit"] },
                },
                "required": ["location"],
            }))
            .build()?])
        .function_call("auto")
        .build()?;

    let response_message = client
        .chat()
        .create(request)
        .await?
        .choices
        .get(0)
        .unwrap()
        .message
        .clone();

    print!("Response: {:?}", response_message);

    if let Some(function_call) = response_message.function_call {
        let mut available_functions: HashMap<&str, StoredFunction> = HashMap::new();

        available_functions.insert(
            "get_current_weather",
            StoredFunction::Option2(Box::new(|location, unit| {
                Box::pin(get_current_weather(location, unit))
            })),
        );
        let function_name = function_call.name;
        let function_args: serde_json::Value = function_call.arguments.parse().unwrap();

        let location = function_args["location"].to_string();
        let unit = "fahrenheit".to_string();
        let function = available_functions.get(function_name.as_str()).unwrap();
        let function_response = match function {
            StoredFunction::Option2(f) => {
                let result = f(location, unit).await;
                println!("result = {:?}", result);
                result
            }
            _ => json!({"error": "function not found"}),
        };

        let message = vec![
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content("What's the weather like in Boston?")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::Function)
                .name(function_name)
                .content(function_response.to_string())
                .build()?,
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-3.5-turbo-0613")
            .messages(message)
            .build()?;

        let response = client.chat().create(request).await?;

        println!("\nResponse:\n");
        for choice in response.choices {
            println!(
                "{}: Role: {}  Content: {:?}",
                choice.index, choice.message.role, choice.message.content
            );
        }
    }

    Ok(())
}

async fn get_current_weather(location: String, unit: String) -> serde_json::Value {
    // let body = reqwest::get("https://www.rust-lang.org")
    //     .await
    //     .unwrap()
    //     .text()
    //     .await
    //     .unwrap();

    // println!("body = {:?}", body);

    let weather_info = json!({
        "location": location,
        "temperature": "72",
        "unit": unit,
        "forecast": ["sunny", "windy"]
    });

    weather_info
}
