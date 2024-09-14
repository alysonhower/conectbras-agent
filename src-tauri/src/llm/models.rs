use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<Content>,
    pub id: String,
    pub model: String,
    pub role: String,
    pub stop_reason: String,
    pub stop_sequence: Option<String>,
    #[serde(rename = "type")]
    pub response_type: String,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub text: String,
    #[serde(rename = "type")]
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnthropicError {
    #[serde(rename = "type")]
    pub response_type: String,
    pub error: OutputError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub file_name: String,
    #[serde(default)]
    pub file_name_history: Vec<String>,
    pub pages_paths: Vec<String>,
    pub reasoning: Reasoning,
    pub json_file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reasoning {
    pub document_summary: DocumentSummary,
    pub document_type: DocumentType,
    pub important_date: ImportantDate,
    pub language: String,
    pub main_entities: MainEntities,
    pub type_abbreviation: TypeAbbreviation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub analysis: String,
    pub formatting_process: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentType {
    pub analysis: String,
    pub type_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportantDate {
    pub analysis: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MainEntities {
    pub analysis: String,
    pub entities: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeAbbreviation {
    pub analysis: String,
    pub type_abbr: String,
}