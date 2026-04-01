//! Data Provenance Tracking
//!
//! Tracks the origin and transformation history of each entity.
//! This provides forensic integrity and audit trails for investigations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single step in the transform chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformStep {
    /// Name of the transform that was applied
    pub transform_name: String,
    /// Timestamp when transform was executed (ISO 8601)
    pub executed_at: String,
    /// ID of the source node that was transformed
    pub source_node_id: String,
    /// Transform execution duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Whether transform succeeded
    pub success: bool,
    /// Error message if transform failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Source types for entity creation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceSource {
    #[default]
    Unknown,          // Source could not be determined
    Manual,           // User manually created the entity
    DnsLookup,        // Created via DNS lookup transform
    WhoisLookup,      // Created via WHOIS lookup transform
    GeoipLookup,      // Created via GeoIP transform
    WebScrape,        // Created via web scraping
    Virustotal,       // Created via VirusTotal lookup
    RemoteTransform,  // Created via remote transform
    Plugin,           // Created via native plugin
    Import,           // Imported from file
    Pipeline,         // Created via transform pipeline
}

impl std::fmt::Display for ProvenanceSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProvenanceSource::Manual => write!(f, "manual"),
            ProvenanceSource::DnsLookup => write!(f, "dns_lookup"),
            ProvenanceSource::WhoisLookup => write!(f, "whois_lookup"),
            ProvenanceSource::GeoipLookup => write!(f, "geoip_lookup"),
            ProvenanceSource::WebScrape => write!(f, "web_scrape"),
            ProvenanceSource::Virustotal => write!(f, "virustotal"),
            ProvenanceSource::RemoteTransform => write!(f, "remote_transform"),
            ProvenanceSource::Plugin => write!(f, "plugin"),
            ProvenanceSource::Import => write!(f, "import"),
            ProvenanceSource::Pipeline => write!(f, "pipeline"),
            ProvenanceSource::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for ProvenanceSource {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "manual" => ProvenanceSource::Manual,
            "dns_lookup" | "dns" => ProvenanceSource::DnsLookup,
            "whois_lookup" | "whois" => ProvenanceSource::WhoisLookup,
            "geoip_lookup" | "geoip" => ProvenanceSource::GeoipLookup,
            "web_scrape" | "scrape" => ProvenanceSource::WebScrape,
            "virustotal" | "vt" => ProvenanceSource::Virustotal,
            "remote_transform" | "remote" => ProvenanceSource::RemoteTransform,
            "plugin" => ProvenanceSource::Plugin,
            "import" => ProvenanceSource::Import,
            "pipeline" => ProvenanceSource::Pipeline,
            _ => ProvenanceSource::Unknown,
        }
    }
}

/// Provenance metadata for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Original source of this entity
    pub source: ProvenanceSource,
    /// Timestamp when entity was first created (ISO 8601)
    pub created_at: String,
    /// User or system that created the entity
    pub created_by: String,
    /// Chain of transforms that led to this entity
    #[serde(default)]
    pub transform_chain: Vec<TransformStep>,
    /// Confidence score (0.0 - 1.0) based on source reliability
    pub confidence: f64,
    /// External reference ID (e.g., VirusTotal hash, WHOIS record ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    /// Notes about the provenance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Default for Provenance {
    fn default() -> Self {
        Self {
            source: ProvenanceSource::default(),
            created_at: String::new(),
            created_by: String::new(),
            transform_chain: Vec::new(),
            confidence: 0.5,
            external_ref: None,
            notes: None,
        }
    }
}

impl Provenance {
    /// Create provenance for a manually created entity
    pub fn manual(created_by: &str) -> Self {
        Self {
            source: ProvenanceSource::Manual,
            created_at: Utc::now().to_rfc3339(),
            created_by: created_by.to_string(),
            transform_chain: Vec::new(),
            confidence: 1.0,
            external_ref: None,
            notes: None,
        }
    }

