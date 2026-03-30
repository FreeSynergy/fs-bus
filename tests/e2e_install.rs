//! F6 — End-to-end install chain test.
//!
//! Simulates the full flow:
//!   1. StoreReader loads catalog → find a known manager package → verify download URL
//!   2. `installer.package.installed` bus event → InventoryBusHandler → inventory entry
//!   3. `service.started` bus event → RegistryBusHandler → registry entry
//!   4. `Registry::endpoint_for_capability` resolves the service endpoint
//!
//! The test does NOT perform a real HTTP download — it verifies the URL template
//! is present in the catalog and simulates the result of a successful install.

use std::sync::Arc;

use fs_bus::{BusMessage, Event, MessageBus};
use fs_db::engine::DbConfig;
use fs_inventory::{Inventory, InventoryBusHandler, PackageInstalledPayload, ReleaseChannel};
use fs_registry::{Registry, RegistryBusHandler, ServiceStartedPayload};
use fs_store::release::Platform;
use fs_store::source::StoreSource;
use fs_store::StoreReader;
use fs_types::ResourceType;

/// Path to the local Store catalog (relative to this crate's root).
const STORE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Store");

// ── helpers ───────────────────────────────────────────────────────────────────

async fn setup_bus() -> (MessageBus, Arc<Inventory>, Arc<Registry>) {
    let inv = Arc::new(
        Inventory::open(DbConfig::sqlite(":memory:"))
            .await
            .expect("inventory"),
    );
    let reg = Arc::new(Registry::open(":memory:").await.expect("registry"));

    let mut bus = MessageBus::new();
    bus.add_handler(Arc::new(InventoryBusHandler::new(Arc::clone(&inv))));
    bus.add_handler(Arc::new(RegistryBusHandler::new(Arc::clone(&reg))));

    (bus, inv, reg)
}

// ── F6.1 — Catalog has a download URL ─────────────────────────────────────────

#[tokio::test]
async fn catalog_manager_has_linux_download_url() {
    let reader = StoreReader::new(StoreSource::Local(STORE_PATH.into()));
    let map = reader.load_all().await.expect("load store");

    let pkg = map
        .all()
        .find(|p| p.id() == "managers.container")
        .expect("managers.container in catalog");

    let release = pkg.latest_release().expect("has release");
    let url = release
        .distribution
        .url_for(&Platform::LinuxX86_64)
        .expect("linux-x86_64 URL present");

    assert!(
        url.contains("FreeSynergy/fs-managers"),
        "URL should point to fs-managers repo, got: {url}"
    );
    assert!(
        url.contains("{version}"),
        "URL should contain {{version}} placeholder, got: {url}"
    );
}

// ── F6.2 — Bus event records in inventory ─────────────────────────────────────

#[tokio::test]
async fn install_event_recorded_in_inventory() {
    let (bus, inv, _reg) = setup_bus().await;

    let payload = PackageInstalledPayload {
        id: "managers.container".into(),
        version: "0.1.0".into(),
        resource_type: ResourceType::App,
        config_path: String::new(),
        data_path: String::new(),
    };
    let event =
        Event::new("installer.package.installed", "fs-store-app", payload).expect("build event");
    let result = bus.publish(BusMessage::fire(event)).await;
    assert!(!result.has_errors(), "bus errors: {:?}", result.errors());

    let resource = inv
        .resource("managers.container")
        .await
        .expect("query ok")
        .expect("resource recorded");
    assert_eq!(resource.id, "managers.container");
    assert_eq!(resource.version, "0.1.0");
    assert_eq!(resource.channel, ReleaseChannel::Stable);
}

// ── F6.3 — Bus event registers service in registry ────────────────────────────

#[tokio::test]
async fn service_start_event_registered_in_registry() {
    let (bus, _inv, reg) = setup_bus().await;

    let payload = ServiceStartedPayload {
        service_id: "fs-manager-container".into(),
        capability: "container".into(),
        endpoint: "http://localhost:9100".into(),
    };
    let event =
        Event::new("service.started", "fs-manager-container", payload).expect("build event");
    let result = bus.publish(BusMessage::fire(event)).await;
    assert!(!result.has_errors(), "bus errors: {:?}", result.errors());

    let endpoint = reg
        .endpoint_for_capability("container")
        .await
        .expect("query ok");
    assert_eq!(
        endpoint,
        Some("http://localhost:9100".to_string()),
        "container manager endpoint should be resolvable"
    );
}

// ── F6.4 — Full chain: install + start + resolve ──────────────────────────────

#[tokio::test]
async fn full_install_chain() {
    let (bus, inv, reg) = setup_bus().await;

    // Step 1: verify catalog URL
    let reader = StoreReader::new(StoreSource::Local(STORE_PATH.into()));
    let map = reader.load_all().await.expect("load store");
    let pkg = map
        .all()
        .find(|p| p.id() == "managers.container")
        .expect("managers.container in catalog");
    let version = pkg.latest_version().expect("has version").to_owned();
    let url_template = pkg
        .latest_release()
        .and_then(|r| r.distribution.url_for(&Platform::LinuxX86_64))
        .expect("linux download URL")
        .replace("{version}", &version);
    assert!(
        url_template.contains(&version),
        "URL template should have version substituted: {url_template}"
    );

    // Step 2: simulate install result (after download + verification)
    let install_payload = PackageInstalledPayload {
        id: pkg.id().to_owned(),
        version: version.clone(),
        resource_type: ResourceType::App,
        config_path: String::new(),
        data_path: String::new(),
    };
    bus.publish(BusMessage::fire(
        Event::new(
            "installer.package.installed",
            "fs-store-app",
            install_payload,
        )
        .expect("build event"),
    ))
    .await;

    // Step 3: simulate service starting after install
    let start_payload = ServiceStartedPayload {
        service_id: "fs-manager-container".into(),
        capability: "container".into(),
        endpoint: "http://localhost:9100".into(),
    };
    bus.publish(BusMessage::fire(
        Event::new("service.started", "fs-manager-container", start_payload).expect("build event"),
    ))
    .await;

    // Step 4: verify full chain
    let resource = inv
        .resource(pkg.id())
        .await
        .expect("query ok")
        .expect("installed in inventory");
    assert_eq!(resource.version, version);

    let endpoint = reg
        .endpoint_for_capability("container")
        .await
        .expect("query ok");
    assert_eq!(endpoint, Some("http://localhost:9100".to_string()));
}
