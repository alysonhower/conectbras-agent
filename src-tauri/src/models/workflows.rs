use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// export interface ExtactDocumentImagesStage {
//   documentPath: string;
//   imagesDirectory: string;
//   dataDirectory: string;
//   startTime: number;
// }

// export interface ExtactDocumentImagesStageSuccess
//   extends ExtactDocumentImagesStage {
//   endTime: number;
//   elapsedTime: number;
//   documentClonePath: string;
// }

// export interface ExtactDocumentImagesStageError
//   extends ExtactDocumentImagesStage {
//   endTime: number;
//   elapsedTime: number;
//   errorMessage: string;
// }

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractDocumentImagesStage {
    pub document_path: PathBuf,
    pub document_clone_path: PathBuf,
    pub images_directory: PathBuf,
    pub start_time: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractDocumentImagesStageSuccess {
    pub document_path: PathBuf,
    pub images_directory: PathBuf,
    pub start_time: u64,
    pub end_time: u64,
    pub elapsed_time: u64,
    pub document_clone_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractDocumentImagesStageError {
    pub document_path: PathBuf,
    pub images_directory: PathBuf,
    pub start_time: u64,
    pub end_time: u64,
    pub elapsed_time: u64,
    pub error_message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagePreprocessStage {
    pub id: String,
    pub data_directory: String,
    pub selected_pages: Vec<u32>,
    pub images_directory: PathBuf,
    pub start_time: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Date {
    pub date: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PagePreprocessStageResult {
    pub dates: Vec<Date>,
    pub type_name: String,
    pub type_abbr: String,
    pub summary: String,
    pub suggested_file_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagePreprocessStageSuccess {
    pub id: String,
    pub selected_pages: Vec<u32>,
    pub data_directory: String,
    pub images_directory: String,
    pub start_time: u64,
    pub end_time: u64,
    pub elapsed_time: u64,
    pub preprocess_pages_stage_result: PagePreprocessStageResult,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentProcessStage {
    pub id: String,
    pub selected_pages: Vec<u32>,
    pub data_directory: String,
    pub images_directory: String,
    pub start_time: u64,
    pub end_time: u64,
    pub elapsed_time: u64,
    pub preprocess_pages_stage_result: PagePreprocessStageResult,
    pub document_path: String,
    pub file_name: String,
    pub file_name_history: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagePreprocessStageError {
    pub id: String,
    pub data_directory: String,
    pub selected_pages: Vec<u32>,
    pub images_directory: PathBuf,
    pub start_time: u64,
    pub end_time: u64,
    pub elapsed_time: u64,
    pub error_message: String,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct ProgressState {
    pub pages_processed: usize,
    pub pages_to_process: usize,
    pub total_document_pages: usize,
    pub estimated_seconds_remaining: u64,
    pub extracted_page_numbers: Vec<usize>,
}
