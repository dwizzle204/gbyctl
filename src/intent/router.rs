//! Deterministic keyword-based intent routing for v1.

use std::collections::BTreeMap;

use crate::skills::types::{IntentConfidence, IntentMatch, SkillId};

/// Map user request text into a supported skill.
#[must_use]
pub fn route(request: &str) -> Option<IntentMatch> {
    let text = request.to_lowercase();

    if let Some(package) = extract_package_hint(&text) {
        let mut args = BTreeMap::new();
        let _old = args.insert("package".to_owned(), package);
        return Some(IntentMatch {
            skill_id: SkillId::InstallPackage,
            confidence: IntentConfidence::High,
            args,
        });
    }

    if is_disk_pressure_request(&text) {
        return Some(IntentMatch {
            skill_id: SkillId::DiskFullTriage,
            confidence: IntentConfidence::High,
            args: BTreeMap::new(),
        });
    }

    if text.contains("resize") && text.contains("root") {
        return Some(IntentMatch {
            skill_id: SkillId::ResizeRootPlan,
            confidence: IntentConfidence::High,
            args: BTreeMap::new(),
        });
    }

    if is_maintenance_guidance_request(&text) {
        return Some(IntentMatch {
            skill_id: SkillId::MaintenanceGuidance,
            confidence: IntentConfidence::Medium,
            args: BTreeMap::new(),
        });
    }

    if let Some(package) = extract_package_status_hint(&text) {
        let mut args = BTreeMap::new();
        let _old = args.insert("package".to_owned(), package);
        return Some(IntentMatch {
            skill_id: SkillId::PackageStatus,
            confidence: IntentConfidence::Medium,
            args,
        });
    }

    if text.contains("reboot") || text.contains("kernel") || text.contains("crash") {
        return Some(IntentMatch {
            skill_id: SkillId::DiagnoseRebootOrKernelIssue,
            confidence: IntentConfidence::High,
            args: BTreeMap::new(),
        });
    }

    if is_logs_request(&text) {
        let mut args = BTreeMap::new();
        if let Some(service) = extract_service_hint(&text) {
            let _old = args.insert("service".to_owned(), service);
        }
        return Some(IntentMatch {
            skill_id: SkillId::LogsGuidance,
            confidence: IntentConfidence::Medium,
            args,
        });
    }

    if is_firewall_request(&text) {
        let mut args = BTreeMap::new();
        if let Some(port) = extract_first_port(&text) {
            let _old = args.insert("port".to_owned(), port);
        }
        return Some(IntentMatch {
            skill_id: SkillId::TroubleshootFirewall,
            confidence: IntentConfidence::Medium,
            args,
        });
    }

    if text.contains("storage") || text.contains("lsblk") || text.contains("mount") {
        return Some(IntentMatch {
            skill_id: SkillId::InspectStorage,
            confidence: IntentConfidence::Medium,
            args: BTreeMap::new(),
        });
    }

    if is_health_request(&text) {
        return Some(IntentMatch {
            skill_id: SkillId::Doctor,
            confidence: IntentConfidence::Medium,
            args: BTreeMap::new(),
        });
    }

    if is_service_request(&text) {
        let mut args = BTreeMap::new();
        if let Some(service) = extract_service_hint(&text) {
            let _old = args.insert("service".to_owned(), service);
        }
        return Some(IntentMatch {
            skill_id: SkillId::ServiceStatus,
            confidence: IntentConfidence::Medium,
            args,
        });
    }

    None
}

fn is_disk_pressure_request(text: &str) -> bool {
    (text.contains("disk") || text.contains("storage") || text.contains("space"))
        && (text.contains("full")
            || text.contains("usage")
            || text.contains("out of space")
            || text.contains("no space"))
}

fn is_firewall_request(text: &str) -> bool {
    text.contains("firewall")
        || text.contains("ufw")
        || text.contains("open port")
        || text.contains("allow port")
        || text.contains("cannot reach port")
        || text.contains("can't reach port")
        || text.contains("port ")
}

fn is_service_request(text: &str) -> bool {
    text.contains("service")
        || text.contains("daemon")
        || text.contains("systemctl")
        || text.contains("won't start")
        || text.contains("failed to start")
        || text.contains("status of")
}

fn is_health_request(text: &str) -> bool {
    text.contains("doctor")
        || text.contains("health")
        || text.contains("server slow")
        || text.contains("why is my server slow")
        || text.contains("running services")
        || text.contains("list services")
        || text.contains("check system")
}

fn is_maintenance_guidance_request(text: &str) -> bool {
    text.contains("best practices")
        || text.contains("security updates")
        || text.contains("system updates")
        || text.contains("kernel updates")
        || text.contains("hardening")
        || text.contains("patching")
        || text.contains("maintenance")
}

fn is_logs_request(text: &str) -> bool {
    text.contains("logs")
        || text.contains("what happened")
        || text.contains("recent errors")
        || text.contains("journal")
        || text.contains("why did this happen")
}

