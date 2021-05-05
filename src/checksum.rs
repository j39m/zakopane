// Implements the `zakopane checksum` subcommand.

use std::io::Read;
use std::io::Write;

use crate::structs::ZakopaneError;

const MAX_TASKS: usize = 8;
const READ_SIZE: usize = 2 << 20;

struct ChecksumWithPath {
    checksum: String,
    path: std::path::PathBuf,
}

impl ChecksumWithPath {
    pub fn new(checksum: String, path: std::path::PathBuf) -> Self {
        Self { checksum, path }
    }
}

type ChecksumResult = Result<ChecksumWithPath, ZakopaneError>;
type ChecksumTaskJoinHandle = tokio::task::JoinHandle<Vec<ChecksumWithPath>>;

struct ChecksumTaskDispatcherData {
    path: std::path::PathBuf,

    // Rate-limiter for spawned checksum tasks.
    semaphore: std::sync::Arc<tokio::sync::Semaphore>,

    // Counts total number of spawned checksum tasks.
    spawn_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,

    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
}

impl ChecksumTaskDispatcherData {
    pub fn new(
        path: std::path::PathBuf,
        spawn_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        sender: tokio::sync::mpsc::Sender<ChecksumResult>,
    ) -> Self {
        Self {
            path,
            semaphore: std::sync::Arc::new(tokio::sync::Semaphore::new(MAX_TASKS)),
            spawn_counter,
            sender,
        }
    }
}

// Sends a checksum task result downstream to the collector task.
fn checksum_task_send_result(
    result: ChecksumResult,
    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
) {
    if let Err(e) = sender.blocking_send(result) {
        eprintln!("BUG: Sender::send() failed: {}", e);
    }
}

fn checksum_task_impl(path: std::path::PathBuf) -> ChecksumResult {
    let mut hasher = crypto_hash::Hasher::new(crypto_hash::Algorithm::SHA256);
    let mut buffer: Vec<u8> = vec![0; READ_SIZE];
    let mut file = std::fs::File::open(&path).map_err(ZakopaneError::Io)?;
    loop {
        let read_bytes = file.read(&mut buffer).map_err(ZakopaneError::Io)?;
        if read_bytes == 0 {
            let checksum = hasher
                .finish()
                .into_iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<Vec<String>>()
                .join("");
            return Ok(ChecksumWithPath::new(checksum, path));
        }
        hasher
            .write_all(&buffer[..read_bytes])
            .map_err(ZakopaneError::Io)?;
    }
}

// Represents a spawned checksum task.
//
// `_permit` is moved into this checksum task simply to hold onto the
// semaphore-dispensed resource for the duration of this task.
fn checksum_task(
    path: std::path::PathBuf,
    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    let result = checksum_task_impl(path);
    checksum_task_send_result(result, sender);
}

// Represents the spawned collection task.
async fn collector_task(
    mut receiver: tokio::sync::mpsc::Receiver<ChecksumResult>,
    spawn_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
) -> Vec<ChecksumWithPath> {
    let mut sums: Vec<ChecksumWithPath> = Vec::new();
    let mut errors: usize = 0;
    loop {
        while let Some(result) = receiver.recv().await {
            match result {
                Ok(digest_with_path) => sums.push(digest_with_path),
                Err(_) => errors += 1,
            }
        }
        if sums.len() + errors == spawn_counter.load(std::sync::atomic::Ordering::SeqCst) {
            return sums;
        }
    }
}

// Spawns the collector task that listens for checksum results
// provided by the checksum tasks.
async fn spawn_collector(
    path: std::path::PathBuf,
) -> (ChecksumTaskDispatcherData, ChecksumTaskJoinHandle) {
    let (sender, receiver) = tokio::sync::mpsc::channel(MAX_TASKS);
    let spawn_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let join_handle = tokio::task::spawn(collector_task(receiver, spawn_counter.clone()));
    (
        ChecksumTaskDispatcherData::new(path, spawn_counter, sender),
        join_handle,
    )
}

async fn spawn_checksum_tasks(context: ChecksumTaskDispatcherData) {
    let walk_iter = walkdir::WalkDir::new(&context.path).into_iter();
    // The `filter_entry()` call is crafted s.t.
    // *    we skip and don't descend into hidden directories
    // *    unless the hidden directory is the target directory, because
    //      the target directory is always the first yielded value from
    //      the `WalkDir`.
    for entry in walk_iter.filter_entry(|e| {
        e.path() == &context.path
            || !e
                .file_name()
                .to_str()
                .map(|path| path.starts_with("."))
                .unwrap_or(false)
    }) {
        if let Ok(direntry) = entry {
            let path = direntry.into_path();
            if let Ok(metadata) = std::fs::symlink_metadata(&path) {
                if !metadata.file_type().is_file() {
                    continue;
                }
            } else {
                continue;
            }

            context
                .spawn_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let permit = context.semaphore.clone().acquire_owned().await.unwrap();
            let sender = context.sender.clone();
            tokio::task::spawn_blocking(move || checksum_task(path, sender, permit));
        }
    }
}

// Pretty-prints the sorted `checksums` in a format much like what the
// `sha256sum` binary outputs.
//
// Note that the standard zakopane snapshot header is not added here.
fn pretty_format_checksums(path: std::path::PathBuf, checksums: Vec<ChecksumWithPath>) -> String {
    let mut buffer: Vec<String> = checksums
        .into_iter()
        .map(|e| {
            format!(
                "{}  ./{}",
                e.checksum,
                e.path.strip_prefix(&path).unwrap().to_str().unwrap()
            )
        })
        .collect();
    buffer.push(String::new());
    buffer.join("\n")
}

async fn checksum_impl(path: std::path::PathBuf) -> String {
    let (dispatcher_data, join_handle) = spawn_collector(path.clone()).await;
    spawn_checksum_tasks(dispatcher_data).await;
    let mut checksums: Vec<ChecksumWithPath> = join_handle.await.unwrap();
    checksums.sort_by(|a, b| a.path.cmp(&b.path));
    pretty_format_checksums(path, checksums)
}

pub fn checksum(path: std::path::PathBuf) -> String {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(checksum_impl(path))
}
