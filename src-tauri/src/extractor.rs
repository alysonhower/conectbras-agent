use crate::utilities::call_utility;
use log::{debug, error, warn};
use lopdf::Document;
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::channel,
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use sys_info;
use tauri::{AppHandle, Emitter, Listener};
use threadpool::ThreadPool;
use tokio::time::timeout;

const MIN_BATCH_SIZE: usize = 5;
const MAX_BATCH_SIZE: usize = 20;
const IMAGE_DENSITY: &str = "150";
const IMAGE_RESIZE: &str = "1500x1500";
const MAX_RETRIES: usize = 3;
const MAX_TIMEOUT: u64 = 60;

#[derive(serde::Serialize, Clone, Debug)]
struct ProgressUpdate {
    pages_processed: usize,
    pages_to_process: usize,
    total_document_pages: usize,
    estimated_seconds_remaining: u64,
    extracted_page_numbers: Vec<usize>,
}

#[tauri::command]
pub async fn extract_document_images(
    app: AppHandle,
    document_path: PathBuf,
    images_directory: PathBuf,
) -> Result<String, String> {
    let document = load_document(&document_path)?;
    let total_pages = document.get_pages().len();
    let (missing_pages, extracted_pages) = get_missing_pages(&images_directory, total_pages)?;

    if missing_pages.is_empty() {
        app.emit("total-extracted-pages", total_pages)
            .map_err(|e| format!("Failed to emit event: {}", e))?;
        return Ok(format!(
            "All images already extracted. Found {} matching .webp files.",
            total_pages
        ));
    }

    process_missing_pages(app, document_path, images_directory, missing_pages, extracted_pages, total_pages).await
}

fn load_document(document_path: &PathBuf) -> Result<Document, String> {
    Document::load(document_path).map_err(|e| {
        error!("Failed to load PDF: {}", e);
        format!("Failed to load PDF: {}", e)
    })
}

fn get_missing_pages(images_directory: &PathBuf, total_pages: usize) -> Result<(Vec<usize>, Vec<usize>), String> {
    let output_path = Path::new(images_directory);
    if !output_path.exists() {
        fs::create_dir_all(images_directory)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
        return Ok(((1..=total_pages).collect(), vec![]));
    }

    let webp_files: Vec<_> = fs::read_dir(output_path)
        .map_err(|e| format!("Failed to read output directory: {}", e))?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("webp") {
                    path.file_stem().and_then(|s| s.to_str()).and_then(|s| s.parse::<usize>().ok())
                } else {
                    None
                }
            })
        })
        .collect();

    let missing_pages: Vec<usize> = (1..=total_pages)
        .filter(|&page| !webp_files.contains(&page))
        .collect();

    let extracted_pages: Vec<usize> = webp_files.into_iter().filter(|&page| page <= total_pages).collect();

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
    let all_extracted_pages = Arc::new(Mutex::new(extracted_pages));
    let start_time = Instant::now();

    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_flag_clone = cancel_flag.clone();

    let cancel_listener = app.listen("cancel-processing", move |_| {
        cancel_flag_clone.store(true, Ordering::SeqCst);
        debug!("Cancellation requested");
    });

    let batch_size = get_adaptive_batch_size();
    let pool = ThreadPool::new(batch_size);
    let (tx, rx) = channel();

    // Initial progress update
    app.emit(
        "progress",
        ProgressUpdate {
            pages_processed: 0,
            pages_to_process: num_missing_pages,
            total_document_pages: total_pages,
            estimated_seconds_remaining: 0,
            extracted_page_numbers: all_extracted_pages.lock().unwrap().clone(),
        },
    )
    .map_err(|e| format!("Failed to emit initial progress event: {}", e))?;

    for batch in missing_pages.chunks(batch_size) {
        if cancel_flag.load(Ordering::SeqCst) {
            debug!("Processing cancelled");
            break;
        }

        process_batch(
            &app,
            &document_path,
            &images_directory,
            batch,
            &progress,
            &cancel_flag,
            &pool,
            &tx,
        );

        handle_batch_results(
            &rx,
            batch.len(),
            &all_extracted_pages,
            &failures,
            &progress,
            &app,
            num_missing_pages,
            total_pages,
            start_time,
        )?;
    }

    pool.join();

    app.unlisten(cancel_listener);

    finalize_processing(&app, &progress, &failures, num_missing_pages, total_pages, &cancel_flag)
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

