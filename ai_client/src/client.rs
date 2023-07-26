use std::error::Error;

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionFunctions, ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, Role,
    },
    Client,
};

use crate::functions;

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

        
        self.messages = messages;

        let response = self.client.chat().create(request).await?;

        let response_message = response.choices[0].message.clone();

        Ok(response_message.content.unwrap_or_default())
    }
}
