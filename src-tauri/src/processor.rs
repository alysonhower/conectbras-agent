use regex::Regex;
use std::fs;
use std::path::Path;
use tauri::AppHandle;

use super::models::workflows::{
    DocumentProcessStage, DocumentProcessStageSuccess, DocumentProcessStageError, PagePreprocessStage, PagePreprocessStageError, PagePreprocessStageResult, PagePreprocessStageSuccess,
};
use super::{call_utility, call_utility2};

#[tauri::command]
pub async fn generate_file_name(
    handle: AppHandle,
    page_preprocess_stage: PagePreprocessStage,
) -> Result<PagePreprocessStageSuccess, PagePreprocessStageError> {
    let pages_paths = page_preprocess_stage.get_pages_paths();
    let document_directory = page_preprocess_stage.get_document_directory();
    for page_path in pages_paths.clone() {
        let file_name = page_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let destination_path = document_directory.join(file_name);
        fs::copy(page_path, destination_path).map_err(|e| PagePreprocessStageError {
            id: page_preprocess_stage.id.clone(),
            data_directory: page_preprocess_stage.data_directory.clone(),
            selected_pages: page_preprocess_stage.selected_pages.clone(),
            images_directory: page_preprocess_stage.images_directory.clone(),
            error_message: e.to_string(),
        })?;
    }
    let args = vec![
        "--input".to_owned(),
        document_directory.display().to_string(),
    ];
    let result = call_utility2(handle.clone(), "filenamegen".to_owned(), args, true).await;

    match result {
        Ok(output) => {
            let re = Regex::new(r"<output>([\s\S]*?)</output>").map_err(|e| {
                PagePreprocessStageError {
                    id: page_preprocess_stage.id.clone(),
                    data_directory: page_preprocess_stage.data_directory.clone(),
                    selected_pages: page_preprocess_stage.selected_pages.clone(),
                    images_directory: page_preprocess_stage.images_directory.clone(),
                    error_message: e.to_string(),
                }
            })?;

            let captures = re
                .captures(&output)
                .ok_or_else(|| PagePreprocessStageError {
                    id: page_preprocess_stage.id.clone(),
                    data_directory: page_preprocess_stage.data_directory.clone(),
                    selected_pages: page_preprocess_stage.selected_pages.clone(),
                    images_directory: page_preprocess_stage.images_directory.clone(),
                    error_message: "No output tags found".to_string(),
                })?;

            let json_str = captures
                .get(1)
                .ok_or_else(|| PagePreprocessStageError {
                    id: page_preprocess_stage.id.clone(),
                    data_directory: page_preprocess_stage.data_directory.clone(),
                    selected_pages: page_preprocess_stage.selected_pages.clone(),
                    images_directory: page_preprocess_stage.images_directory.clone(),
                    error_message: "No content between output tags".to_string(),
                })?
                .as_str();

            let preprocess_result: PagePreprocessStageResult = serde_json::from_str(json_str)
                .map_err(|e| PagePreprocessStageError {
                    id: page_preprocess_stage.id.clone(),
                    data_directory: page_preprocess_stage.data_directory.clone(),
                    selected_pages: page_preprocess_stage.selected_pages.clone(),
                    images_directory: page_preprocess_stage.images_directory.clone(),
                    error_message: e.to_string(),
                })?;

            let result_file_path = document_directory.join("result.json");
            fs::write(&result_file_path, json_str).map_err(|e| PagePreprocessStageError {
                id: page_preprocess_stage.id.clone(),
                data_directory: page_preprocess_stage.data_directory.clone(),
                selected_pages: page_preprocess_stage.selected_pages.clone(),
                images_directory: page_preprocess_stage.images_directory.clone(),
                error_message: format!("Failed to write result.json: {}", e),
            })?;

            Ok(PagePreprocessStageSuccess {
                id: page_preprocess_stage.id,
                selected_pages: page_preprocess_stage.selected_pages,
                data_directory: page_preprocess_stage.data_directory,
                images_directory: page_preprocess_stage
                    .images_directory
                    .to_string_lossy()
                    .to_string(),
                preprocess_pages_stage_result: preprocess_result,
            })
        }
        Err(e) => Err(PagePreprocessStageError {
            id: page_preprocess_stage.id,
            data_directory: page_preprocess_stage.data_directory,
            selected_pages: page_preprocess_stage.selected_pages,
            images_directory: page_preprocess_stage.images_directory,
            error_message: e,
        }),
    }
}

