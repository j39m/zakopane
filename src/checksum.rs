// Implements the `zakopane checksum` subcommand.

use crate::structs::ZakopaneError;

const MAX_TASKS: usize = 8;

#[derive(Default)]
struct ChecksumTaskManager {
    // Non-associative container enumerating paths and checksums.
    sums: std::vec::Vec<(String, String)>,

    // Map of outstanding checksum tasks.
    // *    Keys: absolute paths being checksummed
    // *    Values: tokio JoinHandles for checksum tasks
    tasks: std::collections::HashMap<String, tokio::task::JoinHandle<String>>,
}

async fn do_checksum(path: std::path::PathBuf) -> Result<String, ZakopaneError> {
    todo!()
}

impl ChecksumTaskManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn spawn_task(&self, path: std::path::PathBuf) {
        assert!(
            self.tasks.len() < MAX_TASKS,
            "attempted to spawn too many tasks"
        );
        tokio::spawn(do_checksum(path));
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
fn checksum() -> Result<String, ZakopaneError> {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(checksum_impl())
}
