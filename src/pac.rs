use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::VecDeque;
use crate::proxy_config::ProxyConfig;
use crate::constants::APP_CONFIG;
use log::{info, error, warn, debug};
use crate::error::{Result, ProxyCatError};

/// Represents a proxy rule with host and proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRule {
    /// The hostname to match
    pub host: String,
    /// The proxy server hostname
    pub proxy_host: String,
    /// The proxy server port
    pub proxy_port: u16,
}

/// Represents an external PAC function with its modified name and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPacFunction {
    /// The original URL where the PAC file was loaded from
    pub original_url: String,
    /// The modified function name (with unique suffix)
    pub function_name: String,
    /// The complete function text with modified name
    pub function_text: String,
}

/// Wrapper for ProxyRule with enabled/disabled state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRuleItem {
    /// The proxy rule
    pub rule: ProxyRule,
    /// Whether this rule is enabled
    pub enabled: bool,
}

/// Wrapper for bypass list item with enabled/disabled state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BypassListItem {
    /// The hostname or IP address to bypass
    pub host: String,
    /// Whether this bypass rule is enabled
    pub enabled: bool,
}

/// Wrapper for ExternalPacFunction with enabled/disabled state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPacFunctionItem {
    /// The external PAC function
    pub function: ExternalPacFunction,
    /// Whether this function is enabled
    pub enabled: bool,
}

/// Represents the Proxy Auto-Configuration (PAC) settings
/// This struct contains the configuration needed to generate a PAC file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacConfig {
    /// List of proxy rules for different hosts
    pub proxy_rules: VecDeque<ProxyRuleItem>,
    /// List of hostnames and IP addresses to bypass the proxy
    pub bypass_list: VecDeque<BypassListItem>,
    /// List of external PAC functions loaded from URLs
    pub external_pac_functions: VecDeque<ExternalPacFunctionItem>,
}

impl Default for PacConfig {
    /// Creates a default PAC configuration with empty rules
    fn default() -> Self {
        info!("Creating default PAC configuration");
        Self {
            proxy_rules: VecDeque::new(),
            bypass_list: VecDeque::from([
                BypassListItem {
                    host: "localhost".to_string(),
                    enabled: true,
                },
                BypassListItem {
                    host: "127.0.0.1".to_string(),
                    enabled: true,
                },
                BypassListItem {
                    host: "::1".to_string(),
                    enabled: true,
                },
            ]),
            external_pac_functions: VecDeque::new(),
        }
    }
}

impl PacConfig {
    /// Creates a PAC configuration from Windows proxy settings
    /// This function parses the Windows proxy configuration and converts it
    /// into a format suitable for generating a PAC file
    pub fn from_windows_config(config: &ProxyConfig) -> Self {
        info!("Converting Windows proxy config to PAC config...");
        let mut pac_config = Self::default();
        
        // Parse proxy server address if present
        if let Some(proxy_server) = &config.proxy_server {
            info!("Processing proxy server: {}", proxy_server);
            if let Some((host, port)) = parse_proxy_server(proxy_server) {
                // Add a default rule for all hosts
                pac_config.proxy_rules.push_back(ProxyRuleItem {
                    rule: ProxyRule {
                        host: "*".to_string(),
                        proxy_host: host.clone(),
                        proxy_port: port,
                    },
                    enabled: true,
                });
                info!("Added default proxy rule - Host: *, Proxy: {}:{}", host, port);
                
                // Save the configuration after adding proxy rule
                if let Err(e) = pac_config.save_current() {
                    error!("Failed to save configuration after adding proxy rule: {}", e);
                }
            } else {
                warn!("Failed to parse proxy server address");
            }
        } else {
            info!("No proxy server configured");
        }

        // Parse bypass list if present
        if let Some(bypass) = &config.proxy_bypass {
            info!("Processing bypass list: {}", bypass);
            let bypass_items: VecDeque<BypassListItem> = bypass
                .split(';')
                .filter(|s| !s.is_empty())
                .map(|s| BypassListItem {
                    host: s.to_string(),
                    enabled: true,
                })
                .collect();
            pac_config.bypass_list = bypass_items;
            debug!("Parsed bypass list: {:?}", pac_config.bypass_list);
        } else {
            info!("No bypass list configured");
        }

        // Add default bypass entries if not present
        for default in ["localhost", "127.0.0.1", "::1"] {
            if !pac_config.bypass_list.iter().any(|item| item.host == default) {
                info!("Adding default bypass entry: {}", default);
                pac_config.bypass_list.push_back(BypassListItem {
                    host: default.to_string(),
                    enabled: true,
                });
            }
        }

        // Save the configuration after adding default entries
        if let Err(e) = pac_config.save_current() {
            error!("Failed to save configuration after adding default entries: {}", e);
        }

        debug!("Final PAC configuration: {:?}", pac_config);
        pac_config
    }

