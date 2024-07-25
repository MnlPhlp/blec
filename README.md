# BLEC

A simple cross-platform **BLE** **C**lient library based on the awesome [btleplug library](https://github.com/deviceplug/btleplug)

This mainly wraps btleplug methods in way that is easier to use if you just need one active connection. It also deals with setup on the different platforms.
Most of that setup was taken from [flutter_btleplug](https://github.com/trobanga/flutter_btleplug).

My use case and the reason I created this is for simple communication in an Android app created in Flutter with business logic in Rust.

## Basic Usage

```rs
// initialize library
blec::init();

// scan for available devices
// either use channel to receive devices when discovered
let (tx,rx) = mpsc::channel(1);
blec::discover(tx,1000);
// or use discover_blocking() or discover_async() to receive Vec with devices after timeout
let devices = blec::discover_blocking(1000);

// get address of wanted device and call connect
// you also have to pass the wanted service and characteristics UUIDs
// a callback called on disconnect is optional
let adr = devices[0].address;
blec::connect(adr,<service UUID>, <charac UUIDs>, None / Some(disconnect callback));

// after this you can send/receive data to/from  the characteristics
// send
blec::send_data(<charac UUID>, <data>);
// read
let data = blec::recv_data(<charac UUD>);
// listen for notification
// the callback gets called with the notification data when a notification is received
blec::subscribe(<charac UUID>, <callblack>);

// at the end you can disconnect
blec::disconnect()
```

## Android Setup

In order to use this on android you need the Java part of [jni-utils-rs](https://github.com/deviceplug/jni-utils-rs) and [droidplug](https://github.com/deviceplug/btleplug/tree/master/src/droidplug/java).
Some more Information for Android setup is provided by the [btleplug library](https://github.com/deviceplug/btleplug/tree/master) this is based on.

## Example

See [esp_wifi_setup_app](https://github.com/MnlPhlp/esp_wifi_setup_app) for an example that uses this library in a flutter app to setup wifi credentials on a esp32.

The app part of that example can be used as basis for a flutter app using blec.