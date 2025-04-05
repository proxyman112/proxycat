use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::LazyLock;

/// Application-wide constants
pub struct AppConfig {
    /// The host address for the local HTTP server
    pub host: &'static LazyLock<Arc<Mutex<String>>>,
    /// The port number for the local HTTP server
    pub port: &'static AtomicU16,
    /// The path to the master PAC file
    pub master_pac_path: &'static LazyLock<Arc<Mutex<String>>>,
    /// The full URL for the master PAC file
    pub master_pac_url: &'static LazyLock<Arc<Mutex<String>>>,
    /// The path to the configuration file
    pub config_file: &'static str,
}

static PORT: AtomicU16 = AtomicU16::new(12112);
static DEFAULT_HOST: &str = "127.0.0.1";
static DEFAULT_PAC_PATH: &str = "/master.pac";

static HOST: LazyLock<Arc<Mutex<String>>> = LazyLock::new(|| Arc::new(Mutex::new(DEFAULT_HOST.to_string())));
static PAC_PATH: LazyLock<Arc<Mutex<String>>> = LazyLock::new(|| Arc::new(Mutex::new(DEFAULT_PAC_PATH.to_string())));
static MASTER_PAC_URL: LazyLock<Arc<Mutex<String>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(format!("http://{}:{}{}", DEFAULT_HOST, PORT.load(Ordering::SeqCst), DEFAULT_PAC_PATH)))
});

/// Global application configuration
pub static APP_CONFIG: AppConfig = AppConfig {
    host: &HOST,
    port: &PORT,
    master_pac_path: &PAC_PATH,
    master_pac_url: &MASTER_PAC_URL,
    config_file: "proxycat_config.json",
};

impl AppConfig {
    /// Updates the port number and returns the new PAC URL
    pub fn update_port(&self, new_port: u16) -> String {
        self.port.store(new_port, Ordering::SeqCst);
        let host = self.host.lock().unwrap();
        let pac_path = self.master_pac_path.lock().unwrap();
        let new_url = format!("http://{}:{}{}", host, new_port, pac_path);
        *self.master_pac_url.lock().unwrap() = new_url.clone();
        new_url
    }

    /// Gets the current port number
    pub fn get_port(&self) -> u16 {
        self.port.load(Ordering::SeqCst)
    }

    /// Updates the host and returns the new PAC URL
    pub fn update_host(&self, new_host: String) -> String {
        let port = self.get_port();
        let pac_path = self.master_pac_path.lock().unwrap();
        let new_url = format!("http://{}:{}{}", new_host, port, pac_path);
        *self.host.lock().unwrap() = new_host;
        *self.master_pac_url.lock().unwrap() = new_url.clone();
        new_url
    }

    /// Gets the current host
    pub fn get_host(&self) -> String {
        self.host.lock().unwrap().clone()
    }

    /// Updates the PAC path and returns the new PAC URL
    pub fn update_pac_path(&self, new_path: String) -> String {
        let host = self.host.lock().unwrap();
        let port = self.get_port();
        let new_url = format!("http://{}:{}{}", host, port, new_path);
        *self.master_pac_path.lock().unwrap() = new_path;
        *self.master_pac_url.lock().unwrap() = new_url.clone();
        new_url
    }

    /// Gets the current PAC path
    pub fn get_pac_path(&self) -> String {
        self.master_pac_path.lock().unwrap().clone()
    }

    /// Gets the current PAC URL
    pub fn get_pac_url(&self) -> String {
        self.master_pac_url.lock().unwrap().clone()
    }
} 