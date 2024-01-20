use crate::setup::{self, RUNTIME};
use crate::{block_on, spawn, BleAddress, BleDevice};
use crate::{handler::BleHandler, BleError};
use futures::{Future, StreamExt};
use once_cell::sync::OnceCell;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

static HANDLER: OnceCell<Mutex<BleHandler>> = OnceCell::new();

/// The init() function must be called before anything else.
/// At the moment the developer has to make sure it is only called once.
pub fn init() -> Result<(), BleError> {
    // crate the runtime include architecture specific differences
    setup::create_runtime()?;
    let rt = RUNTIME.get().ok_or(BleError::RuntimeNotInitialized)?;
    HANDLER
        .set(Mutex::new(rt.block_on(BleHandler::new())?))
        .map_err(|_| BleError::HandlerAlreadyInitialized)?;
    // start a loop running in the background for handling ble events
    rt.spawn(event_loop());
    Ok(())
}

async fn event_loop() -> Result<(), BleError> {
    let handler = get_handler();
    let mut events = handler.lock().await.get_event_stream().await?;
    while let Some(event) = events.next().await {
        handler.lock().await.handle_event(event).await?;
    }
    Ok(())
}

fn get_handler() -> &'static Mutex<BleHandler> {
    let handler = HANDLER
        .get()
        .ok_or(BleError::HandlerNotInitialized)
        .unwrap();
    handler
}

async fn run_on_runtime<F, O>(f: F) -> Result<O, BleError>
where
    F: Future<Output = Result<O, BleError>> + Send + 'static,
    O: Send + 'static,
{
    let rt = RUNTIME.get().ok_or(BleError::RuntimeNotInitialized)?;
    rt.spawn(f).await.map_err(|e| BleError::JoinError(e))?
}

pub async fn connect(
    addr: BleAddress,
    service: Uuid,
    characs: Vec<Uuid>,
    on_disconnect: Option<impl Fn() + Send + 'static>,
) -> Result<(), BleError> {
    run_on_runtime(async move {
        let mut handler = get_handler().lock().await;
        handler.connect(addr, service, characs, on_disconnect).await
    })
    .await
}

pub async fn disconnect() -> Result<(), BleError> {
    run_on_runtime(async move {
        let mut handler = get_handler().lock().await;
        handler.disconnect().await
    })
    .await
}

pub fn discover(sink: mpsc::Sender<Vec<BleDevice>>, timeout: u64) -> Result<(), BleError> {
    spawn(async move {
        let mut handler = get_handler().lock().await;
        handler.discover(Some(sink), timeout).await
    })
}

pub async fn discover_async(timeout: u64) -> Result<Vec<BleDevice>, BleError> {
    let discovered = run_on_runtime(async move {
        let mut handler = get_handler().lock().await;
        handler.discover(None, timeout).await
    })
    .await?;
    Ok(discovered)
}

pub fn discover_blocking(timeout: u64) -> Result<Vec<BleDevice>, BleError> {
    let discovered = block_on(async move {
        let mut handler = get_handler().lock().await;
        handler.discover(None, timeout).await
    })??;
    Ok(discovered)
}

pub async fn send_data(charac: Uuid, data: Vec<u8>) -> Result<(), BleError> {
    run_on_runtime(async move {
        let mut handler = get_handler().lock().await;
        handler.send_data(charac, &data).await
    })
    .await
}

pub async fn recv_data(charac: Uuid) -> Result<Vec<u8>, BleError> {
    run_on_runtime(async move {
        let mut handler = get_handler().lock().await;
        handler.recv_data(charac).await
    })
    .await
}

pub async fn is_connected() -> Result<bool, BleError> {
    run_on_runtime(async move {
        let handler = get_handler().lock().await;
        handler.check_connected().await
    })
    .await
}

pub async fn connected_device() -> Result<BleDevice, BleError> {
    run_on_runtime(async move {
        let handler = get_handler().lock().await;
        handler.connected_device().await
    })
    .await
}

pub async fn subscribe(
    charac: Uuid,
    callback: impl Fn(&[u8]) + Send + Sync + 'static,
) -> Result<(), BleError> {
    run_on_runtime(async move {
        let mut handler = get_handler().lock().await;
        handler.subscribe(charac, callback).await
    })
    .await
}
