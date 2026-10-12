//! Automated Tests for Link Mage Enhancements
//!
//! Tests for: Provenance

// ── Provenance Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod provenance_tests {
    use crate::provenance::*;

    #[test]
    fn test_manual_provenance_creation() {
        let prov = Provenance::manual("test_user");

        assert_eq!(prov.source, ProvenanceSource::Manual);
        assert_eq!(prov.created_by, "test_user");
        assert_eq!(prov.confidence, 1.0);
        assert!(prov.transform_chain.is_empty());
        assert!(!prov.created_at.is_empty());
    }

    #[test]
    fn test_transform_provenance_creation() {
        let prov = Provenance::from_transform(
            ProvenanceSource::DnsLookup,
            "node_123",
            "DNS Lookup",
            "system",
            Some(150),
            Some("example.com".to_string()),
        );

        assert_eq!(prov.source, ProvenanceSource::DnsLookup);
        assert_eq!(prov.created_by, "system");
        assert_eq!(prov.transform_chain.len(), 1);
        assert_eq!(prov.transform_chain[0].transform_name, "DNS Lookup");
        assert_eq!(prov.transform_chain[0].source_node_id, "node_123");
        assert_eq!(prov.transform_chain[0].duration_ms, Some(150));
        assert_eq!(prov.external_ref, Some("example.com".to_string()));
    }

    #[test]
    fn test_confidence_by_source() {
        assert_eq!(
            Provenance::source_reliability(&ProvenanceSource::Manual),
            1.0
        );
        assert_eq!(
            Provenance::source_reliability(&ProvenanceSource::DnsLookup),
            0.95
        );
        assert_eq!(
            Provenance::source_reliability(&ProvenanceSource::WhoisLookup),
            0.90
        );
        assert_eq!(
            Provenance::source_reliability(&ProvenanceSource::GeoipLookup),
            0.85
        );
        assert_eq!(
            Provenance::source_reliability(&ProvenanceSource::WebScrape),
            0.75
        );
        assert_eq!(
            Provenance::source_reliability(&ProvenanceSource::Virustotal),
            0.95
        );
        assert_eq!(
            Provenance::source_reliability(&ProvenanceSource::RemoteTransform),
            0.70
        );
        assert_eq!(
            Provenance::source_reliability(&ProvenanceSource::Plugin),
            0.80
        );
        assert_eq!(
            Provenance::source_reliability(&ProvenanceSource::Import),
            0.85
        );
        assert_eq!(
            Provenance::source_reliability(&ProvenanceSource::Unknown),
            0.50
        );
    }

    #[test]
    fn test_add_transform_step() {
        let mut prov = Provenance::manual("user");

        prov = prov.add_transform_step("DNS Lookup", "node_123", Some(100), true, None);

        assert_eq!(prov.transform_chain.len(), 1);
        assert_eq!(prov.transform_chain[0].transform_name, "DNS Lookup");
        assert!(prov.transform_chain[0].success);
        assert!(prov.confidence < 1.0); // Decayed
    }

    #[test]
    fn test_confidence_decay() {
        let mut prov = Provenance::manual("user");
        let initial_confidence = prov.confidence;

        for i in 0..5 {
            prov = prov.add_transform_step(
                &format!("Step {}", i),
                &format!("node_{}", i),
                Some(50),
                true,
                None,
            );
        }

        assert_eq!(prov.transform_chain.len(), 5);
        assert!(prov.confidence < initial_confidence);
        assert!(prov.confidence >= 0.1); // Minimum floor
    }

    #[test]
    fn test_failed_transform_recording() {
        let prov = Provenance::manual("user");
        let updated = prov.add_transform_step(
            "Failed Transform",
            "node_123",
            Some(100),
            false,
            Some("Connection timeout".to_string()),
        );

        assert!(!updated.transform_chain[0].success);
        assert_eq!(
            updated.transform_chain[0].error,
            Some("Connection timeout".to_string())
        );
    }

    #[test]
    fn test_merge_provenance() {
        let prov1 = Provenance::manual("user1");
        let prov2 = Provenance::from_transform(
            ProvenanceSource::DnsLookup,
            "node_456",
            "DNS",
            "system",
            None,
            None,
        );

        let merged = Provenance::merge(&[prov1.clone(), prov2.clone()]);

        assert!(!merged.transform_chain.is_empty());
        assert_eq!(merged.notes, Some("Merged from 2 sources".to_string()));
        assert!(merged.confidence > 0.0);
    }

    #[test]
    fn test_format_provenance() {
        let prov = Provenance::from_transform(
            ProvenanceSource::DnsLookup,
            "node_123",
            "DNS Lookup",
            "system",
            Some(150),
            Some("example.com".to_string()),
        );

        let formatted = prov.format();

        assert!(formatted.contains("Source:"));
        assert!(formatted.contains("dns_lookup"));
        assert!(formatted.contains("Created:"));
        assert!(formatted.contains("Confidence:"));
        assert!(formatted.contains("DNS Lookup"));
    }

    #[test]
    fn test_provenance_tracker() {
        let tracker = ProvenanceTracker::new("DNS Lookup", "node_123");

        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert!(tracker.duration_ms() >= 10);

        let prov = tracker.create_success_provenance(
            ProvenanceSource::DnsLookup,
            "system",
            Some("example.com".to_string()),
        );

        assert_eq!(prov.source, ProvenanceSource::DnsLookup);
        assert!(!prov.transform_chain.is_empty());
    }

    #[test]
    fn test_provenance_source_from_str() {
        assert_eq!(ProvenanceSource::from("manual"), ProvenanceSource::Manual);
        assert_eq!(
            ProvenanceSource::from("dns_lookup"),
            ProvenanceSource::DnsLookup
        );
        assert_eq!(
            ProvenanceSource::from("whois"),
            ProvenanceSource::WhoisLookup
        );
        assert_eq!(
            ProvenanceSource::from("geoip"),
            ProvenanceSource::GeoipLookup
        );
        assert_eq!(
            ProvenanceSource::from("virustotal"),
            ProvenanceSource::Virustotal
        );
        assert_eq!(
            ProvenanceSource::from("unknown_source"),
            ProvenanceSource::Unknown
        );
    }

    #[test]
    fn test_provenance_display() {
        assert_eq!(format!("{}", ProvenanceSource::Manual), "manual");
        assert_eq!(format!("{}", ProvenanceSource::DnsLookup), "dns_lookup");
        assert_eq!(format!("{}", ProvenanceSource::Unknown), "unknown");
    }
}

