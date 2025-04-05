# ProxyCat

A lightweight Windows proxy management tool.

## Description

ProxyCat is a user-friendly proxy management tool designed to simplify the experience of accessing various resources that require different proxy configurations. It provides a seamless way to manage and switch between multiple proxy settings without the complexity of manual configuration.

### Motivation

Many users, especially those working with governmental resources, often need to access different systems that require specific proxy configurations. Manually switching between these proxies can be time-consuming and error-prone. ProxyCat was developed to address this challenge by providing:

- Easy proxy configuration management through a simple web interface
- Automatic switching between different proxy settings
- System tray integration for convenient access
- Minimal resource footprint while running in the background

## Requirements

- Rust 1.75 or later
- Windows 7 or later

## Installation

1. Install Rust from https://rustup.rs/
2. Clone this repository
3. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

1. Run the application:
   ```bash
   cargo run --release
   ```
2. The application will start and show an icon in the system tray
3. Double-click the tray icon or use the context menu to open the web interface
4. The web interface will be available at http://localhost:12112

## Features

- System tray icon with context menu
- Web interface accessible through the tray icon
- Minimal executable size
- Windows compatibility

## Building for Distribution

To create a minimal executable:

```bash
cargo build --release
```

The executable will be located in `target/release/proxycat.exe` 