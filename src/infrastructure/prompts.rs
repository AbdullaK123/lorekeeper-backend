use crate::infrastructure::InfrastructureError;
use std::fs;

const PROMPTS_PATH: &str = "./prompts";

pub fn load_prompt(prompt_name: &str) -> Result<String, InfrastructureError> {
    let prompt_path = format!("{PROMPTS_PATH}/{prompt_name}.md");
    let prompt = fs::read_to_string(&prompt_path)
        .inspect_err(|_| tracing::error!("loading configured AI prompt failed"))
        .map_err(|e| InfrastructureError::InternalError(e.to_string()))?;
    tracing::debug!(bytes = prompt.len(), "configured AI prompt loaded");
    Ok(prompt)
}
