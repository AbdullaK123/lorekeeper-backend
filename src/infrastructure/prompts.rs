use std::fs;
use crate::infrastructure::InfrastructureError;

const PROMPTS_PATH: &str = "./prompts";

pub fn load_prompt(prompt_name: &str) -> Result<String, InfrastructureError> {
    let prompt_path = format!("{PROMPTS_PATH}/{prompt_name}.md");
    fs::read_to_string(&prompt_path)
        .map_err(|e| InfrastructureError::InternalError(e.to_string()))
}