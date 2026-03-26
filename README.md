# fs-bus

Async, topic-based event bus for FreeSynergy with buffered delivery, retry, and
role-based subscriptions.

## Build

```sh
cargo build --release
cargo test
```

## Architecture

```text
Producer → BusMessage → MessageBus → Router → [TopicHandler, BusBridge, …]
                                   ↓
                         SubscriptionManager (role → topic filter)
                                   ↓
                         StandingOrdersEngine (persistent triggers)
                                   ↓
                         RoutingConfig (TOML rules → delivery + storage)
```

- `MessageBus` — core bus: publish, subscribe, routing
- `Router` / `RoutingConfig` — topic dispatch with TOML-configurable rules
- `Subscription` / `SubscriptionManager` — role-based topic subscriptions
- `StandingOrder` / `StandingOrdersEngine` — persistent trigger rules
- `EventBuffer` / `RetryPolicy` — buffered delivery with exponential backoff
- `BusMessage` / `Event` / `EventMeta` — message envelope types
- `TopicHandler` / `topic_matches` — glob-pattern topic matching
- `Transform` / `ChainTransform` — message transformation pipeline
- `BusBridge` (feature: `bridge`) — HTTP bus-to-bus forwarding
- `TeraTransform` (feature: `tera-transform`) — Tera template transforms

## Features

- `bridge` — enables BusBridge for HTTP forwarding between bus instances
- `tera-transform` — enables TeraTransform for template-based message transforms
