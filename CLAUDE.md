# CLAUDE.md – fs-bus

## What is this?

FreeSynergy Message Bus — async, topic-based event routing with buffered delivery,
exponential-backoff retry, role-based subscriptions, standing orders, configurable
routing rules, and optional bus-to-bus bridging.

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md
- After every feature: commit directly

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
- `Router` / `RoutingConfig` / `RoutingRule` — topic routing rules
- `Subscription` / `SubscriptionManager` — role-based topic subscriptions
- `StandingOrder` / `StandingOrdersEngine` — persistent trigger rules
- `EventBuffer` / `RetryPolicy` — buffered delivery with backoff
- `BusMessage` / `Event` / `EventMeta` — message envelope types
- `TopicHandler` / `topic_matches` — topic matching and handlers
- `Transform` / `ChainTransform` — message transformation pipeline
- `BusBridge` (feature: `bridge`) — HTTP bus-to-bus forwarding
- `TeraTransform` (feature: `tera-transform`) — Tera template transforms

## Dependencies

- **fs-libs** (`../fs-libs/`) — `fs-types` (StrLabel, etc.), `fs-template` (optional)

## Features

- `bridge` — enables BusBridge for HTTP forwarding between bus instances
- `tera-transform` — enables TeraTransform for template-based message transforms
