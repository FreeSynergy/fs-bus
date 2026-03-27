// F3 Integration test: fs-bus ↔ fs-inventory ↔ fs-registry
//
// Verifies that:
//   1. fs-inventory subscribes to installer::* bus messages
//      → installer.package.installed → resource appears in inventory
//   2. fs-registry subscribes to service::* bus messages
//      → service.started → service appears in registry
//   3. Full chain: service starts → bus fires → registry knows it

use std::sync::Arc;

use fs_bus::{BusMessage, Event, MessageBus};
use fs_inventory::{Inventory, InventoryBusHandler, PackageInstalledPayload};
use fs_registry::{Registry, RegistryBusHandler, ServiceStartedPayload};

// ── F3.1: installer.package.installed → fs-inventory ─────────────────────────

#[tokio::test]
async fn installer_event_recorded_in_inventory() {
    let inv = Arc::new(Inventory::open(":memory:").await.expect("open inventory"));
    let handler = Arc::new(InventoryBusHandler::new(Arc::clone(&inv)));

    let mut bus = MessageBus::new();
    bus.add_handler(handler);

    let payload = PackageInstalledPayload {
        id: "kanidm".into(),
        version: "1.4.2".into(),
        resource_type: fs_types::ResourceType::App,
        config_path: String::new(),
        data_path: String::new(),
    };

    let event =
        Event::new("installer.package.installed", "fs-store", payload).expect("build event");
    let result = bus.publish(BusMessage::fire(event)).await;
    assert!(
        !result.has_errors(),
        "handler returned errors: {:?}",
        result.errors()
    );

    let resource = inv
        .resource("kanidm")
        .await
        .expect("query failed")
        .expect("kanidm not in inventory");

    assert_eq!(resource.id, "kanidm");
    assert_eq!(resource.version, "1.4.2");
}

// ── F3.2: service.started → fs-registry ──────────────────────────────────────

#[tokio::test]
async fn service_started_registered_in_registry() {
    let registry = Arc::new(Registry::open(":memory:").await.expect("open registry"));
    let handler = Arc::new(RegistryBusHandler::new(Arc::clone(&registry)));

    let mut bus = MessageBus::new();
    bus.add_handler(handler);

    let payload = ServiceStartedPayload {
        service_id: "kanidm".into(),
        capability: "iam".into(),
        endpoint: "http://kanidm:8443".into(),
    };

    let event = Event::new("service.started", "kanidm", payload).expect("build event");
    let result = bus.publish(BusMessage::fire(event)).await;
    assert!(
        !result.has_errors(),
        "handler returned errors: {:?}",
        result.errors()
    );

    let entries = registry
        .by_capability("iam")
        .await
        .expect("registry query failed");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].service_id, "kanidm");
    assert_eq!(entries[0].endpoint, "http://kanidm:8443");
}

// ── F3.3: service.stopped → deregistered ─────────────────────────────────────

#[tokio::test]
async fn service_stopped_deregistered_from_registry() {
    let registry = Arc::new(Registry::open(":memory:").await.expect("open registry"));
    let handler = Arc::new(RegistryBusHandler::new(Arc::clone(&registry)));

    let mut bus = MessageBus::new();
    bus.add_handler(handler);

    // Register first
    let start = ServiceStartedPayload {
        service_id: "stalwart".into(),
        capability: "mail".into(),
        endpoint: "http://stalwart:25".into(),
    };
    bus.publish(BusMessage::fire(
        Event::new("service.started", "stalwart", start).unwrap(),
    ))
    .await;

    assert_eq!(registry.by_capability("mail").await.unwrap().len(), 1);

    // Now stop it
    let stop = fs_registry::ServiceStoppedPayload {
        service_id: "stalwart".into(),
    };
    let result = bus
        .publish(BusMessage::fire(
            Event::new("service.stopped", "stalwart", stop).unwrap(),
        ))
        .await;
    assert!(!result.has_errors());

    assert_eq!(
        registry.by_capability("mail").await.unwrap().len(),
        0,
        "stalwart should have been deregistered"
    );
}

// ── F3.4: full chain — program starts → bus → registry ───────────────────────

#[tokio::test]
async fn full_chain_start_registers_in_bus_and_registry() {
    let inv = Arc::new(Inventory::open(":memory:").await.expect("inventory"));
    let reg = Arc::new(Registry::open(":memory:").await.expect("registry"));

    let mut bus = MessageBus::new();
    bus.add_handler(Arc::new(InventoryBusHandler::new(Arc::clone(&inv))));
    bus.add_handler(Arc::new(RegistryBusHandler::new(Arc::clone(&reg))));

    // Program installs
    bus.publish(BusMessage::fire(
        Event::new(
            "installer.package.installed",
            "fs-store",
            PackageInstalledPayload {
                id: "forgejo".into(),
                version: "9.0.0".into(),
                resource_type: fs_types::ResourceType::Container,
                config_path: String::new(),
                data_path: String::new(),
            },
        )
        .unwrap(),
    ))
    .await;

    // Program registers capability
    bus.publish(BusMessage::fire(
        Event::new(
            "service.started",
            "forgejo",
            ServiceStartedPayload {
                service_id: "forgejo".into(),
                capability: "git".into(),
                endpoint: "http://forgejo:3000".into(),
            },
        )
        .unwrap(),
    ))
    .await;

    // Verify both sides
    assert!(
        inv.resource("forgejo").await.unwrap().is_some(),
        "forgejo not in inventory"
    );
    let git_services = reg.by_capability("git").await.unwrap();
    assert_eq!(git_services.len(), 1, "forgejo not registered for git");
    assert_eq!(git_services[0].service_id, "forgejo");
}
