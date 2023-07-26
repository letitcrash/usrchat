use async_openai::types::{ChatCompletionFunctions, ChatCompletionFunctionsArgs};
use serde_json::json;

pub fn list_functions() -> [ChatCompletionFunctions; 2] {
    [
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
            .build().unwrap(),
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
            .build().unwrap(),
    ]
}