#[tauri::command]
pub async fn process_document(
    handle: AppHandle,
    document_process_stage: DocumentProcessStage,
) -> Result<DocumentProcessStageSuccess, DocumentProcessStageError> {
    let file_name = document_process_stage
        .preprocess_pages_stage_result
        .suggested_file_name
        .clone();
    let input_path = document_process_stage.document_path.clone();
    let data_directory = document_process_stage.data_directory.clone();
    let pages_to_process = document_process_stage
        .selected_pages
        .iter()
        .map(|page| page.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let output_dir = Path::new(&data_directory).join("documents");
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| DocumentProcessStageError {
            id: document_process_stage.id.clone(),
            selected_pages: document_process_stage.selected_pages.clone(),
            data_directory: document_process_stage.data_directory.clone(),
            images_directory: document_process_stage.images_directory.clone(),
            preprocess_pages_stage_result: document_process_stage.preprocess_pages_stage_result.clone(),
            document_path: document_process_stage.document_path.clone(),
            file_name: document_process_stage.file_name.clone(),
            error_message: format!("Failed to create output directory: {}", e),
        })?;
    }
    let output_path = output_dir.join(&file_name).with_extension("pdf");
    
    // QPDF utility call
    let is_success = call_utility(
        handle.clone(),
        "qpdf".to_owned(),
        vec![
            "--empty".to_owned(),
            "--pages".to_owned(),
            input_path.clone(),
            pages_to_process,
            "--".to_owned(),
            output_path.to_string_lossy().to_string(),
        ],
        false,
    )
    .await;

    if !is_success {
        return Err(DocumentProcessStageError {
            id: document_process_stage.id,
            selected_pages: document_process_stage.selected_pages,
            data_directory: document_process_stage.data_directory,
            images_directory: document_process_stage.images_directory,
            preprocess_pages_stage_result: document_process_stage.preprocess_pages_stage_result,
            document_path: document_process_stage.document_path,
            file_name: document_process_stage.file_name,
            error_message: "Failed to call QPDF utility".to_string(),
        });
    }

    // OCRmyPDF utility call
    let is_success = call_utility(
        handle.clone(),
        "ocrmypdf.exe".to_owned(),
        vec![
            "--force-ocr".to_owned(),
            "--pdf-renderer".to_owned(),
            "hocr".to_owned(),
            "--color-conversion-strategy".to_owned(),
            "UseDeviceIndependentColor".to_owned(),
            "-l".to_owned(),
            "por".to_owned(),
            "--clean".to_owned(),
            "--output-type".to_owned(),
            "pdfa-2".to_owned(),
            output_path.to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string(),
        ],
        false,
    )
    .await;

    if !is_success {
        return Err(DocumentProcessStageError {
            id: document_process_stage.id,
            selected_pages: document_process_stage.selected_pages,
            data_directory: document_process_stage.data_directory,
            images_directory: document_process_stage.images_directory,
            preprocess_pages_stage_result: document_process_stage.preprocess_pages_stage_result,
            document_path: document_process_stage.document_path,
            file_name: document_process_stage.file_name,
            error_message: "Failed to call OCRmyPDF utility".to_string(),
        });
    }

    Ok(DocumentProcessStageSuccess {
        id: document_process_stage.id,
        selected_pages: document_process_stage.selected_pages,
        data_directory: document_process_stage.data_directory,
        images_directory: document_process_stage.images_directory,
        preprocess_pages_stage_result: document_process_stage.preprocess_pages_stage_result,
        document_path: output_path.to_string_lossy().to_string(),
        file_name,
    })
}
