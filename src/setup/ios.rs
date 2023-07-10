// taken from https://github.com/trobanga/flutter_btleplug/blob/b092ef415b36e60f4bb6df0ca261efdedaaa4a7e/packages/btleplug/native/src/ble/setup/ios.rs

use crate::ble::Error;

pub static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn create_runtime() -> Result<(), Error> {
    let runtime = {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .on_thread_start(|| {})
            .build()
            .unwrap()
    };
    RUNTIME.set(runtime).map_err(|_| Error::Runtime)?;
    Ok(())
}