    /// Helper function to find the FindProxyForURL function in the text
    fn find_proxy_function(content: &str) -> Option<(usize, usize)> {
        let function_start = content.find("function FindProxyForURL")?;
        let mut brace_count = 0;
        let mut in_function = false;
        let mut end_pos = 0;

        // Scan through the content to find the matching closing brace
        for (i, c) in content[function_start..].chars().enumerate() {
            if c == '{' {
                brace_count += 1;
                in_function = true;
            } else if c == '}' {
                brace_count -= 1;
                if in_function && brace_count == 0 {
                    end_pos = function_start + i + 1;
                    break;
                }
            }
        }

        if end_pos > function_start {
            Some((function_start, end_pos))
        } else {
            None
        }
    }

    /// Helper function to generate a valid suffix from URL
    fn generate_function_suffix(url: &str) -> String {
        // Simple URL sanitization - replace non-alphanumeric chars with underscore
        url.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
    }

    /// Loads additional proxy rules from an external PAC file
    /// This function fetches the PAC file from the specified URL and parses it
    /// to extract proxy rules, appending them to the existing configuration
    pub async fn load_external_pac(&mut self, url: &str) {
        info!("Loading additional PAC file from {}...", url);
        if let Ok(response) = reqwest::get(url).await {
            if let Ok(content) = response.text().await {
                // Try to find the FindProxyForURL function in the content
                if let Some((start, end)) = Self::find_proxy_function(&content) {
                    let original_function = &content[start..end];
                    
                    // Generate a unique suffix from the URL
                    let suffix = Self::generate_function_suffix(url);
                    let new_function_name = format!("FindProxyForURL_{}", suffix);

                    // Replace the function name
                    let modified_function = original_function.replace(
                        "function FindProxyForURL",
                        &format!("function {}", new_function_name)
                    );
                    // Check if this function name already exists
                    if self.external_pac_functions.iter().any(|f| f.function.function_name == new_function_name) {
                        info!("Function {} already exists, skipping", new_function_name);
                        return;
                    }

                    // Store the external PAC function
                    self.external_pac_functions.push_back(ExternalPacFunctionItem {
                        function: ExternalPacFunction {
                            original_url: url.to_string(),
                            function_name: new_function_name,
                            function_text: modified_function,
                        },
                        enabled: true,
                    });

                    info!("Successfully added external PAC function from {}", url);
                    
                    // Save the updated configuration
                    if let Err(e) = self.save_current() {
                        error!("Failed to save configuration after adding external PAC: {}", e);
                    }
                } else {
                    warn!("No FindProxyForURL function found in the PAC file");
                }
            } else {
                error!("Failed to read PAC file content");
            }
        } else {
            error!("Failed to fetch PAC file from {}", url);
        }
    }

    /// Saves the PAC configuration to a file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ProxyCatError::Pac(format!("Failed to serialize PAC config: {}", e)))?;
        std::fs::write(path, json)
            .map_err(|e| ProxyCatError::Pac(format!("Failed to write PAC config file: {}", e)))?;
        Ok(())
    }

    /// Loads a PAC configuration from a file
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ProxyCatError::Pac(format!("Failed to read PAC config file: {}", e)))?;
        serde_json::from_str(&content)
            .map_err(|e| ProxyCatError::Pac(format!("Failed to deserialize PAC config: {}", e)))
    }

    /// Saves the current PAC configuration to the default location
    pub fn save_current(&self) -> Result<()> {
        self.save_to_file("pac_config.json")
    }
}

