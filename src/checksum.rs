// Implements the `zakopane checksum` subcommand.

use crate::structs::ZakopaneError;

const MAX_TASKS: i32 = 8;

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
