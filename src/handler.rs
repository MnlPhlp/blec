use crate::setup::RUNTIME;
use crate::{BleDevice, BleError};
use btleplug::api::CentralEvent;
use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::{Stream, StreamExt};
use log::debug;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::task::AbortHandle;
use tokio::time::sleep;
use uuid::Uuid;

struct Listener {
    uuid: Uuid,
    callback: Arc<dyn Fn(&[u8]) + Send + Sync>,
}

pub struct BleHandler {
    connected: Option<Arc<Peripheral>>,
    characs: Vec<Characteristic>,
    devices: HashMap<String, Peripheral>,
    adapter: Adapter,
    notify_abort: Option<AbortHandle>,
    notify_listeners: Arc<Mutex<Vec<Listener>>>,
    on_disconnect: Option<Mutex<Box<dyn Fn() + Send>>>,
}

impl BleHandler {
    pub async fn new() -> Result<Self, BleError> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().expect("cannot fail");
        Ok(Self {
            devices: HashMap::new(),
            characs: vec![],
            connected: None,
            adapter: central,
            notify_abort: None,
            notify_listeners: Arc::new(Mutex::new(vec![])),
            on_disconnect: None,
        })
    }

    pub async fn connect(
        &mut self,
        address: String,
        service: Uuid,
        characs: Vec<Uuid>,
        on_disconnect: Option<impl Fn() + Send + 'static>,
    ) -> Result<(), BleError> {
        if self.devices.len() == 0 {
            self.discover(None, 1000).await?;
        }
        // connect to the given address
        self.connect_device(address).await?;
        // discover service/characteristics
        self.connect_service(service, &characs).await?;
        // set callback to run on disconnect
        if let Some(cb) = on_disconnect {
            self.on_disconnect = Some(Mutex::new(Box::new(cb)));
        }
        // start background task for notifications
        let rt = RUNTIME.get().ok_or(BleError::RuntimeNotInitialized)?;
        rt.spawn(listen_notify(
            self.get_device().await?,
            self.notify_listeners.clone(),
        ));
        Ok(())
    }

    async fn connect_service(&mut self, service: Uuid, characs: &[Uuid]) -> Result<(), BleError> {
        let device = self.get_device().await?;
        device.discover_services().await?;
        let services = device.services();
        let s = services
            .iter()
            .find(|s| s.uuid == service)
            .ok_or(BleError::ServiceNotFound)?;
        for c in &s.characteristics {
            if characs.contains(&c.uuid) {
                self.characs.push(c.clone());
            }
        }
        Ok(())
    }

    async fn connect_device(&mut self, address: String) -> Result<(), BleError> {
        if let Some(dev) = self.connected.clone() {
            if dev.address().to_string() == address {
                return Err(BleError::AlreadyConnected.into());
            }
        }
        let device = self
            .devices
            .get(&address)
            .ok_or(BleError::UnknownPeripheral(address.clone()))?;
        if !device.is_connected().await? {
            debug!("Connecting to device");
            device.connect().await?;
            debug!("Connecting done");
        }
        self.connected = Some(Arc::new(device.clone()));
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), BleError> {
        if let Some(notify) = self.notify_abort.as_ref() {
            notify.abort();
            self.notify_abort = None;
        }
        if let Some(dev) = self.connected.as_mut() {
            if let Ok(true) = dev.is_connected().await {
                dev.disconnect().await?;
            }
            self.connected = None;
        }
        if let Some(on_disconnect) = &self.on_disconnect {
            let callback = on_disconnect.lock().await;
            callbac);
        }
        self.characs.clear();
        self.devices.clear();
        Ok(())
    }

    /// Scans for [timeout] milliseconds and periodically sends discovered devices
    /// Also returns vector with all devices after timeout
    pub async fn discover(
        &mut self,
        tx: Option<mpsc::Sender<Vec<BleDevice>>>,
        timeout: u64,
    ) -> Result<Vec<BleDevice>, BleError> {
        self.adapter
            .start_scan(ScanFilter {
                // services: vec![*SERVICE_UUID],
                services: vec![],
            })
            .await?;
        self.devices.clear();
        let loops = (timeout as f64 / 200.0).round() as u64;
        let mut devices = vec![];
        for _ in 0..loops {
            sleep(Duration::from_millis(200)).await;
            let discovered = self.adapter.peripherals().await?;
            devices = self.add_devices(discovered).await;
            if !devices.is_empty() {
                if let Some(tx) = &tx {
                    tx.send(devices.clone())
                        .await
                        .map_err(|e| BleError::SendingDevices(e))?;
                }
            }
        }
        self.adapter.stop_scan().await?;
        Ok(devices)
    }

    async fn add_devices(&mut self, discovered: Vec<Peripheral>) -> Vec<BleDevice> {
        let mut devices = vec![];
        for p in discovered {
            if let Ok(dev) = BleDevice::from_peripheral(&p).await {
                self.devices.insert(dev.address.clone(), p);
                devices.push(dev);
            }
        }
        devices.sort();
        devices
    }

    pub async fn send_data(&mut self, c: Uuid, data: &[u8]) -> Result<(), BleError> {
        let dev = self.get_device().await?;
        let charac = self.get_charac(c)?;
        dev.write(charac, &data, WriteType::WithoutResponse).await?;
        Ok(())
    }

    pub async fn recv_data(&mut self, c: Uuid) -> Result<Vec<u8>, BleError> {
        let dev = self.get_device().await?;
        let charac = self.get_charac(c)?;
        let data = dev.read(charac).await?;
        Ok(data)
    }

    fn get_charac(&self, uuid: Uuid) -> Result<&Characteristic, BleError> {
        let charac = self.characs.iter().find(|c| c.uuid == uuid);
        charac.ok_or(BleError::CharacNotAvailable(uuid.to_string()).into())
    }

    async fn get_device(&mut self) -> Result<Arc<Peripheral>, BleError> {
        let dev = self.connected.as_ref().ok_or(BleError::NoDeviceConnected)?;
        if !dev.is_connected().await? {
            self.disconnect().await?;
            return Err(BleError::NoDeviceConnected.into());
        } else {
            return Ok(dev.clone());
        }
    }

    pub async fn check_connected(&self) -> Result<bool, BleError> {
        let mut connected = false;
        if let Some(dev) = self.connected.as_ref() {
            connected = dev.is_connected().await?;
        }
        Ok(connected)
    }

    pub async fn subscribe(
        &mut self,
        c: Uuid,
        callback: impl Fn(&[u8]) + Send + Sync + 'static,
    ) -> Result<(), BleError> {
        let dev = self.get_device().await?;
        let charac = self.get_charac(c)?;
        dev.subscribe(charac).await?;
        self.notify_listeners.lock().await.push(Listener {
            uuid: charac.uuid,
            callback: Arc::new(callback),
        });
        Ok(())
    }

    pub(super) async fn get_event_stream(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = CentralEvent> + Send>>, BleError> {
        let events = self.adapter.events().await?;
        Ok(events)
    }

    pub async fn handle_event(&mut self, event: CentralEvent) -> Result<(), BleError> {
        // logi!("handling event {event:?}");
        match event {
            CentralEvent::DeviceDisconnected(_) => self.disconnect().await,
            _ => Ok(()),
        }
    }

    pub async fn connected_device(&self) -> Result<BleDevice, BleError> {
        let p = self.connected.as_ref().ok_or(BleError::NoDeviceConnected)?;
        let d = BleDevice::from_peripheral(&p).await?;
        Ok(d)
    }
}

async fn listen_notify(
    dev: Arc<Peripheral>,
    listeners: Arc<Mutex<Vec<Listener>>>,
) -> Result<(), BleError> {
    let mut stream = dev.notifications().await?;
    let rt = RUNTIME.get().ok_or(BleError::RuntimeNotInitialized)?;
    while let Some(data) = stream.next().await {
        for l in listeners.lock().await.iter() {
            if l.uuid == data.uuid {
                let data = data.value.clone();
                let cb = l.callback.clone();
                rt.spawn_blocking(move || cb(&data));
            }
        }
    }
    Ok(())
}