// ── Integration Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod integration_tests {
    use crate::graph::{Edge, Node};
    use crate::provenance::Provenance;

    #[test]
    fn test_node_with_provenance() {
        let prov = Provenance::manual("tester");
        let node = Node::with_provenance("Domain", prov.clone());

        assert_eq!(node.node_type, "Domain");
        assert!(node.provenance.is_some());
        assert_eq!(node.provenance.as_ref().unwrap().source, prov.source);
    }

    #[test]
    fn test_edge_with_provenance() {
        let prov = Provenance::manual("tester");
        let mut edge = Edge::new("source_id", "target_id", "test-edge");
        edge.provenance = Some(prov);

        assert!(edge.provenance.is_some());
        assert_eq!(edge.provenance.as_ref().unwrap().created_by, "tester");
    }

    #[test]
    fn test_provenance_serialization() {
        let prov = Provenance::from_transform(
            crate::provenance::ProvenanceSource::DnsLookup,
            "node_123",
            "DNS Lookup",
            "system",
            Some(150),
            Some("example.com".to_string()),
        );

        let json = serde_json::to_string(&prov).unwrap();
        assert!(json.contains("dns_lookup"));
        assert!(json.contains("DNS Lookup"));
        assert!(json.contains("example.com"));

        let deserialized: Provenance = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.source, prov.source);
        assert_eq!(
            deserialized.transform_chain.len(),
            prov.transform_chain.len()
        );
    }
}
