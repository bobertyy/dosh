use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ErrorJson {
    pub error: String,
}

impl ErrorJson {
    pub fn new<Message: Into<String>>(message: Message) -> Self {
        Self {
            error: message.into(),
        }
    }
}
