// taken from https://github.com/trobanga/flutter_btleplug/blob/b092ef415b36e60f4bb6df0ca261efdedaaa4a7e/packages/btleplug/native/src/ble/setup.rs

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::*;

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::*;

use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;
pub static RUNTIME: OnceCell<Runtime> = once_cell::sync::OnceCell::new();

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn create_runtime() -> Result<(), super::BleError> {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("Thread {id}")
        })
        .build()
        .unwrap();
    RUNTIME.set(runtime).map_err(|_| super::BleError::Runtime)?;
    Ok(())
}
