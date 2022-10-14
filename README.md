# UniPager status MQTT bridge

[![CI](https://github.com/DanNixon/unipager-status-mqtt/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/DanNixon/unipager-status-mqtt/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/dannixon/unipager-status-mqtt/status.svg)](https://deps.rs/repo/github/dannixon/unipager-status-mqtt)

A tool to publish UniPager status via MQTT.

This operates by listening to the websocket API, parsing messages received and spitting them back out via MQTT.
Nothing special, but fulfils a very specific purpose.

## Configuration

See [the example](./examples/config.toml).

## Usage

`unipager-mqtt-bridge -c [config file]`.

## Messages

### Availability

`online` or `offline` depending on if the bridge is running or not.

### Timeslot

A number from 0 - 15 indicating the current time slot.

### Queue Length

A positive number indicating the number of messages in the transmission queue.

### Transmitting

`true` or `false` depending on if the transmitter is currently transmitting or not.

### New Message

A message for each new message to arrive in the queue:
```json
{
  "destination": 0,
  "text": ""
}
```
