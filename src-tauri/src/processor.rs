use regex::Regex;
use std::fs;
use std::path::Path;
use tauri::AppHandle;

use super::models::workflows::{
    DocumentProcessStage, PagePreprocessStage, PagePreprocessStageResult,
};
use super::{call_utility, call_utility2};

#[tauri::command]
pub async fn generate_file_name(
    handle: AppHandle,
    page_preprocess_stage: PagePreprocessStage,
) -> Result<PagePreprocessStageResult, String> {
    let pages_paths = page_preprocess_stage.get_pages_paths();
    let document_directory = page_preprocess_stage.get_document_directory();
    for page_path in pages_paths.clone() {
        let file_name = page_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let destination_path = document_directory.join(file_name);
        fs::copy(page_path, destination_path).map_err(|e| e.to_string())?;
    }
    let args = vec![
        "--input".to_owned(),
        document_directory.display().to_string(),
    ];
    let result = call_utility2(handle.clone(), "filenamegen".to_owned(), args, true).await;
    match result {
        Ok(output) => {
            let re = Regex::new(r"<output>([\s\S]*?)</output>").map_err(|e| e.to_string())?;
            let captures = re.captures(&output).ok_or("No output tags found")?;
            let json_str = captures
                .get(1)
                .ok_or("No content between output tags")?
                .as_str();
            let result: PagePreprocessStageResult =
                serde_json::from_str(json_str).map_err(|e| e.to_string())?;
            let result_file_path = document_directory.join("result.json");
            fs::write(&result_file_path, json_str)
                .map_err(|e| format!("Failed to write result.json: {}", e))?;
            Ok(result)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn process_document(
    handle: AppHandle,
    document_process_stage: DocumentProcessStage,
) -> Result<(), String> {
    let file_name = document_process_stage
        .preprocess_pages_stage_result
        .suggested_file_name;
    let input_path = document_process_stage.document_path;
    let data_directory = document_process_stage.data_directory;
    let pages_to_process = document_process_stage
        .selected_pages
        .iter()
        .map(|page| page.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let output_dir = Path::new(&data_directory).join("documents");
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    }
    let output_path = output_dir.join(file_name).with_extension("pdf");
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
        return Err("Failed to call utility".to_string());
    } else {
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
            return Err("Failed to call utility".to_string());
        } else {
            Ok(())
        }
    }
}
// debug!("Debug: pages = {:?}", pages);
// debug!("Debug: data_path = {}", data_path);

// let pages_paths: Vec<String> = pages
//     .iter()
//     .map(|page| {
//         let path = format!("{}/{}.webp", data_path, page);
//         println!("Debug: Generated path = {}", path);
//         path
//     })
//     .collect();

// println!("Debug: pages_paths = {:?}", pages_paths);

// let mut args = vec!["-f".to_owned()];
// args.extend(pages_paths);
// args.extend(vec![
//     "-p".to_owned(),
//     prompt_file,
// ]);

// println!("Debug: final args = {:?}", args);

// let is_success = call_utility(handle.clone(), "filenamegen".to_owned(), args, true).await;

// if !is_success {
//     return Err("Failed to call utility".to_owned());
// }

// ######################################

// let is_success = call_utility(
//     handle.clone(),
//     "qpdf".to_owned(),
//     vec![
//         "--empty".to_owned(),
//         "--pages".to_owned(),
//         original_file,
//         pages.join(","),
//         "--".to_owned(),
//         output.clone(),
//     ],
//     false
// )
// .await;

// if !is_success {
//     return Err("Failed to call utility".to_string());
// } else {
//     let is_success = call_utility(
//         handle.clone(),
//         "ocrmypdf.exe".to_owned(),
//         vec![
//             "--force-ocr".to_owned(),
//             "--pdf-renderer".to_owned(),
//             "hocr".to_owned(),
//             "--color-conversion-strategy".to_owned(),
//             "UseDeviceIndependentColor".to_owned(),
//             "-l".to_owned(),
//             "por".to_owned(),
//             "--clean".to_owned(),
//             "--output-type".to_owned(),
//             "pdfa-2".to_owned(),
//             output.clone(),
//             output,
//         ],
//         false
//     )
//     .await;
// }

//     Ok(())
// }

// use std::path::Path;

// use crate::llm::models::DocumentInfo;
// use regex::Regex;
// use tauri::async_runtime;
// use tauri_plugin_shell::{process::CommandEvent, ShellExt};

// #[tauri::command]
// pub async fn final_pipeline(
//     handle: tauri::AppHandle,
//     document_info: DocumentInfo,
// ) -> Result<(), String> {
//     let parent_dir = Path::new(&document_info.json_file_path).parent().expect("Failed to get parent directory");
//     let re = Regex::new(r"(.+)-data$").unwrap();
//     let original_file = re.replace(&parent_dir.to_string_lossy(), "$1").to_string();
//     let original_file = Path::new(&original_file).with_extension("pdf");
//     let done_dir = parent_dir.join("done");

//     if !done_dir.exists() {
//         std::fs::create_dir_all(&done_dir).map_err(|_| "Failed to create done directory")?;
//     }

//     let save_path = done_dir.join(document_info.file_name).with_extension("pdf");

//     if save_path.exists() {
//         std::fs::remove_file(&save_path).map_err(|_| "Failed to delete file")?;
//     }

//     let mut pages = vec![];

//     for page in document_info.pages_paths {
//         let page_number = extract_page_number(&page);
//         pages.push(page_number.to_string());
//     }

//     let success = call_utility(
//         handle.clone(),
//         "qpdf".to_owned(),
//         vec![
//             "--empty".to_string(),
//             "--pages".to_string(),
//             original_file.to_string_lossy().to_string(),
//             pages.join(","),
//             "--".to_string(),
//             save_path.to_string_lossy().to_string(),
//         ],
//     )
//     .await;

//     if success {
//         let success = call_utility(
//             handle.clone(),
//             "ocrmypdf.exe".to_owned(),
//             vec![
//                 "--force-ocr".to_string(),
//                 "--pdf-renderer".to_string(),
//                 "hocr".to_string(),
//                 "--color-conversion-strategy".to_string(),
//                 "UseDeviceIndependentColor".to_string(),
//                 "-l".to_string(),
//                 "por".to_string(),
//                 "--clean".to_string(),
//                 "--output-type".to_string(),
//                 "pdfa-2".to_string(),
//                 save_path.to_string_lossy().to_string(),
//                 save_path.to_string_lossy().to_string(),
//             ],
//         ).await;
//         if !success {
//             return Err("Failed to call utility".to_string());
//         }
//     }

//     Ok(())
// }

// async fn call_utility(handle: tauri::AppHandle, utility: String, args: Vec<String>) -> bool {
//     let spawn_utility = async_runtime::spawn(async move {
//         let (mut rx, child) = handle
//             .shell()
//             .command(utility)
//             .args(args)
//             .spawn()
//             .expect("Failed to spawn process");

//         let mut is_success = false;

//         while let Some(event) = rx.recv().await {
//             match event {
//                 CommandEvent::Stdout(data) => {
//                     println!("{}", String::from_utf8_lossy(&data));
//                 }
//                 CommandEvent::Stderr(data) => {
//                     println!("{}", String::from_utf8_lossy(&data));
//                 }
//                 CommandEvent::Terminated(status) => {
//                     if let Some(code) = status.code {
//                         println!("Process terminated with status: {}", code);
//                         if code == 0 {
//                             is_success = true;
//                         }
//                     }
//                     if let Some(signal) = status.signal {
//                         println!("Process terminated with signal: {}", signal);
//                     }
//                 }
//                 _ => {}
//             }
//         }
//         child.kill().expect("Failed to kill process");
//         is_success
//     });

//     spawn_utility.await.unwrap()
// }

// fn extract_page_number(input: &str) -> &str {
//     let re = Regex::new(r"page-(\d+)").expect("Regex should never fail");
//     re.captures(input)
//         .and_then(|caps| caps.get(1).map(|m| m.as_str()))
//         .unwrap_or("unidentified")
// }

// #[tauri::command]
// pub fn open_in_explorer(path: &str) -> Result<(), String> {
//     let mut command = std::process::Command::new("explorer");
//     command.args(&["/select,", path]);
//     command.spawn().map_err(|_| "Failed to open in explorer")?;
//     Ok(())
// }
