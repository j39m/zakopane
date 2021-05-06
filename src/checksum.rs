// Implements the `zakopane checksum` subcommand.

use std::convert::TryInto;
use std::io::Read;
use std::io::Write;

use crate::structs::ChecksumCliOptions;
use crate::structs::ZakopaneError;

const READ_SIZE: usize = 1 << 20;

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
    // Rate-limiter for spawned checksum tasks.
    semaphore: std::sync::Arc<tokio::sync::Semaphore>,

    cli_options: ChecksumCliOptions,

    // Counts total number of spawned checksum tasks.
    spawn_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,

    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
}

impl ChecksumTaskDispatcherData {
    pub fn new(
        cli_options: ChecksumCliOptions,
        spawn_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        sender: tokio::sync::mpsc::Sender<ChecksumResult>,
    ) -> Self {
        Self {
            semaphore: std::sync::Arc::new(tokio::sync::Semaphore::new(cli_options.max_tasks)),
            cli_options,
            spawn_counter,
            sender,
        }
    }
}

struct FileDetails {
    is_file: bool,
    is_big: bool,
}

fn get_file_details(path: &std::path::PathBuf, big_file_bytes: Option<u64>) -> FileDetails {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => {
            return FileDetails {
                is_file: false,
                is_big: false,
            }
        }
    };

    FileDetails {
        is_file: metadata.file_type().is_file(),
        is_big: if let Some(val) = big_file_bytes {
            metadata.len() > val
        } else {
            false
        },
    }
}

async fn get_semaphore_permit(
    path: &std::path::PathBuf,
    context: &ChecksumTaskDispatcherData,
) -> Option<tokio::sync::OwnedSemaphorePermit> {
    let file_details = get_file_details(&path, context.cli_options.big_file_bytes);
    if !file_details.is_file {
        return None;
    }

    // If the file is "big" (arbitrarily defined by user), attempt to
    // seize all permits to force us momentarily to work with a bit less
    // I/O contention.
    let permit = if file_details.is_big {
        context
            .semaphore
            .clone()
            .acquire_many_owned(context.cli_options.max_tasks.try_into().unwrap())
            .await
            .unwrap()
    } else {
        context.semaphore.clone().acquire_owned().await.unwrap()
    };
    Some(permit)
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
            break;
        }
    }
    if errors != 0 {
        eprintln!("WARNING: {} errors collected", errors);
    }
    sums
}

// Spawns the collector task that listens for checksum results
// provided by the checksum tasks.
async fn spawn_collector(
    options: ChecksumCliOptions,
) -> (ChecksumTaskDispatcherData, ChecksumTaskJoinHandle) {
    let (sender, receiver) = tokio::sync::mpsc::channel(options.max_tasks);
    let spawn_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let join_handle = tokio::task::spawn(collector_task(receiver, spawn_counter.clone()));
    (
        ChecksumTaskDispatcherData::new(options, spawn_counter, sender),
        join_handle,
    )
}

async fn spawn_checksum_tasks(context: ChecksumTaskDispatcherData) {
    let walk_iter = walkdir::WalkDir::new(&context.cli_options.path).into_iter();
    // The `filter_entry()` call is crafted s.t.
    // *    we skip and don't descend into hidden directories
    // *    unless the hidden directory is the target directory, because
    //      the target directory is always the first yielded value from
    //      the `WalkDir`.
    for entry in walk_iter.filter_entry(|e| {
        e.path() == &context.cli_options.path
            || !e
                .file_name()
                .to_str()
                .map(|path| path.starts_with("."))
                .unwrap_or(false)
    }) {
        let path = match entry {
            Ok(d) => d,
            Err(_) => continue,
        }
        .into_path();
        let permit = match get_semaphore_permit(&path, &context).await {
            Some(p) => p,
            None => continue,
        };

        context
            .spawn_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let sender = context.sender.clone();
        tokio::task::spawn_blocking(move || checksum_task(path, sender, permit));
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

async fn checksum_impl(options: ChecksumCliOptions) -> String {
    let path = options.path.clone();
    let (dispatcher_data, join_handle) = spawn_collector(options).await;
    spawn_checksum_tasks(dispatcher_data).await;
    let mut checksums: Vec<ChecksumWithPath> = join_handle.await.unwrap();
    checksums.sort_by(|a, b| a.path.cmp(&b.path));
    pretty_format_checksums(path, checksums)
}

pub fn checksum(options: ChecksumCliOptions) -> String {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(checksum_impl(options))
}
