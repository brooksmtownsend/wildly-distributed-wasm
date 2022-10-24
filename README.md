# Wildly Distributed Wasm Demo

An absurd demo showing off the benefits of distributed WebAssembly and wasmCloud. Soon after Cloud Native Wasm day, a similar application will be posted on the Cosmonic [Things to Build](https://cosmonic.com/docs/category/things-to-build/) documentation. This is a deliberately overengineered TODO app that has a couple of awesome features:

## Features

- [x] Data persistence
- [x] Data replication across any number of clouds and edges
- [x] Automatically fetch data from the fastest responder
- [x] Immediate failover during cloud outages

Logically, the structure of the application is flat and consists of three WebAssembly actors:

1. TODO API Gateway that receives HTTP requests over `wasmcloud:httpserver`
1. UI Actor that receives an actor-to-actor call and returns UI assets, no frontend deployment needed
1. Distributed KV actor that receives Messages on `wasmcloud:messaging` and stores TODOs in a keyvalue store using `wasmcloud:keyvalue`

![logical organization of application](./logic-arch.png)

However, the real deployment architecture is completely flexible and as-demoed runs across Cosmonic, AWS, Azure, GCP, and Oracle cloud:

![actual deployment architecture](./cloud-arch.png)

## Running this demo
To run this demonstration across clouds and edges, you'll want to use [Cosmonic](https://get.cosmonic.com). You can run this demo with wasmCloud locally by:
1. [Installing and running Redis](https://redis.io/docs/getting-started/)
1. [Installing wash](https://wasmcloud.dev/overview/installation/)
1. Run `wash up`
1. Run `make start` in this repository to deploy all the WebAssembly modules and capability providers, then visit [http://localhost:8080](http://localhost:8080) to access the TODO app


## TODO

- [ ] Implement the update operation on the TODO app
- [x] Add in config.jsons e.g. `{"url": "redis://elasticache_url.aws.com:6379"}`. Note, these are the real configuration files that were used for the demonstration, but the URLs have been changed and the clusters no longer exist.

## What does this not have?

- [ ] Error handling on distributed sets
- [ ] Data recovery and syncing
