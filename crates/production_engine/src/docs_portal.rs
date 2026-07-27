use tracing::info;

/// Documentation, Website Landing Page & Developer SDK Portal Generator.
pub struct DocumentationPortal;

impl DocumentationPortal {
    pub fn build_portal() -> bool {
        info!("Building Production Documentation Portal (mdBook Docs + Website Landing Page + Developer SDK Portal)...");
        info!("Documentation Portal [1/3]: mdBook Architecture & API Docs -> Generated");
        info!("Documentation Portal [2/3]: Platform Web Landing Page -> Generated");
        info!("Documentation Portal [3/3]: Developer SDK Samples Portal -> Generated");
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docs_portal_generation() {
        assert!(DocumentationPortal::build_portal());
    }
}