/// Parses a proxy server string in the format "host:port"
/// Returns a tuple of (host, port) if successful, None otherwise
fn parse_proxy_server(proxy: &str) -> Option<(String, u16)> {
    debug!("Parsing proxy server string: {}", proxy);
    let parts: Vec<&str> = proxy.split(':').collect();
    if parts.len() == 2 {
        if let Ok(port) = parts[1].parse::<u16>() {
            debug!("Successfully parsed proxy server - Host: {}, Port: {}", parts[0], port);
            return Some((parts[0].to_string(), port));
        }
    }
    warn!("Failed to parse proxy server string");
    None
}

/// Type alias for thread-safe shared access to PAC configuration
pub type SharedPacConfig = Arc<RwLock<PacConfig>>;

/// Generates the content of a PAC file based on the current configuration
/// The PAC file contains JavaScript code that browsers use to determine
/// whether to use a proxy for a given URL
pub fn generate_pac_content(config: &PacConfig) -> String {
    info!("Generating PAC file content...");
    
    // Generate bypass list check
    let bypass_list = if config.bypass_list.is_empty() {
        "false".to_string()
    } else {
        config.bypass_list
            .iter()
            .filter(|item| item.enabled)
            .map(|item| format!("host === '{}'", item.host))
            .collect::<Vec<_>>()
            .join(" || ")
    };

    // Generate proxy rules
    let proxy_rules = config.proxy_rules
        .iter()
        .filter(|item| item.enabled)
        .map(|item| {
            if item.rule.host == "*" {
                format!("return 'PROXY {}:{}';", item.rule.proxy_host, item.rule.proxy_port)
            } else {
                format!("if (host == '{}') return 'PROXY {}:{}';", 
                    item.rule.host, item.rule.proxy_host, item.rule.proxy_port)
            }
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    // Generate external PAC functions
    let external_functions = config.external_pac_functions
        .iter()
        .filter(|item| item.enabled)
        .map(|item| item.function.function_text.clone())
        .collect::<Vec<_>>()
        .join("\n\n");

    // Generate external PAC function calls
    let external_calls = config.external_pac_functions
        .iter()
        .filter(|item| item.enabled)
        .map(|item| {
            format!(
                "    // Try external PAC function from {}\n    const result{} = {}(url, host);\n    if (!isEmptyStringSafe(result{})) return result{};",
                item.function.original_url,
                item.function.function_name, 
                item.function.function_name,
                item.function.function_name,
                item.function.function_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let content = format!(
        r#"
function FindProxyForURL(url, host) {{

    function isEmptyStringSafe(str) {{
        // Handle null/undefined
        if (str == null) return true;
        // Handle non-string types
        if (typeof str !== 'string') return true;
        return str.length === 0;
    }}

    // All external PAC functions first
    {}
    
    // Bypass list - URLs matching these patterns will bypass the proxy
    if ({}) {{
        return "DIRECT";
    }}
    
    // Try external PAC functions
    {}

    // Proxy rules - check each rule against the host
    {}
    
    // Default to direct connection if no rules match
    return "DIRECT";
}}"#,
        external_functions,
        bypass_list,
        external_calls,
        proxy_rules
    );

    debug!("Generated PAC file content with {} proxy rules and {} external PAC functions", 
        config.proxy_rules.len(),
        config.external_pac_functions.len()
    );
    content
}

/// Creates a shared PAC configuration by reading Windows proxy settings
/// This function initializes the PAC configuration from the current Windows
/// proxy settings and wraps it in a thread-safe shared structure
pub fn create_shared_config() -> SharedPacConfig {
    info!("Creating shared PAC configuration...");
    
    // Try to load existing configuration
    let pac_config = match PacConfig::load_from_file(APP_CONFIG.config_file) {
        Ok(config) => {
            info!("Loaded existing configuration from file");
            config
        }
        Err(e) => {
            warn!("Could not load configuration file: {}", e);
            info!("Creating new configuration from Windows settings");
            
            // Create new config from Windows settings
            let windows_config = ProxyConfig::from_windows().unwrap_or_else(|e| {
                error!("Failed to read Windows proxy settings: {}", e);
                info!("Using empty proxy configuration");
                ProxyConfig::new()
            });
            
            let config = PacConfig::from_windows_config(&windows_config);
            
            // Save the new configuration
            if let Err(e) = config.save_to_file(APP_CONFIG.config_file) {
                error!("Failed to save initial configuration: {}", e);
            }
            
            config
        }
    };

    info!("Created shared PAC configuration");
    Arc::new(RwLock::new(pac_config))
} 