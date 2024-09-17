// use super::models::workflows::{ExtractDocumentImagesStage, ExtractDocumentImagesStageSuccess, ExtractDocumentImagesStageError, ProgressState};
use super::models::workflows::{ExtractDocumentImagesStage, ProgressState};
use crate::utilities::call_utility;
use log::{debug, error, warn};
use lopdf::Document;
use rayon::prelude::*;
use std::{
    fs::{self, copy, create_dir_all},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use sys_info;
use tauri::{AppHandle, Emitter, Listener};
use tokio::time::timeout;

const MIN_BATCH_SIZE: usize = 5;
const MAX_BATCH_SIZE: usize = 20;
const IMAGE_DENSITY: &str = "150";
const IMAGE_RESIZE: &str = "1500x1500";
const MAX_RETRIES: usize = 3;
const MAX_TIMEOUT: u64 = 60;

#[tauri::command]
pub async fn extract_document_images(
    app: AppHandle,
    extract_document_images_stage: ExtractDocumentImagesStage,
) -> Result<String, String> {
    let ExtractDocumentImagesStage {
        document_path,
        document_clone_path,
        images_directory,
    } = extract_document_images_stage;
    create_dir_all(&images_directory).map_err(|e| {
        error!("Failed to create output directory: {}", e);
        format!("Failed to create output directory: {}", e)
    })?;

    copy(&document_path, &document_clone_path).map_err(|e| {
        error!("Failed to copy document to data directory: {}", e);
        format!("Failed to copy document to data directory: {}", e)
    })?;

    let document = load_document(&document_clone_path)?;
    let total_pages = document.get_pages().len();
    let (missing_pages, extracted_pages) = get_missing_pages(&images_directory, total_pages)?;

    if missing_pages.is_empty() {
        app.emit("total-extracted-pages", total_pages)
            .map_err(|e| {
                error!("Failed to emit total-extracted-pages event: {}", e);
                format!("Failed to emit total-extracted-pages event: {}", e)
            })?;
        return Ok(format!(
            "All images already extracted. Found {} matching the total number of pages in the document.",
            total_pages
        ));
    }

    process_missing_pages(
        app,
        document_clone_path,
        images_directory,
        missing_pages,
        extracted_pages,
        total_pages,
    )
    .await
}

fn load_document(document_path: &PathBuf) -> Result<Document, String> {
    Document::load(document_path).map_err(|e| {
        error!("Failed to load PDF: {}", e);
        format!("Failed to load PDF: {}", e)
    })
}

fn get_missing_pages(
    images_directory: &PathBuf,
    total_pages: usize,
) -> Result<(Vec<usize>, Vec<usize>), String> {
    let output_path = Path::new(images_directory);
    if !output_path.exists() {
        fs::create_dir_all(images_directory).map_err(|e| {
            error!("Failed to create output directory: {}", e);
            format!("Failed to create output directory: {}", e)
        })?;
        return Ok(((1..=total_pages).collect(), vec![]));
    }

    let webp_files: Vec<_> = fs::read_dir(output_path)
        .map_err(|e| {
            error!("Failed to read output directory: {}", e);
            format!("Failed to read output directory: {}", e)
        })?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("webp") {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .and_then(|s| s.parse::<usize>().ok())
                        .map(|n| n + 1) // Adjust for 0-based indexing
                } else {
                    None
                }
            })
        })
        .collect();

    let missing_pages: Vec<usize> = (1..=total_pages)
        .into_par_iter()
        .filter(|&page| !webp_files.contains(&page))
        .collect();

    let extracted_pages: Vec<usize> = webp_files
        .into_iter()
        .filter(|&page| page <= total_pages)
        .collect();

    Ok((missing_pages, extracted_pages))
}