fn extract_first_port(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        if token.chars().all(|ch| ch.is_ascii_digit()) {
            return Some(token.to_owned());
        }
    }
    None
}

fn extract_service_hint(text: &str) -> Option<String> {
    let known = [
        "ssh",
        "sshd",
        "nginx",
        "apache2",
        "tomcat10",
        "docker",
        "docker.io",
        "containerd",
        "postgresql",
        "mysql",
        "mariadb",
        "ufw",
    ];
    for service in known {
        if text.contains(service) {
            return Some(service.to_owned());
        }
    }
    None
}

fn extract_package_hint(text: &str) -> Option<String> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let has_install_verb = tokens.iter().any(|token| {
        let normalized = token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-');
        normalized == "install" || normalized == "add"
    });
    if !has_install_verb {
        return None;
    }

    let known = [
        "tomcat10",
        "tomcat",
        "nginx",
        "docker.io",
        "docker",
        "postgresql",
        "mysql-server",
        "apache2",
        "redis-server",
    ];
    for package in known {
        if text.contains(package) {
            return Some(if package == "tomcat" {
                "tomcat10".to_owned()
            } else if package == "docker" {
                "docker.io".to_owned()
            } else {
                package.to_owned()
            });
        }
    }

    for window in tokens.windows(2) {
        if let [action, value] = window
            && (*action == "install" || (*action == "add" && text.contains("package")))
        {
            let candidate = value.trim_matches(|c: char| {
                !c.is_ascii_alphanumeric() && c != '.' && c != '+' && c != '-'
            });
            if !candidate.is_empty() {
                return Some(candidate.to_owned());
            }
        }
    }

    None
}

fn extract_package_status_hint(text: &str) -> Option<String> {
    let asks_package_status = text.contains("installed")
        || text.contains("version")
        || text.contains("update")
        || text.contains("upgradable")
        || text.contains("package status");
    if !asks_package_status {
        return None;
    }

    let known = [
        "nginx",
        "docker.io",
        "docker",
        "tomcat10",
        "tomcat",
        "postgresql",
        "mysql-server",
        "apache2",
        "redis-server",
    ];
    for package in known {
        if text.contains(package) {
            return Some(if package == "tomcat" {
                "tomcat10".to_owned()
            } else if package == "docker" {
                "docker.io".to_owned()
            } else {
                package.to_owned()
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::route;
    use crate::skills::types::{IntentConfidence, IntentMatch, SkillId};

    #[test]
    fn routes_disk_full() {
        let intent = route("my disk is full");
        assert!(intent.is_some());
        let intent = intent.unwrap_or(IntentMatch {
            skill_id: SkillId::Doctor,
            confidence: IntentConfidence::Low,
            args: BTreeMap::new(),
        });
        assert_eq!(intent.skill_id, SkillId::DiskFullTriage);
    }

    #[test]
    fn extracts_firewall_port() {
        let intent = route("why cannot hit port 8080");
        assert!(intent.is_some());
        let intent = intent.unwrap_or(IntentMatch {
            skill_id: SkillId::Doctor,
            confidence: IntentConfidence::Low,
            args: BTreeMap::new(),
        });
        assert_eq!(intent.skill_id, SkillId::TroubleshootFirewall);
        let port = intent.args.get("port");
        assert_eq!(port, Some(&"8080".to_owned()));
    }

    #[test]
    fn routes_slow_server_to_doctor() {
        let intent = route("why is my server slow");
        assert!(intent.is_some());
        let intent = if let Some(intent) = intent {
            intent
        } else {
            return;
        };
        assert_eq!(intent.skill_id, SkillId::Doctor);
    }

    #[test]
    fn extracts_package_install_request() {
        let intent = route("install nginx");
        assert!(intent.is_some());
        let intent = if let Some(intent) = intent {
            intent
        } else {
            return;
        };
        assert_eq!(intent.skill_id, SkillId::InstallPackage);
        assert_eq!(intent.args.get("package"), Some(&"nginx".to_owned()));
    }

    #[test]
    fn routes_security_best_practices() {
        let intent = route("what are the security best practices for updates and kernels");
        assert!(intent.is_some());
        let intent = if let Some(intent) = intent {
            intent
        } else {
            return;
        };
        assert_eq!(intent.skill_id, SkillId::MaintenanceGuidance);
    }

    #[test]
    fn routes_package_version_question() {
        let intent = route("what version of nginx is installed");
        assert!(intent.is_some());
        let intent = if let Some(intent) = intent {
            intent
        } else {
            return;
        };
        assert_eq!(intent.skill_id, SkillId::PackageStatus);
        assert_eq!(intent.args.get("package"), Some(&"nginx".to_owned()));
    }

    #[test]
    fn routes_log_question() {
        let intent = route("show me recent logs for nginx");
        assert!(intent.is_some());
        let intent = if let Some(intent) = intent {
            intent
        } else {
            return;
        };
        assert_eq!(intent.skill_id, SkillId::LogsGuidance);
        assert_eq!(intent.args.get("service"), Some(&"nginx".to_owned()));
    }
}
