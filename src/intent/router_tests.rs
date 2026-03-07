#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use super::router::route;
use crate::skills::types::{IntentConfidence, IntentMatch, SkillId};

fn fallback_intent() -> IntentMatch {
    IntentMatch {
        skill_id: SkillId::Doctor,
        confidence: IntentConfidence::Low,
        args: BTreeMap::new(),
    }
}

#[test]
fn routes_disk_pressure_matrix() {
    let cases = [
        "my disk is full",
        "i am out of space on the server",
        "storage usage is too high",
        "no space left on root",
    ];

    for request in cases {
        let intent = route(request).unwrap_or_else(fallback_intent);
        assert_eq!(intent.skill_id, SkillId::DiskFullTriage, "{request}");
    }
}

#[test]
fn routes_health_matrix() {
    let cases = [
        "why is my server slow",
        "show running services",
        "check system health",
        "list services",
    ];

    for request in cases {
        let intent = route(request).unwrap_or_else(fallback_intent);
        assert_eq!(intent.skill_id, SkillId::Doctor, "{request}");
    }
}

#[test]
fn routes_package_install_matrix() {
    let cases = [
        ("install nginx", "nginx"),
        ("install docker", "docker.io"),
        ("install tomcat", "tomcat10"),
        ("add package redis-server", "redis-server"),
    ];

    for (request, expected_package) in cases {
        let intent = route(request).unwrap_or_else(fallback_intent);
        assert_eq!(intent.skill_id, SkillId::InstallPackage, "{request}");
        assert_eq!(
            intent.args.get("package"),
            Some(&expected_package.to_owned()),
            "{request}"
        );
    }
}

#[test]
fn routes_package_status_matrix() {
    let cases = [
        ("what version of nginx is installed", "nginx"),
        ("is docker installed", "docker.io"),
        ("what updates exist for tomcat", "tomcat10"),
    ];

    for (request, expected_package) in cases {
        let intent = route(request).unwrap_or_else(fallback_intent);
        assert_eq!(intent.skill_id, SkillId::PackageStatus, "{request}");
        assert_eq!(
            intent.args.get("package"),
            Some(&expected_package.to_owned()),
            "{request}"
        );
    }
}

#[test]
fn routes_logs_matrix() {
    let cases = [
        ("show me recent logs for nginx", Some("nginx")),
        ("what happened on this server", None),
        ("show recent errors", None),
    ];

    for (request, expected_service) in cases {
        let intent = route(request).unwrap_or_else(fallback_intent);
        assert_eq!(intent.skill_id, SkillId::LogsGuidance, "{request}");
        match expected_service {
            Some(service) => assert_eq!(intent.args.get("service"), Some(&service.to_owned())),
            None => assert!(!intent.args.contains_key("service"), "{request}"),
        }
    }
}

#[test]
fn routes_maintenance_matrix() {
    let cases = [
        "what are the security best practices for updates and kernels",
        "system updates guidance",
        "kernel updates and patching",
        "host hardening maintenance checklist",
    ];

    for request in cases {
        let intent = route(request).unwrap_or_else(fallback_intent);
        assert_eq!(intent.skill_id, SkillId::MaintenanceGuidance, "{request}");
    }
}

#[test]
fn routes_firewall_and_service_matrices() {
    let firewall = route("open port 8080").unwrap_or_else(fallback_intent);
    assert_eq!(firewall.skill_id, SkillId::TroubleshootFirewall);
    assert_eq!(firewall.args.get("port"), Some(&"8080".to_owned()));

    let service = route("service nginx won't start").unwrap_or_else(fallback_intent);
    assert_eq!(service.skill_id, SkillId::ServiceStatus);
    assert_eq!(service.args.get("service"), Some(&"nginx".to_owned()));
}

#[test]
fn prefers_package_status_over_install_for_installed_phrase() {
    let intent = route("is nginx installed").unwrap_or_else(fallback_intent);
    assert_eq!(intent.skill_id, SkillId::PackageStatus);
}

#[test]
fn unsupported_requests_return_none() {
    let cases = [
        "write a python script",
        "refactor my code",
        "tell me a joke",
        "summarize this article",
    ];

    for request in cases {
        assert!(route(request).is_none(), "{request}");
    }
}
