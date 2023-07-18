# BLEC

A simple cross-platform **BLE** **C**lient library based on the awesome [btleplug library](https://github.com/deviceplug/btleplug)"

This mainly wraps btleplug methods in way that is easier to use if you just need one active connection. It also deals with setup on the different platforms.
Most of that setup was taken from [flutter_btleplug](https://github.com/trobanga/flutter_btleplug).

My use case and the reason I created this is for simple communication in an Android app created in Flutter with business logic in Rust.
