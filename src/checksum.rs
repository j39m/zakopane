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

// Thread-shared fields are wrapped in `Arc` to silence build complaints
// about references (moved into spawned threads) outliving this struct.
struct ChecksumTaskManager {
    // Non-associative container enumerating paths and checksums.
    sums: std::vec::Vec<ChecksumWithPath>,

    // Rate-limiter for spawned checksum tasks.
    semaphore: std::sync::Arc<tokio::sync::Semaphore>,

    results_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,

    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
    receiver: tokio::sync::mpsc::Receiver<ChecksumResult>,
}

// Sends a checksum task result downstream to the collector task.
async fn send_checksum_result(
    result: ChecksumResult,
    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
) {
    if let Err(_) = sender.send(result).await {
        eprintln!("BUG: Sender::send() failed");
    }
}

// Separated from `do_checksum()` to ensure the `add_permits()` call is
// always hit.
async fn do_checksum_impl(
    path: std::path::PathBuf,
    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
) {
    let contents = match crate::helpers::ingest_file(path.to_str().unwrap()) {
        Ok(contents) => contents,
        Err(e) => return send_checksum_result(Err(e), sender).await,
    };
    let checksum = crypto_hash::hex_digest(crypto_hash::Algorithm::SHA256, contents.as_ref());
    send_checksum_result(
        Ok(ChecksumWithPath::new(
            checksum,
            String::from(path.to_str().unwrap()),
        )),
        sender,
    )
    .await;
}

async fn do_checksum(
    path: std::path::PathBuf,
    sender: tokio::sync::mpsc::Sender<ChecksumResult>,
    semaphore_clone: std::sync::Arc<tokio::sync::Semaphore>,
) {
    do_checksum_impl(path, sender).await;

    // See comment in `ChecksumTaskManager::spawn_task()`.
    semaphore_clone.add_permits(1);
}

impl ChecksumTaskManager {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(MAX_TASKS);
        Self {
            sums: Default::default(),
            semaphore: std::sync::Arc::new(tokio::sync::Semaphore::new(MAX_TASKS)),
            results_counter: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            sender: sender,
            receiver: receiver,
        }
    }

    async fn spawn_task(&mut self, path: std::path::PathBuf) {
        self.results_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let permit = self.semaphore.acquire().await.unwrap();

        // Icky workaround for not being able to pass `SemaphorePermit`
        // directly into a spawned task. Note that new permits are added
        // later in the checksum task.
        //
        // See also: https://github.com/tokio-rs/tokio/issues/1998
        permit.forget();

        let sender = self.sender.clone();
        let semaphore_clone = self.semaphore.clone();
        tokio::task::spawn_blocking(move || do_checksum(path, sender, semaphore_clone));
    }

    async fn collect_checksum_results(&mut self) {
        loop {
            if let Some(result) = self.receiver.recv().await {
                match result {
                    Ok(digest_with_path) => self.sums.push(digest_with_path),
                    Err(e) => panic!("BUG from checksum task: {:?}", e),
                }
            }
            if self.sums.len()
                == self
                    .results_counter
                    .load(std::sync::atomic::Ordering::SeqCst)
            {
                return;
            }
        }
    }
}

async fn checksum_impl() -> Result<String, ZakopaneError> {
    // *    Walk the target path.
    // *    If we can, then spawn a task, providing the current path
    //      in the walk and a cloned transmitter.
    // *    Call `poll_recv()` on the receiver repeatedly, updating
    //      our checksum map and `join()`ing the completed tasks.
    todo!()
}

#[allow(dead_code)]
pub fn checksum() -> Result<String, ZakopaneError> {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(checksum_impl())
}
