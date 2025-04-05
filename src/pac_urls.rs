/// Represents a PAC file URL with its description
#[derive(Debug, Clone)]
pub struct PacUrl {
    /// The URL of the PAC file
    pub url: String,
    /// Description of what this PAC file is used for
    pub description: String,
}

/// Returns a Vec of known PAC file URLs and their descriptions in the order they should be loaded
pub fn get_pac_urls() -> Vec<PacUrl> {
    vec![
        PacUrl {
            url: "http://wpad/wpad.dat".to_string(),
            description: "WPAD (Web Proxy Auto-Discovery Protocol) PAC file".to_string(),
        },
        PacUrl {
            url: "http://localhost:3333/files/proxy.pac".to_string(),
            description: "itTLS PAC file".to_string(),
        },
        PacUrl {
            url: "http://localhost:10224/proxy.pac".to_string(),
            description: "avTune PAC file".to_string(),
        },
    ]
} 