    /// Create provenance for an entity created by a transform
    #[allow(clippy::too_many_arguments)]
    pub fn from_transform(
        source: ProvenanceSource,
        source_node_id: &str,
        transform_name: &str,
        created_by: &str,
        duration_ms: Option<u64>,
        external_ref: Option<String>,
    ) -> Self {
        let executed_at = Utc::now().to_rfc3339();
        let step = TransformStep {
            transform_name: transform_name.to_string(),
            executed_at: executed_at.clone(),
            source_node_id: source_node_id.to_string(),
            duration_ms,
            success: true,
            error: None,
        };

        let confidence = Self::calculate_confidence(&source, &[step.clone()]);

        Self {
            source,
            created_at: executed_at,
            created_by: created_by.to_string(),
            transform_chain: vec![step],
            confidence,
            external_ref,
            notes: None,
        }
    }

    /// Add a transform step to existing provenance
    pub fn add_transform_step(
        &self,
        transform_name: &str,
        source_node_id: &str,
        duration_ms: Option<u64>,
        success: bool,
        error: Option<String>,
    ) -> Self {
        let step = TransformStep {
            transform_name: transform_name.to_string(),
            executed_at: Utc::now().to_rfc3339(),
            source_node_id: source_node_id.to_string(),
            duration_ms,
            success,
            error,
        };

        let mut new_chain = self.transform_chain.clone();
        new_chain.push(step);

        let confidence = Self::calculate_confidence(&self.source, &new_chain);

        Self {
            transform_chain: new_chain,
            confidence,
            ..self.clone()
        }
    }

    /// Calculate confidence score based on source and transform chain
    /// Confidence decreases with each transform step (decay factor)
    fn calculate_confidence(source: &ProvenanceSource, chain: &[TransformStep]) -> f64 {
        let base_confidence = Self::source_reliability(source);
        // Apply decay for each transform step (2% per step)
        let decay_factor = 0.98_f64;
        let chain_decay = decay_factor.powi(chain.len() as i32);
        
        // Factor in failed transforms
        let success_rate = if chain.is_empty() {
            1.0
        } else {
            chain.iter().filter(|s| s.success).count() as f64 / chain.len() as f64
        };

        (base_confidence * chain_decay * success_rate).max(0.1)
    }

    /// Get reliability weight for a source type (public for testing)
    pub fn source_reliability(source: &ProvenanceSource) -> f64 {
        match source {
            ProvenanceSource::Manual => 1.0,
            ProvenanceSource::DnsLookup => 0.95,
            ProvenanceSource::WhoisLookup => 0.90,
            ProvenanceSource::GeoipLookup => 0.85,
            ProvenanceSource::WebScrape => 0.75,
            ProvenanceSource::Virustotal => 0.95,
            ProvenanceSource::RemoteTransform => 0.70,
            ProvenanceSource::Plugin => 0.80,
            ProvenanceSource::Import => 0.85,
            ProvenanceSource::Pipeline => 0.75,
            ProvenanceSource::Unknown => 0.50,
        }
    }

    /// Merge provenance when entities are combined
    pub fn merge(provenances: &[Provenance]) -> Self {
        if provenances.is_empty() {
            return Self::manual("system");
        }

        if provenances.len() == 1 {
            return provenances[0].clone();
        }

        // Use the oldest creation date
        let earliest = provenances
            .iter()
            .min_by_key(|p| DateTime::parse_from_rfc3339(&p.created_at).unwrap_or_default())
            .unwrap();

        // Combine all transform chains
        let mut all_steps: Vec<TransformStep> = provenances
            .iter()
            .flat_map(|p| p.transform_chain.clone())
            .collect();
        all_steps.sort_by(|a, b| {
            DateTime::parse_from_rfc3339(&a.executed_at)
                .unwrap_or_default()
                .cmp(&DateTime::parse_from_rfc3339(&b.executed_at).unwrap_or_default())
        });

        // Average confidence
        let avg_confidence = provenances.iter().map(|p| p.confidence).sum::<f64>() / provenances.len() as f64;

        Self {
            source: earliest.source.clone(),
            created_at: earliest.created_at.clone(),
            created_by: earliest.created_by.clone(),
            transform_chain: all_steps,
            confidence: avg_confidence,
            notes: Some(format!("Merged from {} sources", provenances.len())),
            ..Default::default()
        }
    }

