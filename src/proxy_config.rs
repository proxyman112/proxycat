use windows::Win32::Networking::WinInet::{
    InternetQueryOptionW,
    InternetSetOptionW,
    INTERNET_OPTION_PER_CONNECTION_OPTION,
    INTERNET_PER_CONN_OPTION_LISTW,
    INTERNET_PER_CONN_OPTIONW,
    INTERNET_PER_CONN_PROXY_SERVER,
    INTERNET_PER_CONN_PROXY_BYPASS,
    INTERNET_PER_CONN_AUTOCONFIG_URL,
    INTERNET_OPTION_PROXY_SETTINGS_CHANGED,
    INTERNET_OPTION_REFRESH,
    INTERNET_OPTION_SETTINGS_CHANGED,
};
use windows::core::PWSTR;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use log::{info, error, warn, debug};
use crate::error::{Result, ProxyCatError};

/// Represents the Windows proxy configuration settings
/// This struct holds the proxy server address, bypass list, and auto-config URL
/// along with a flag indicating whether proxy is enabled
#[derive(Debug)]
pub struct ProxyConfig {
    /// The proxy server address in the format "host:port"
    pub proxy_server: Option<String>,
    /// Semicolon-separated list of addresses to bypass the proxy
    pub proxy_bypass: Option<String>,
    /// URL for automatic proxy configuration (PAC file)
    pub auto_config_url: Option<String>,
    /// Whether proxy is currently enabled
    pub use_proxy: bool,
}

impl ProxyConfig {
    /// Creates a new empty proxy configuration
    pub fn new() -> Self {
        info!("Creating new empty proxy configuration");
        Self {
            proxy_server: None,
            proxy_bypass: None,
            auto_config_url: None,
            use_proxy: false,
        }
    }

    /// Reads the current proxy configuration from Windows settings
    /// This function uses the Windows API to query the system's proxy settings
    /// Returns a Result containing either the ProxyConfig or an error
    pub fn from_windows() -> Result<Self> {
        info!("Reading proxy configuration from Windows settings...");
        unsafe {
            let mut config = Self::new();
            let mut option_list = INTERNET_PER_CONN_OPTION_LISTW::default();
            let mut options = [
                INTERNET_PER_CONN_OPTIONW::default(),
                INTERNET_PER_CONN_OPTIONW::default(),
                INTERNET_PER_CONN_OPTIONW::default(),
            ];

            // Set up the option list structure
            option_list.dwSize = std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32;
            option_list.dwOptionCount = 3;
            option_list.dwOptionError = 0;
            option_list.pOptions = options.as_mut_ptr();

            // Configure which options we want to query
            options[0].dwOption = INTERNET_PER_CONN_PROXY_SERVER;
            options[1].dwOption = INTERNET_PER_CONN_PROXY_BYPASS;
            options[2].dwOption = INTERNET_PER_CONN_AUTOCONFIG_URL;

            // Query the Windows API for proxy settings
            if InternetQueryOptionW(
                None,
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                Some(&mut option_list as *mut _ as *mut _),
                &mut (std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32),
            ).is_ok() {
                info!("Successfully queried Windows proxy settings");
                
                // Extract proxy server address
                if !options[0].Value.pszValue.is_null() {
                    config.proxy_server = Some(wide_to_string(options[0].Value.pszValue.0));
                    debug!("Found proxy server: {:?}", config.proxy_server);
                }

                // Extract proxy bypass list
                if !options[1].Value.pszValue.is_null() {
                    config.proxy_bypass = Some(wide_to_string(options[1].Value.pszValue.0));
                    debug!("Found proxy bypass list: {:?}", config.proxy_bypass);
                }

                // Extract auto-config URL
                if !options[2].Value.pszValue.is_null() {
                    config.auto_config_url = Some(wide_to_string(options[2].Value.pszValue.0));
                    debug!("Found auto-config URL: {:?}", config.auto_config_url);
                }

                // Determine if proxy is enabled
                config.use_proxy = config.proxy_server.is_some() || config.auto_config_url.is_some();
                info!("Proxy enabled: {}", config.use_proxy);
            } else {
                warn!("No proxy settings found or error occurred while querying");
            }

            Ok(config)
        }
    }

    /// Sets the Windows proxy configuration to use a PAC file
    /// This function configures Windows to use the specified PAC file URL
    pub fn set_pac_file(pac_url: &str) -> Result<()> {
        info!("Setting Windows proxy configuration to use PAC file: {}", pac_url);
        unsafe {
            let mut option_list = INTERNET_PER_CONN_OPTION_LISTW::default();
            let mut options = [INTERNET_PER_CONN_OPTIONW::default()];

            // Set up the option list structure
            option_list.dwSize = std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32;
            option_list.dwOptionCount = 1;
            option_list.dwOptionError = 0;
            option_list.pOptions = options.as_mut_ptr();

            // Configure option for PAC file
            options[0].dwOption = INTERNET_PER_CONN_AUTOCONFIG_URL;
            let mut wide_url: Vec<u16> = pac_url.encode_utf16().chain(std::iter::once(0)).collect();
            options[0].Value.pszValue = PWSTR::from_raw(wide_url.as_mut_ptr());
            
            // Set the proxy configuration
            if InternetSetOptionW(
                None,
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                Some(&option_list as *const _ as *const _),
                std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
            ).is_ok() {
                info!("Successfully set PAC file configuration");

                // Notify Windows that proxy settings have changed
                let _ = InternetSetOptionW(None, INTERNET_OPTION_PROXY_SETTINGS_CHANGED, None, 0);
                let _ = InternetSetOptionW(None, INTERNET_OPTION_REFRESH, None, 0);
                let _ = InternetSetOptionW(None, INTERNET_OPTION_SETTINGS_CHANGED, None, 0);

                info!("Successfully notified Windows of proxy settings change");
                Ok(())
            } else {
                error!("Failed to set PAC file configuration");
                Err(ProxyCatError::Windows("Failed to set PAC file configuration".to_string()))
            }
        }
    }

    /// Gets the current PAC file URL from Windows settings
    pub fn get_pac_file() -> Result<String> {
        info!("Getting current PAC file URL from Windows settings...");
        unsafe {
            let mut option_list = INTERNET_PER_CONN_OPTION_LISTW::default();
            let mut options = [INTERNET_PER_CONN_OPTIONW::default()];

            // Set up the option list structure
            option_list.dwSize = std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32;
            option_list.dwOptionCount = 1;
            option_list.dwOptionError = 0;
            option_list.pOptions = options.as_mut_ptr();

            // Configure option for PAC file
            options[0].dwOption = INTERNET_PER_CONN_AUTOCONFIG_URL;

            // Query the Windows API for PAC file URL
            if InternetQueryOptionW(
                None,
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                Some(&mut option_list as *mut _ as *mut _),
                &mut (std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32),
            ).is_ok() {
                if !options[0].Value.pszValue.is_null() {
                    let url = wide_to_string(options[0].Value.pszValue.0);
                    info!("Found PAC file URL: {}", url);
                    Ok(url)
                } else {
                    info!("No PAC file URL found");
                    Ok(String::new())
                }
            } else {
                error!("Failed to query PAC file URL");
                Err(ProxyCatError::Windows("Failed to query PAC file URL".to_string()))
            }
        }
    }
}

/// Converts a wide string pointer to a Rust String
/// This is used to convert Windows API wide string responses to Rust strings
fn wide_to_string(ptr: *const u16) -> String {
    unsafe {
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(ptr, len);
        OsString::from_wide(slice).to_string_lossy().into_owned()
    }
} 