pub mod ble;
mod error;
mod handler;
mod setup;
pub use ble::*;
use btleplug::{
    api::{BDAddr, Peripheral as _},
    platform::Peripheral,
};
pub use error::BleError;
use futures::Future;
use setup::RUNTIME;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug, Clone, Ord, Eq)]
pub struct BleDevice {
    pub address: BleAddress,
    pub name: String,
    pub is_connected: bool,
}

impl PartialOrd for BleDevice {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.address.cmp(&other.address))
    }
}

impl PartialEq for BleDevice {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl BleDevice {
    async fn from_peripheral(peripheral: &Peripheral) -> Result<Self, BleError> {
        Ok(Self {
            address: peripheral.address().into(),
            name: peripheral
                .properties()
                .await?
                .unwrap_or_default()
                .local_name
                .ok_or(BleError::UnknownPeripheral(peripheral.id().to_string()))?,
            is_connected: peripheral.is_connected().await?,
        })
    }
}

/// spawn future on the internally initialized runtime
pub fn spawn<F>(f: F) -> Result<(), BleError>
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    let rt = RUNTIME.get().ok_or(BleError::RuntimeNotInitialized)?;
    rt.spawn(f);
    Ok(())
}

/// block on an async operation
pub fn block_on<F: Future>(f: F) -> Result<F::Output, BleError> {
    let rt = RUNTIME.get().ok_or(BleError::RuntimeNotInitialized)?;
    Ok(rt.block_on(f))
}

#[derive(Debug, Clone, Copy, Ord, Eq, PartialOrd, PartialEq, Hash, Default)]
pub struct BleAddress {
    pub address: [u8; 6],
}
impl PartialEq<BDAddr> for BleAddress {
    fn eq(&self, other: &BDAddr) -> bool {
        self.address.eq(&other.into_inner())
    }
}
impl From<BDAddr> for BleAddress {
    fn from(addr: BDAddr) -> Self {
        Self {
            address: addr.into_inner(),
        }
    }
}
impl Display for BleAddress {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let a = &self.address;
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            a[0], a[1], a[2], a[3], a[4], a[5]
        )
    }
}
impl BleAddress {
    // Parses a Bluetooth address with colons `:` as delimiters.
    pub fn from_str_delim(addr_str: &str) -> Result<Self, ParseBleAddressError> {
        match BDAddr::from_str_delim(addr_str) {
            Ok(addr) => Ok(addr.into()),
            Err(_) => Err(ParseBleAddressError),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct ParseBleAddressError;
