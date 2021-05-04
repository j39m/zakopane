// Implements the `zakopane checksum` subcommand.

use crate::structs::ZakopaneError;

const MAX_TASKS: usize = 8;

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

// Separated from `checksum_task()` to ensure the `add_permits()` call is
// always hit.
fn checksum_task_impl(path: std::path::PathBuf, sender: tokio::sync::mpsc::Sender<ChecksumResult>) {
    let contents = match std::fs::read(&path).map_err(ZakopaneError::Io) {
        Ok(contents) => contents,
        Err(e) => return checksum_task_send_result(Err(e), sender),
    };
    let checksum = crypto_hash::hex_digest(crypto_hash::Algorithm::SHA256, &contents);
    checksum_task_send_result(Ok(ChecksumWithPath::new(checksum, path)), sender);
}

// Represents a spawned checksum task.
fn checksum_task(
    path: std::path::PathBuf,
    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
    semaphore_clone: std::sync::Arc<tokio::sync::Semaphore>,
) {
    checksum_task_impl(path, sender);

    // See comment in `ChecksumTaskManager::spawn_task()`.
    semaphore_clone.add_permits(1);
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
    let spawn_counter_clone = spawn_counter.clone();
    let join_handle =
        tokio::task::spawn(async move { collector_task(receiver, spawn_counter_clone).await });
    (
        ChecksumTaskDispatcherData::new(path, spawn_counter, sender),
        join_handle,
    )
}

async fn spawn_checksum_tasks(context: ChecksumTaskDispatcherData) {
    let walk_iter = walkdir::WalkDir::new(context.path).into_iter();
    for entry in walk_iter.filter_entry(|e| {
        !e.file_name()
            .to_str()
            .map(|path| path.starts_with(".")) // XXX(j39m)
            .unwrap_or(false)
    }) {
        if let Ok(direntry) = entry {
            let path = direntry.into_path();
            if !path.is_file() {
                continue;
            }

            context
                .spawn_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let permit = context.semaphore.acquire().await.unwrap();

            // Icky workaround for not being able to pass `SemaphorePermit`
            // directly into a spawned task. Note that new permits are added
            // later in the checksum task.
            //
            // See also: https://github.com/tokio-rs/tokio/issues/1998
            permit.forget();

            let sender = context.sender.clone();
            let semaphore_clone = context.semaphore.clone();
            tokio::task::spawn_blocking(move || checksum_task(path, sender, semaphore_clone))
                .await
                .unwrap();
        }
    }
}

// Pretty-prints the sorted `checksums` in a format much like what the
// `sha256sum` binary outputs.
//
// Note that the standard zakopane snapshot header is not added here.
fn pretty_format_checksums(path: std::path::PathBuf, checksums: Vec<ChecksumWithPath>) -> String {
    let mut buffer: Vec<String> = Vec::new();
    for digest_line in checksums {
        buffer.push(format!(
            "{}  ./{}",
            digest_line.checksum,
            digest_line
                .path
                .strip_prefix(&path)
                .unwrap()
                .to_str()
                .unwrap()
        ));
    }
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
