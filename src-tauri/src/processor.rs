use regex::Regex;
use std::fs;
use std::path::Path;
use tauri::AppHandle;

use super::models::workflows::{
    DocumentProcessStage, DocumentProcessStageError, DocumentProcessStageSuccess,
    PagePreprocessStage, PagePreprocessStageError, PagePreprocessStageResult,
    PagePreprocessStageSuccess,
};
use super::{call_utility, call_utility2};

#[tauri::command]
pub async fn run_page_preprocess_stage(
    handle: AppHandle,
    page_preprocess_stage: PagePreprocessStage,
) -> Result<PagePreprocessStageSuccess, PagePreprocessStageError> {
    // Introduce a test error condition
    if page_preprocess_stage.id == "test_error" {
        return Err(PagePreprocessStageError {
            id: page_preprocess_stage.id,
            data_directory: page_preprocess_stage.data_directory,
            selected_pages: page_preprocess_stage.selected_pages,
            images_directory: page_preprocess_stage.images_directory,
            error_message: "Forced error for testing".to_string(),
        });
    }

    let page_number_prefix = format!("p-{}", page_preprocess_stage.selected_pages.iter().map(|&x| x.to_string()).collect::<Vec<String>>().join("-"));
    let pages_paths = page_preprocess_stage.get_pages_paths();
    let preprocessed_pages_directory = page_preprocess_stage.get_preprocessed_pages_directory();
    for page_path in pages_paths.clone() {
        let file_name = page_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let destination_path = preprocessed_pages_directory.join(file_name);
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
        preprocessed_pages_directory.display().to_string(),
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

            let result_file_path = preprocessed_pages_directory.join("result.json");
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
                images_directory: page_preprocess_stage.images_directory,
                page_preprocess_stage_result: preprocess_result,
                page_number_prefix,
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
pub async fn run_document_process_stage(
    handle: AppHandle,
    document_process_stage: DocumentProcessStage,
) -> Result<DocumentProcessStageSuccess, DocumentProcessStageError> {
    if document_process_stage.id == "test_error" {
        return Err(DocumentProcessStageError {
            id: document_process_stage.id,
            selected_pages: document_process_stage.selected_pages,
            data_directory: document_process_stage.data_directory,
            images_directory: document_process_stage.images_directory,
            page_preprocess_stage_result: document_process_stage.page_preprocess_stage_result,
            document_path: document_process_stage.document_path,
            file_name: document_process_stage.file_name,
            error_message: "Forced error for testing".to_string(),
            page_number_prefix: document_process_stage.page_number_prefix,
        });
    }


    let file_name = document_process_stage
        .page_preprocess_stage_result
        .suggested_file_name
        .clone();

    let file_name = format!("{}-{}.pdf", document_process_stage.page_number_prefix, file_name);
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
            page_preprocess_stage_result: document_process_stage
                .page_preprocess_stage_result
                .clone(),
            document_path: document_process_stage.document_path.clone(),
            file_name: document_process_stage.file_name.clone(),
            error_message: format!("Failed to create output directory: {}", e),
            page_number_prefix: document_process_stage.page_number_prefix.clone(),
        })?;
    }

    let output_path = output_dir
        .join(&file_name)
        .with_extension("pdf")
        .display()
        .to_string();

    // QPDF utility call
    let is_success = call_utility(
        handle.clone(),
        "qpdf".to_owned(),
        vec![
            "--empty".to_owned(),
            "--pages".to_owned(),
            input_path,
            pages_to_process,
            "--".to_owned(),
            output_path.clone(),
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
            page_preprocess_stage_result: document_process_stage.page_preprocess_stage_result,
            document_path: document_process_stage.document_path,
            file_name: document_process_stage.file_name,
            error_message: "Failed to call QPDF utility".to_string(),
            page_number_prefix: document_process_stage.page_number_prefix,
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
            output_path.clone(),
            output_path.clone(),
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
            page_preprocess_stage_result: document_process_stage.page_preprocess_stage_result,
            document_path: document_process_stage.document_path,
            file_name: document_process_stage.file_name,
            error_message: "Failed to call OCRmyPDF utility".to_string(),
            page_number_prefix: document_process_stage.page_number_prefix,
        });
    }




    Ok(DocumentProcessStageSuccess {
        id: document_process_stage.id,
        selected_pages: document_process_stage.selected_pages,
        data_directory: document_process_stage.data_directory,
        images_directory: document_process_stage.images_directory,
        page_preprocess_stage_result: document_process_stage.page_preprocess_stage_result,
        document_path: output_path,
        file_name,
        page_number_prefix: document_process_stage.page_number_prefix,
    })
}


#[tauri::command]
pub fn run_update_file_name(file_name: String, document_path: String) -> Result<String, String> {
    let document_path = Path::new(&document_path);
    let new_file_name = document_path
        .with_file_name(file_name)
        .with_extension("pdf");
    fs::rename(&document_path, &new_file_name).map_err(|e| e.to_string())?;
    Ok(new_file_name.display().to_string())
}

#[tauri::command]
pub fn open_in_explorer(path: &str) -> Result<(), String> {
    let mut command = std::process::Command::new("explorer");
    command.args(&["/select,", path]);
    command.spawn().map_err(|_| "Failed to open in explorer")?;
    Ok(())
}