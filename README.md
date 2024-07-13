# sma-proto

A Rust crate that provides an implementation of the SMA Speedwire protocol
for building custom applications that communicate with SMA energy-meters or
inverters.

## Crate Features and Goals

* [ ] High level client for easy integration into applications.
* [ ] Implement energy-meter protocol.
* [ ] Implement inverter data readout protocol.
* [x] Optional **`no_std`** support for embedded devices.
* [ ] Verify messages during de-serialization.
* [ ] Being efficient if possible.
* [x] Simple Wireshark dissector for debugging on network layer.
      (Lua script is located in the repository root.)

## Rust Feature Flags
* **`std`** (default) — Remove this feature to make the library
  `no_std` compatible.
* **`client`** — Enables a tokio based high level client.

## Specification

* Energymeter protocol: [link](https://cdn.sma.de/fileadmin/content/www.developer.sma.de/docs/EMETER-Protokoll-TI-en-10.pdf)
* Inverter protocol: reverse-engineered

## License

**sma-proto** is licensed under the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or (at your
option) any later version.

## Disclaimer

This project is not affiliated with SMA.
All trademarks belong to their respective owners.
