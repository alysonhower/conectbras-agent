use log::{debug, error};
use std::{fs::create_dir_all, path::PathBuf, time::Instant};
use tauri::{AppHandle, Emitter};

use super::workflows::*;

impl PagePreprocessStage {
    pub fn get_pages_paths(&self) -> Vec<PathBuf> {
        self.selected_pages
            .iter()
            .map(|page| {
                self.images_directory
                    .join(page.to_string())
                    .with_extension("webp")
            })
            .collect()
    }
    pub fn get_document_directory(&self) -> PathBuf {
        let page_numbers = self
            .selected_pages
            .iter()
            .map(|&num| num.to_string())
            .collect::<Vec<String>>()
            .join("-");
        let document_directory = self.images_directory.join(page_numbers);
        if !document_directory.exists() {
            let _ = create_dir_all(&document_directory);
        }
        document_directory
    }
}

impl ProgressState {
    pub fn new(total_document_pages: usize) -> Self {
        Self {
            pages_processed: 0,
            pages_to_process: 0,
            total_document_pages,
            estimated_seconds_remaining: 0,
            extracted_page_numbers: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        current_progress: usize,
        num_missing_pages: usize,
        start_time: Instant,
        all_extracted: Vec<usize>,
        app: &AppHandle,
    ) -> Result<(), String> {
        self.pages_processed = current_progress;
        self.pages_to_process = num_missing_pages;

        let elapsed = start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();

        if current_progress > 0 {
            let pages_per_second = current_progress as f64 / elapsed_secs;
            let remaining_pages = num_missing_pages.saturating_sub(current_progress) as f64;
            self.estimated_seconds_remaining = (remaining_pages / pages_per_second) as u64;
        } else {
            self.estimated_seconds_remaining = (num_missing_pages * 1) as u64; // Assume 2 seconds per page as a starting point
        }

        self.extracted_page_numbers = all_extracted;

        app.emit("progress", self.clone()).map_err(|e| {
            error!("Failed to emit progress event: {}", e);
            format!("Failed to emit progress event: {}", e)
        })?;

        debug!(
            "Emitted progress: {}/{}, est. remaining: {} seconds",
            self.pages_processed, self.pages_to_process, self.estimated_seconds_remaining
        );

        Ok(())
    }
}