async fn process_missing_pages(
    app: AppHandle,
    document_path: PathBuf,
    images_directory: PathBuf,
    missing_pages: Vec<usize>,
    extracted_pages: Vec<usize>,
    total_pages: usize,
) -> Result<String, String> {
    let num_missing_pages = missing_pages.len();
    let progress = Arc::new(AtomicUsize::new(0));
    let failures = Arc::new(Mutex::new(Vec::new()));
    let all_extracted_pages = Arc::new(Mutex::new(extracted_pages.clone()));
    let start_time = Instant::now();

    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_flag_clone = cancel_flag.clone();

    let cancel_listener = app.listen("cancel-processing", move |_| {
        cancel_flag_clone.store(true, Ordering::SeqCst);
        debug!("Cancellation requested");
    });

    let batch_size = get_adaptive_batch_size();
    let batches: Vec<_> = missing_pages.chunks(batch_size).collect();

    let mut progress_state = ProgressState::new(total_pages);
    progress_state.pages_to_process = num_missing_pages;
    progress_state.extracted_page_numbers = extracted_pages.clone();
    progress_state.update(0, num_missing_pages, start_time, extracted_pages, &app)?;

    for batch in batches {
        if cancel_flag.load(Ordering::SeqCst) {
            break;
        }

        let (successful_pages, failed_pages) = process_batch(
            &app,
            &document_path,
            &images_directory,
            batch,
            &progress,
            &cancel_flag,
        )
        .await?;

        handle_batch_results(
            &successful_pages,
            &failed_pages,
            &all_extracted_pages,
            &failures,
            &progress,
            &app,
            num_missing_pages,
            start_time,
            &mut progress_state,
            &images_directory,
        )?;
    }

    app.unlisten(cancel_listener);

    finalize_processing(
        &app,
        &progress,
        &failures,
        num_missing_pages,
        total_pages,
        &cancel_flag,
    )
}

async fn process_batch(
    app: &AppHandle,
    document_path: &PathBuf,
    images_directory: &PathBuf,
    batch: &[usize],
    progress: &Arc<AtomicUsize>,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<(Vec<usize>, Vec<usize>), String> {
    for _ in 0..MAX_RETRIES {
        if cancel_flag.load(Ordering::SeqCst) {
            debug!("Batch processing cancelled");
            return Ok((vec![], batch.to_vec()));
        }

        let page_spec = create_page_spec(batch);
        let document_path_with_pages = format!("{}[{}]", document_path.display(), page_spec);

        let args = vec![
            "-density".to_owned(),
            IMAGE_DENSITY.to_owned(),
            document_path_with_pages,
            "-resize".to_owned(),
            IMAGE_RESIZE.to_owned(),
            format!("{}\\index-%d.webp", images_directory.display()),
        ];

        println!("args: {:?}", args);

        let result = match timeout(
            Duration::from_secs(MAX_TIMEOUT),
            call_utility(app.clone(), "magick.exe".to_owned(), args, false),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                warn!("Batch processing timed out");
                false
            }
        };

        if result {
            progress.fetch_add(batch.len(), Ordering::SeqCst);
            return Ok((batch.to_vec(), vec![]));
        }
    }

    warn!("Failed to process batch after {} retries", MAX_RETRIES);
    Ok((vec![], batch.to_vec()))
}

fn get_adaptive_batch_size() -> usize {
    let cpu_count = match sys_info::cpu_num() {
        Ok(count) => count as usize,
        Err(e) => {
            warn!("Failed to get CPU count: {}. Falling back to 4.", e);
            4
        }
    };

    let (total_mem, available_mem) = match sys_info::mem_info() {
        Ok(mem) => {
            let total = mem.total as f64 * 1024.0;
            let avail = mem.free as f64 * 1024.0;
            debug!(
                "Raw memory info: total = {} KB, free = {} KB",
                mem.total, mem.free
            );
            (total, avail)
        }
        Err(e) => {
            warn!("Failed to get memory info: {}. Using default values.", e);
            (
                8.0 * 1024.0 * 1024.0 * 1024.0,
                4.0 * 1024.0 * 1024.0 * 1024.0,
            )
        }
    };

    let mem_threshold = total_mem / 4.0;

    debug!(
        "Total memory: {:.2} GB, Available memory: {:.2} GB",
        total_mem / 1_073_741_824.0,
        available_mem / 1_073_741_824.0
    );
    debug!(
        "Memory threshold: {:.2} GB",
        mem_threshold / 1_073_741_824.0
    );

    let mem_factor = if available_mem > mem_threshold {
        1.0 + (2.0 * available_mem / total_mem)
    } else {
        0.5 + (0.5 * available_mem / mem_threshold)
    };

    debug!("Memory factor: {:.2}", mem_factor);

    let batch_size = (cpu_count as f64 * mem_factor).round() as usize;
    let clamped_batch_size = batch_size.clamp(MIN_BATCH_SIZE, MAX_BATCH_SIZE);

    debug!(
        "Calculated batch size: {}, Clamped batch size: {}",
        batch_size, clamped_batch_size
    );

    clamped_batch_size
}

