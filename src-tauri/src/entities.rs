use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDefinition {
    pub node_type: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    pub shape: String,
}

pub struct EntityRegistry {
    entities: HashMap<String, EntityDefinition>,
}

impl EntityRegistry {
    pub fn new() -> Self {
        let mut entities = HashMap::new();

        let defaults = vec![
            EntityDefinition {
                node_type: "IP".into(),
                display_name: "IP Address".into(),
                icon: "ip".into(),
                color: "--cyto-ip-bg".into(),
                shape: "ellipse".into(),
            },
            EntityDefinition {
                node_type: "IPv6".into(),
                display_name: "IPv6 Address".into(),
                icon: "ip".into(),
                color: "--cyto-ipv6-bg".into(),
                shape: "ellipse".into(),
            },
            EntityDefinition {
                node_type: "Domain".into(),
                display_name: "Domain".into(),
                icon: "domain".into(),
                color: "--cyto-domain-bg".into(),
                shape: "round-rectangle".into(),
            },
            EntityDefinition {
                node_type: "URL".into(),
                display_name: "URL".into(),
                icon: "url".into(),
                color: "--cyto-url-bg".into(),
                shape: "round-tag".into(),
            },
            EntityDefinition {
                node_type: "Email".into(),
                display_name: "Email".into(),
                icon: "email".into(),
                color: "--cyto-email-bg".into(),
                shape: "diamond".into(),
            },
            EntityDefinition {
                node_type: "Person".into(),
                display_name: "Person".into(),
                icon: "person".into(),
                color: "--cyto-person-bg".into(),
                shape: "hexagon".into(),
            },
            EntityDefinition {
                node_type: "Organisation".into(),
                display_name: "Organisation".into(),
                icon: "org".into(),
                color: "--cyto-org-bg".into(),
                shape: "round-pentagon".into(),
            },
            EntityDefinition {
                node_type: "Hash".into(),
                display_name: "Hash".into(),
                icon: "hash".into(),
                color: "--cyto-hash-bg".into(),
                shape: "octagon".into(),
            },
            EntityDefinition {
                node_type: "Location".into(),
                display_name: "Location".into(),
                icon: "location".into(),
                color: "--cyto-location-bg".into(),
                shape: "round-diamond".into(),
            },
            EntityDefinition {
                node_type: "VirusTotalReport".into(),
                display_name: "VirusTotal Report".into(),
                icon: "vt".into(),
                color: "--cyto-vt-bg".into(),
                shape: "round-rectangle".into(),
            },
            EntityDefinition {
                node_type: "Collection".into(),
                display_name: "Collection".into(),
                icon: "collection".into(),
                color: "--cyto-collection-bg".into(),
                shape: "barrel".into(),
            },
        ];

        for def in defaults {
            entities.insert(def.node_type.clone(), def);
        }

        Self { entities }
    }

    pub fn all(&self) -> Vec<EntityDefinition> {
        self.entities.values().cloned().collect()
    }

    pub fn get(&self, node_type: &str) -> Option<&EntityDefinition> {
        self.entities.get(node_type)
    }
}

impl Default for EntityRegistry {
    fn default() -> Self {
        Self::new()
    }
}
