use async_openai::{
    types::{
        ChatCompletionFunctionsArgs, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, FunctionCall, Role,
    },
    Client,
};
use serde_json::json;
use std::error::Error;
use std::{collections::HashMap, future::Future, pin::Pin};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

static SYSTEM_MSG: &str = r#"
You are a personal assistant. 
If you think that user message is a command to save data, 
you can use persist_data function to save it in database.
Use the following types:
- task
- note
- reminder
- event
- contact
- location
- link
- file
- image
- video
- audio
If there is something you can comment about user message and suggest ideas, 
enhance it with valuable information and add it data to be persisted in database. 
If it's not clear what to do next you ask user about further guidance.
"#;

enum StoredFunction {
    // Option0(Box<dyn Fn() -> ()>),
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
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(SYSTEM_MSG)
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                // .content("buy some milk")
                .content("note some ideas how to get back into a healthy relationship")
                .build()?,
        ])
        .functions([
            ChatCompletionFunctionsArgs::default()
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
                .build()?,
            ChatCompletionFunctionsArgs::default()
                .name("persist_data")
                .description("Persis data into a database")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "data": {
                            "type": "string",
                            "description": "The data to save",
                        },
                        "type": {
                            "type": "string",
                            "enum": [
                                "task",
                                "note",
                                "reminder",
                                "event",
                                "contact",
                                "location",
                                "link",
                                "file",
                                "image",
                                "video",
                                "audio"
                            ],
                            "description": "The type of data to save",
                        }
                    },
                    "required": ["task"],
                }))
                .build()?,
        ])
        .function_call("auto")
        .build()?;

    let mut response = client
        .chat()
        .create(request)
        .await?;

    let response_message = response.choices[0].message.clone();

    // print!("Response: {:?}", response_message);

    if let Some(function_call) = response_message.function_call {
        let mut available_functions: HashMap<&str, StoredFunction> = HashMap::new();

        available_functions.insert(
            "get_current_weather",
            StoredFunction::Option2(Box::new(|location, unit| {
                Box::pin(get_current_weather(location, unit))
            })),
        );

        available_functions.insert(
            "persist_data",
            StoredFunction::Option2(Box::new(|data, data_type| Box::pin(persist_data(data, data_type)))),
        );

        let (function_response, function_name) = match function_call {
            FunctionCall {
                name: function_name,
                arguments: function_args,
            } => {
                println!("function_name = {:?}", function_name);
                let function = available_functions.get(function_name.as_str()).unwrap();
                let function_args: serde_json::Value = function_args.parse().unwrap();
                let function_response = match (function, function_name.as_str()) {
                    (StoredFunction::Option2(f), "get_current_weather") => {
                        let location = function_args["location"].to_string();
                        let unit = "fahrenheit".to_string();
                        let result = f(location, unit).await;
                        (result, function_name)
                    }
                    (StoredFunction::Option2(f), "persist_data") => {
                        let data = function_args["data"].to_string();
                        let data_type = function_args["type"].to_string();
                        let result = f(data, data_type).await;
                        (result, function_name)
                    }
                    _ => panic!("function not found"),
                };
                function_response
            }
        };

        let message = vec![
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(SYSTEM_MSG)
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                // .content("buy some milk")
                .content("note some ideas how to get back into a healthy relationship")
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

        // override response
        response = client.chat().create(request).await?;
    }


    println!("\nResponse:\n");
    for choice in response.choices {
        println!(
            "{}: Role: {}  Content: {:?}",
            choice.index, choice.message.role, choice.message.content
        );
    }

    Ok(())
}

async fn persist_data(data: String, data_type: String) -> serde_json::Value {
    println!("saving data = {:?}", data);
    let saved_data = json!({
        "data": data,
        "type": data_type,
        "id": "1234",
    });

    saved_data
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