    /// Format provenance for display
    pub fn format(&self) -> String {
        let mut parts = Vec::new();

        // Source
        parts.push(format!("Source: {}", self.source));

        // Created
        if let Ok(created_date) = DateTime::parse_from_rfc3339(&self.created_at) {
            parts.push(format!(
                "Created: {}",
                created_date.format("%Y-%m-%d %H:%M:%S").to_string()
            ));
        }

        // Creator
        if !self.created_by.is_empty() {
            parts.push(format!("By: {}", self.created_by));
        }

        // Confidence
        parts.push(format!("Confidence: {:.0}%", self.confidence * 100.0));

        // External reference
        if let Some(ref ext_ref) = self.external_ref {
            parts.push(format!("Ref: {}", ext_ref));
        }

        // Transform chain
        if !self.transform_chain.is_empty() {
            parts.push(format!("\nTransform Chain ({} steps):", self.transform_chain.len()));
            for (i, step) in self.transform_chain.iter().enumerate() {
                let status = if step.success { "✓" } else { "✗" };
                let time_str = if let Ok(dt) = DateTime::parse_from_rfc3339(&step.executed_at) {
                    dt.format("%H:%M:%S").to_string()
                } else {
                    step.executed_at.clone()
                };
                parts.push(format!(
                    "  {}. {} {} ({})",
                    i + 1,
                    status,
                    step.transform_name,
                    time_str
                ));
            }
        }

        parts.join("\n")
    }
}

/// Helper to create a provenance tracking context for transforms
pub struct ProvenanceTracker {
    start_time: DateTime<Utc>,
    transform_name: String,
    source_node_id: String,
}

impl ProvenanceTracker {
    pub fn new(transform_name: &str, source_node_id: &str) -> Self {
        Self {
            start_time: Utc::now(),
            transform_name: transform_name.to_string(),
            source_node_id: source_node_id.to_string(),
        }
    }

    pub fn duration_ms(&self) -> u64 {
        Utc::now()
            .signed_duration_since(self.start_time)
            .num_milliseconds() as u64
    }

    pub fn create_success_provenance(
        &self,
        source: ProvenanceSource,
        created_by: &str,
        external_ref: Option<String>,
    ) -> Provenance {
        Provenance::from_transform(
            source,
            &self.source_node_id,
            &self.transform_name,
            created_by,
            Some(self.duration_ms()),
            external_ref,
        )
    }

    pub fn add_step_to_provenance(
        &self,
        existing: &Provenance,
        success: bool,
        error: Option<String>,
    ) -> Provenance {
        existing.add_transform_step(
            &self.transform_name,
            &self.source_node_id,
            Some(self.duration_ms()),
            success,
            error,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_provenance() {
        let prov = Provenance::manual("test_user");
        assert_eq!(prov.source, ProvenanceSource::Manual);
        assert_eq!(prov.created_by, "test_user");
        assert_eq!(prov.confidence, 1.0);
        assert!(prov.transform_chain.is_empty());
    }

    #[test]
    fn test_transform_provenance() {
        let prov = Provenance::from_transform(
            ProvenanceSource::DnsLookup,
            "node-123",
            "DNS Lookup",
            "system",
            Some(150),
            Some("example.com".to_string()),
        );
        assert_eq!(prov.source, ProvenanceSource::DnsLookup);
        assert_eq!(prov.transform_chain.len(), 1);
        assert_eq!(prov.transform_chain[0].transform_name, "DNS Lookup");
        assert!(prov.transform_chain[0].success);
    }

    #[test]
    fn test_confidence_decay() {
        let mut prov = Provenance::manual("user");
        
        // Add multiple transform steps
        for i in 0..5 {
            prov = prov.add_transform_step(
                &format!("Transform {}", i),
                "node-123",
                Some(100),
                true,
                None,
            );
        }
        
        // Confidence should decrease with each step
        assert!(prov.confidence < 1.0);
        assert!(prov.confidence > 0.1);
    }

    #[test]
    fn test_merge_provenance() {
        let prov1 = Provenance::manual("user1");
        let prov2 = Provenance::from_transform(
            ProvenanceSource::DnsLookup,
            "node-456",
            "DNS",
            "system",
            None,
            None,
        );

        let merged = Provenance::merge(&[prov1, prov2]);
        assert!(merged.transform_chain.len() >= 1);
        assert_eq!(merged.notes, Some("Merged from 2 sources".to_string()));
    }
}
