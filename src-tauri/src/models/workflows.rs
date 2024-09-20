use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractDocumentImagesStage {
    pub document_path: String,
    pub data_directory: String,
    pub images_directory: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractDocumentImagesStageSuccess {
    pub document_path: String,
    pub data_directory: String,
    pub images_directory: String,
    pub document_clone_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractDocumentImagesStageError {
    pub document_path: String,
    pub data_directory: String,
    pub images_directory: String,
    pub error_message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagePreprocessStage {
    pub id: String,
    pub selected_pages: Vec<u32>,
    pub data_directory: String,
    pub images_directory: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PagePreprocessStageResult {
    pub dates: Vec<Date>,
    pub type_name: String,
    pub type_abbr: String,
    pub summary: String,
    pub suggested_file_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Date {
    pub date: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagePreprocessStageSuccess {
    pub id: String,
    pub selected_pages: Vec<u32>,
    pub data_directory: String,
    pub images_directory: String,
    pub page_preprocess_stage_result: PagePreprocessStageResult,
    pub page_number_prefix: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagePreprocessStageError {
    pub id: String,
    pub selected_pages: Vec<u32>,
    pub data_directory: String,
    pub images_directory: String,
    pub error_message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentProcessStage {
    pub id: String,
    pub selected_pages: Vec<u32>,
    pub data_directory: String,
    pub images_directory: String,
    pub document_path: String,
    pub file_name: String,
    pub page_preprocess_stage_result: PagePreprocessStageResult,
    pub page_number_prefix: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentProcessStageSuccess {
    pub id: String,
    pub selected_pages: Vec<u32>,
    pub data_directory: String,
    pub images_directory: String,
    pub document_path: String,
    pub file_name: String,
    pub page_preprocess_stage_result: PagePreprocessStageResult,
    pub page_number_prefix: String,
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentProcessStageError {
    pub id: String,
    pub selected_pages: Vec<u32>,
    pub data_directory: String,
    pub images_directory: String,
    pub document_path: String,
    pub file_name: String,
    pub page_preprocess_stage_result: PagePreprocessStageResult,
    pub page_number_prefix: String,
    pub error_message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishedDocumentProcessStage {
    pub id: String,
    pub selected_pages: Vec<u32>,
    pub data_directory: String,
    pub images_directory: String,
    pub page_preprocess_stage_result: PagePreprocessStageResult,
    pub document_path: String,
    pub file_name: String,
    pub file_name_history: Vec<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct ProgressState {
    pub pages_processed: usize,
    pub pages_to_process: usize,
    pub total_document_pages: usize,
    pub estimated_seconds_remaining: u64,
    pub extracted_page_numbers: Vec<usize>,
}
