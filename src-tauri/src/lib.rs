mod models;
mod extractor;
mod processor;
mod utilities;
pub use utilities::{call_utility, call_utility2};
use extractor::run_extract_document_images_stage;
use processor::{run_page_preprocess_stage, run_document_process_stage, run_update_file_name};
// use llm::{anthropic_pipeline, update_file_name, rename_finished_document};
// use processor::{final_pipeline, open_in_explorer};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // anthropic_pipeline,
            // update_file_name,
            // final_pipeline,
            // open_in_explorer,
            // rename_finished_document
            run_extract_document_images_stage,
            run_page_preprocess_stage,
            run_document_process_stage,
            run_update_file_name
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
