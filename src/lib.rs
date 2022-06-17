use std::sync::Arc;
use open_entry_bindings as entry;

use entry::{runtime::Runtime, event::EventType, executor::{Lock, ExecutorBehaviour}, virtual_thread::VThread};

// This event fires when the first time Runtime has been instantiated.
#[no_mangle]
pub extern "Rust" fn vm_init(runtime: Arc<Runtime>, id: u32) {
    // You must call open_entry_bindings::init(...) to use your any methods in OpenEntry.
    // This should be placed at the first line of the code
    entry::init(&runtime, id);

    // Do anything you want
    println!("Extension initialized with ID {id}");

    // You can set/get your extension-specific data
    // There's unique extension_data for each vthreads.
    {
        let runtime = runtime.clone();

        runtime.clone().tokio_rt.spawn(async move {
            // SATETY: We use i32 in all the cases, So transmute never happens.
            unsafe {
                runtime.extension_data.set(12345).await;

                let value = runtime.extension_data.get::<i32>().await.unwrap();

                assert_eq!(*value, 12345);
            }
        });
    }
}

#[no_mangle]
pub extern "Rust" fn vm_event_recv(_runtime: Arc<Runtime>, event: EventType) {
    // Events received from runtime. or calls from other extensions
    println!("Event Recv: {event:?}");
}

#[no_mangle]
pub fn vm_interrupt(_thread: VThread, lock: Lock, id: u32, drop: bool) -> (Lock, ExecutorBehaviour) {
    // You can execute your custom block commands here
    println!("VM Called Our Interrupt {id}");

    // We're not going to shutdown the executor, So tell them to just execute next instruction
    (handle_lock(drop, lock), ExecutorBehaviour::None)
}

#[no_mangle]
pub fn vm_function_call(_thread: VThread, lock: Lock, id: u32, drop: bool) -> (Lock, ExecutorBehaviour) {
    // You can execute your custom block commands here
    // Difference with vm_interrupt is vm_interrupt only works with registers.
    // But in vm_function_call, you can get more arguments from stack.
    // Also you have to restore all the registers from here.
    println!("VM Called Our Function {id}");

    // We're not going to shutdown the executor, So tell them to just execute next instruction
    (handle_lock(drop, lock), ExecutorBehaviour::None)
}

// When VM uses Block-scoped locking executor, You have to handle locks.
// We unlocks the mutex when executor tells us to unlock it.
// Also we can unlock mutex earlier than this code if needed.
// ex) tokio::time::sleep in your interrupt/function call
#[inline(always)]
fn handle_lock(should_drop: bool, lock: Lock) -> Lock {
    if should_drop {
        drop(lock);

        None
    } else {
        lock
    }
}