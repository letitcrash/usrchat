use std::{collections::HashMap, error::Error};

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionFunctions, ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, FunctionCall, Role,
    },
    Client,
};

use crate::{functions, StoredFunction};

pub struct Agent {
    client: Client<OpenAIConfig>,
    messages: Vec<ChatCompletionRequestMessage>,
    functions: [ChatCompletionFunctions; 2],
}

impl Agent {
    pub async fn new() -> Self {
        let client = Client::new();

        Self {
            client,
            messages: vec![ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(crate::SYSTEM_MSG)
                .build()
                .unwrap()],
            functions: functions::list_functions(),
        }
    }

    pub async fn msg(&mut self, input: &str) -> Result<String, Box<dyn Error>> {
        let new_msg = ChatCompletionRequestMessageArgs::default()
            .role(Role::User)
            .content(input)
            .build()?;

        // let messages = &self.messages.push(new_msg);

        let mut messages = self.messages.clone();
        messages.push(new_msg);

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-3.5-turbo-0613")
            .messages(messages.clone())
            .functions(self.functions.clone())
            .function_call("auto")
            .build()?;

        let response = self.client.chat().create(request).await?;

        let response_message = response.choices[0].message.clone();

        if let Some(function_call) = response_message.function_call {
            let mut available_functions: HashMap<&str, StoredFunction> = HashMap::new();

            available_functions.insert(
                "persist_data",
                StoredFunction::Option2(Box::new(|data, data_type| {
                    Box::pin(crate::persist_data(data, data_type))
                })),
            );

            let (function_response, function_name) = match function_call {
                FunctionCall {
                    name: function_name,
                    arguments: function_args,
                } => {
                    let function = available_functions.get(function_name.as_str()).unwrap();
                    let function_args: serde_json::Value = function_args.parse().unwrap();
                    let function_response = match (function, function_name.as_str()) {
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

            let function_req_args = ChatCompletionRequestMessageArgs::default()
                .role(Role::Function)
                .name(function_name)
                .content(function_response.to_string())
                .build();

            let mut messages = self.messages.clone();
            messages.push(function_req_args.unwrap());

            let request = CreateChatCompletionRequestArgs::default()
                .max_tokens(512u16)
                .model("gpt-3.5-turbo-0613")
                .messages(messages.clone())
                .build()?;

            self.messages = messages;

            let response = self.client.chat().create(request).await?;
            let response_message = response.choices[0].message.clone();

            Ok(response_message.content.unwrap_or_default())
        } else {
            let assistant_message = ChatCompletionRequestMessageArgs::default()
                .role(Role::Assistant)
                .content(response_message.content.clone().unwrap_or_default())
                .build()?;

            messages.push(assistant_message);

            self.messages = messages;

            Ok(response_message.content.unwrap_or_default())
        }
    }
}
