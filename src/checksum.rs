// Implements the `zakopane checksum` subcommand.

use crate::structs::ZakopaneError;

const MAX_TASKS: usize = 8;

struct ChecksumWithPath {
    checksum: String,
    path: String,
}

impl ChecksumWithPath {
    pub fn new(c: String, p: String) -> Self {
        Self {
            checksum: c,
            path: p,
        }
    }
}

type ChecksumResult = Result<ChecksumWithPath, ZakopaneError>;
type ChecksumTaskJoinHandle = tokio::task::JoinHandle<Vec<ChecksumWithPath>>;

struct ChecksumTaskDispatcherData {
    // Rate-limiter for spawned checksum tasks.
    semaphore: std::sync::Arc<tokio::sync::Semaphore>,

    // Counts total number of spawned checksum tasks.
    spawn_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,

    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
}

impl ChecksumTaskDispatcherData {
    pub fn new(
        spawn_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        sender: tokio::sync::mpsc::Sender<ChecksumResult>,
    ) -> Self {
        Self {
            semaphore: std::sync::Arc::new(tokio::sync::Semaphore::new(MAX_TASKS)),
            spawn_counter,
            sender,
        }
    }
}

// Sends a checksum task result downstream to the collector task.
async fn checksum_task_send_result(
    result: ChecksumResult,
    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
) {
    if let Err(_) = sender.send(result).await {
        eprintln!("BUG: Sender::send() failed");
    }
}

// Separated from `checksum_task()` to ensure the `add_permits()` call is
// always hit.
async fn checksum_task_impl(
    path: std::path::PathBuf,
    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
) {
    let contents = match crate::helpers::ingest_file(path.to_str().unwrap()) {
        Ok(contents) => contents,
        Err(e) => return checksum_task_send_result(Err(e), sender).await,
    };
    let checksum = crypto_hash::hex_digest(crypto_hash::Algorithm::SHA256, contents.as_ref());
    checksum_task_send_result(
        Ok(ChecksumWithPath::new(
            checksum,
            String::from(path.to_str().unwrap()),
        )),
        sender,
    )
    .await;
}

async fn checksum_task(
    path: std::path::PathBuf,
    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
    semaphore_clone: std::sync::Arc<tokio::sync::Semaphore>,
) {
    checksum_task_impl(path, sender).await;

    // See comment in `ChecksumTaskManager::spawn_task()`.
    semaphore_clone.add_permits(1);
}

async fn collector_task(
    mut receiver: tokio::sync::mpsc::Receiver<ChecksumResult>,
    spawn_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
) -> Vec<ChecksumWithPath> {
    let mut sums: Vec<ChecksumWithPath> = Vec::new();
    let mut errors: usize = 0;
    loop {
        if let Some(result) = receiver.recv().await {
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
async fn spawn_collector() -> (ChecksumTaskDispatcherData, ChecksumTaskJoinHandle) {
    let (sender, receiver) = tokio::sync::mpsc::channel(MAX_TASKS);
    let spawn_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let spawn_counter_clone = spawn_counter.clone();
    let join_handle =
        tokio::task::spawn(async move { collector_task(receiver, spawn_counter_clone).await });
    (
        ChecksumTaskDispatcherData::new(spawn_counter, sender),
        join_handle,
    )
}

async fn dispatch_checksum_tasks(context: ChecksumTaskDispatcherData) {
    // TODO(j39m): Fix the hardcoded path.
    let walk_iter = walkdir::WalkDir::new("/home/kalvin/.config").into_iter();
    for entry in walk_iter.filter_entry(|e| {
        !e.file_name()
            .to_str()
            .map(|path| path.starts_with("."))
            .unwrap_or(false)
    }) {
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
        tokio::task::spawn_blocking(move || {
            checksum_task(entry.unwrap().path().to_path_buf(), sender, semaphore_clone)
        });
    }
}

fn pretty_format_checksums(checksums: Vec<ChecksumWithPath>) -> String {
    let mut buffer: Vec<String> = Vec::new();
    for digest_line in checksums {
        buffer.push(format!("{}  ./{}", digest_line.checksum, digest_line.path));
    }
    buffer.join("\n")
}

async fn checksum_impl() -> String {
    let (dispatcher_data, join_handle) = spawn_collector().await;
    dispatch_checksum_tasks(dispatcher_data).await;
    let mut checksums: Vec<ChecksumWithPath> = join_handle.await.unwrap();
    checksums.sort_by(|a, b| a.path.cmp(&b.path));
    pretty_format_checksums(checksums)
}

pub fn checksum() -> String {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(checksum_impl())
}
