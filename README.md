# Wildly Distributed Wasm Demo

An absurd demo

## Features

- [x] Data persistence
- [x] Data replication across any number of clouds and edges
- [x] Automatically fetch data from the fastest responder
- [x] Immediate failover during cloud outages

## TODO

- [ ] Make redis KV take config JSON too as input so I can configure each cloud redis to talk to their respective store
- [ ] Implement the remaining operations on the TODO app
- [ ] Test to ensure leaf nodes work properly with the whole local invocation thing

## What does this not have?

- [ ] Error handling on distributed sets
- [ ] Support for non-idempotent operations (updates, increments, etc)
