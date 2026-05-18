//! Time utilities for the core runtime.
//!
//! All timestamps are UTC. The canonical serialization format is ISO 8601
//! with Z suffix (matching SyncServer's format).

use time::OffsetDateTime;

/// Canonical timestamp type used throughout the core.
pub type Timestamp = OffsetDateTime;

/// Format a timestamp as ISO 8601 with Z suffix.
pub fn format_iso8601(ts: Timestamp) -> String {
    ts.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// Parse an ISO 8601 timestamp string (with or without Z).
pub fn parse_iso8601(s: &str) -> Result<Timestamp, time::error::Parse> {
    OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
}

/// Current UTC time.
pub fn now_utc() -> Timestamp {
    OffsetDateTime::now_utc()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_iso8601() {
        let ts = now_utc();
        let s = format_iso8601(ts);
        let parsed = parse_iso8601(&s).unwrap();
        let back = format_iso8601(parsed);
        assert_eq!(s, back);
    }

    #[test]
    fn parse_syncserver_format() {
        let s = "2026-01-15T10:30:00Z";
        let parsed = parse_iso8601(s);
        assert!(parsed.is_ok());
    }
}
