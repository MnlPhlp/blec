use tokio::sync::mpsc::error::SendError;

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BleError {
    #[error("Btleplug error: {0}")]
    Btleplug(#[from] btleplug::Error),

    #[error("JNI {0}")]
    Jni(#[from] jni::errors::Error),

    #[error("Call init() first.")]
    RuntimeNotInitialized,

    #[allow(dead_code)]
    #[error("Cannot initialize CLASS_LOADER")]
    ClassLoader,

    #[allow(dead_code)]
    #[error("Cannot initialize RUNTIME")]
    Runtime,

    #[allow(dead_code)]
    #[error("Java vm not initialized")]
    JavaVM,

    #[error("There is no peripheral with id: {0}")]
    UnknownPeripheral(String),

    #[error("Characteristic with uuid {0:?} not found")]
    CharacNotFound(Uuid),

    #[error("Characteristic {0} not available")]
    CharacNotAvailable(String),

    #[error("No device connected")]
    NoDeviceConnected,

    #[error("Service not found")]
    ServiceNotFound,

    #[error("Device is already connected.")]
    AlreadyConnected,

    #[error("Handler not initialized")]
    HandlerNotInitialized,

    #[error("Handler already initialized")]
    HandlerAlreadyInitialized,

    #[error("received wrong data")]
    WrongData,

    #[error("could not send devices: {0}")]
    SendingDevices(SendError<Vec<crate::BleDevice>>),

    #[error("could not join fuure: {0}")]
    JoinError(tokio::task::JoinError),
}
