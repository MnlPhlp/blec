pub mod ble;
mod error;
mod handler;
mod setup;
use btleplug::{api::Peripheral as _, platform::Peripheral};
pub use error::BleError;

#[derive(Debug, Clone, Ord, Eq)]
pub struct BleDevice {
    pub address: String,
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
            address: peripheral.id().to_string(),
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