fn create_page_spec(pages: &[usize]) -> String {
    let mut ranges = vec![];
    let mut current_range = (pages[0], pages[0]);

    for &page in &pages[1..] {
        if page == current_range.1 + 1 {
            current_range.1 = page;
        } else {
            ranges.push(current_range);
            current_range = (page, page);
        }
    }
    ranges.push(current_range);

    ranges
        .into_iter()
        .map(|(start, end)| {
            if start == end {
                (start - 1).to_string()
            } else {
                format!("{}-{}", start - 1, end - 1)
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn handle_batch_results(
    successful_pages: &[usize],
    failed_pages: &[usize],
    all_extracted_pages: &Arc<Mutex<Vec<usize>>>,
    failures: &Arc<Mutex<Vec<usize>>>,
    progress: &Arc<AtomicUsize>,
    app: &AppHandle,
    num_missing_pages: usize,
    start_time: Instant,
    progress_state: &mut ProgressState,
    images_directory: &PathBuf,
) -> Result<(), String> {
    let (result1, result2) = rayon::join(
        || {
            all_extracted_pages
                .lock()
                .map_err(|e| {
                    error!("Failed to lock all_extracted_pages: {}", e);
                    format!("Failed to lock all_extracted_pages: {}", e)
                })
                .and_then(|mut pages| {
                    pages.extend(successful_pages);
                    pages.par_sort_unstable();
                    Ok(())
                })
        },
        || {
            failures
                .lock()
                .map_err(|e| {
                    error!("Failed to lock failures: {}", e);
                    format!("Failed to lock failures: {}", e)
                })
                .map(|mut fails| fails.extend(failed_pages))
        },
    );

    // Handle potential errors from both operations
    result1?;
    result2?;

    let current_progress = progress.load(Ordering::SeqCst);
    let all_extracted = all_extracted_pages
        .lock()
        .map_err(|e| {
            error!("Failed to lock all_extracted_pages: {}", e);
            format!("Failed to lock all_extracted_pages: {}", e)
        })?
        .clone();

    if current_progress > 0 {
        progress_state.update(
            current_progress,
            num_missing_pages,
            start_time,
            all_extracted.clone(),
            app,
        )?;

        // Rename the extracted images
        rename_extracted_images(successful_pages, images_directory)?;
    }

    Ok(())
}

fn rename_extracted_images(
    successful_pages: &[usize],
    images_directory: &PathBuf,
) -> Result<(), String> {
    for &page in successful_pages {
        let old_name = images_directory.join(format!("index-{}.webp", page - 1));
        let new_name = images_directory.join(format!("{}.webp", page));

        if let Err(e) = fs::rename(&old_name, &new_name) {
            error!(
                "Failed to rename file from {:?} to {:?}: {}",
                old_name, new_name, e
            );
            return Err(format!("Failed to rename file: {}", e));
        }
    }
    Ok(())
}

fn finalize_processing(
    app: &AppHandle,
    progress: &Arc<AtomicUsize>,
    failures: &Arc<Mutex<Vec<usize>>>,
    num_missing_pages: usize,
    total_pages: usize,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<String, String> {
    if let Err(e) = app.emit("webp_files_match", total_pages) {
        warn!("Failed to emit webp_files_match event: {}", e);
    }

    let processed_pages = progress.load(Ordering::SeqCst);
    let failures = failures.lock().map_err(|e| {
        error!("Failed to lock failures: {}", e);
        format!("Failed to lock failures: {}", e)
    })?;

    if cancel_flag.load(Ordering::SeqCst) {
        Ok(format!(
            "Document processing cancelled. {} out of {} missing pages extracted.",
            processed_pages, num_missing_pages
        ))
    } else if failures.is_empty() {
        Ok(format!(
            "Document processed successfully. {} missing pages extracted.",
            processed_pages
        ))
    } else {
        Err(format!(
            "Processed {} missing pages. Failed to process pages: {:?}",
            processed_pages, failures
        ))
    }
}