fn extract_page_images(
    app: &AppHandle,
    document_path: &PathBuf,
    images_directory: &PathBuf,
    page: usize,
    progress: &Arc<AtomicUsize>,
    cancel_flag: &Arc<AtomicBool>,
) -> (usize, bool) {
    for _ in 0..MAX_RETRIES {
        if cancel_flag.load(Ordering::SeqCst) {
            debug!("Page processing cancelled for page {}", page);
            return (page, false);
        }

        let args = vec![
            "-density".to_owned(),
            IMAGE_DENSITY.to_owned(),
            format!("{}[{}]", document_path.display(), page - 1),
            "-resize".to_owned(),
            IMAGE_RESIZE.to_owned(),
            format!("{}\\{}.webp", images_directory.display(), page),
        ];

        let result = tauri::async_runtime::block_on(async {
            match timeout(
                Duration::from_secs(MAX_TIMEOUT),
                call_utility(app.clone(), "magick.exe".to_owned(), args, false),
            )
            .await
            {
                Ok(result) => result,
                Err(_) => {
                    warn!("Page processing timed out for page {}", page);
                    false
                }
            }
        });

        if result {
            progress.fetch_add(1, Ordering::SeqCst);
            return (page, true);
        }
    }

    warn!(
        "Failed to process page {} after {} retries",
        page, MAX_RETRIES
    );
    (page, false)
}

fn process_batch(
    app: &AppHandle,
    document_path: &PathBuf,
    images_directory: &PathBuf,
    batch: &[usize],
    progress: &Arc<AtomicUsize>,
    cancel_flag: &Arc<AtomicBool>,
    pool: &ThreadPool,
    tx: &std::sync::mpsc::Sender<(usize, bool)>,
) {
    batch.par_iter().for_each(|&page| {
        let app = app.clone();
        let document_path = document_path.clone();
        let images_directory = images_directory.clone();
        let progress = Arc::clone(progress);
        let cancel_flag = Arc::clone(cancel_flag);
        let tx = tx.clone();

        pool.execute(move || {
            let result = extract_page_images(
                &app,
                &document_path,
                &images_directory,
                page,
                &progress,
                &cancel_flag,
            );
            tx.send(result).expect("Channel send failed");
        });
    });
}

fn handle_batch_results(
    rx: &std::sync::mpsc::Receiver<(usize, bool)>,
    batch_size: usize,
    all_extracted_pages: &Arc<Mutex<Vec<usize>>>,
    failures: &Arc<Mutex<Vec<usize>>>,
    progress: &Arc<AtomicUsize>,
    app: &AppHandle,
    num_missing_pages: usize,
    total_pages: usize,
    start_time: Instant,
) -> Result<(), String> {
    for _ in 0..batch_size {
        if let Ok((page, success)) = rx.recv() {
            if success {
                all_extracted_pages.lock().unwrap().push(page);
            } else {
                failures.lock().unwrap().push(page);
            }
        }
    }

    let current_progress = progress.load(Ordering::SeqCst);
    let elapsed = start_time.elapsed().as_secs();
    let estimated_total_seconds =
        (elapsed as f64 / current_progress as f64) * num_missing_pages as f64;
    let estimated_seconds_remaining = estimated_total_seconds as u64 - elapsed;

    let mut all_extracted = all_extracted_pages.lock().unwrap().clone();
    all_extracted.sort_unstable();

    app.emit(
        "progress",
        ProgressUpdate {
            pages_processed: current_progress,
            pages_to_process: num_missing_pages,
            total_document_pages: total_pages,
            estimated_seconds_remaining,
            extracted_page_numbers: all_extracted,
        },
    )
    .map_err(|e| format!("Failed to emit progress event: {}", e))?;

    debug!(
        "Emitted progress: {}/{}, est. remaining: {} seconds",
        current_progress, num_missing_pages, estimated_seconds_remaining
    );

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
    let failures = failures.lock().unwrap();

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
