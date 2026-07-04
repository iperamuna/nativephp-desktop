# NativePHP for Desktop

[![Latest Version on Packagist](https://img.shields.io/packagist/v/nativephp/desktop.svg?style=flat-square)](https://packagist.org/packages/nativephp/desktop)
[![GitHub Tests Action Status](https://img.shields.io/github/actions/workflow/status/nativephp/desktop/run-tests.yml?branch=main&label=tests&style=flat-square)](https://github.com/nativephp/desktop/actions?query=workflow%3Arun-tests+branch%3Amain)
[![GitHub Code Style Action Status](https://img.shields.io/github/actions/workflow/status/nativephp/desktop/fix-php-code-style-issues.yml?branch=main&label=code%20style&style=flat-square)](https://github.com/nativephp/desktop/actions?query=workflow%3A"Fix+PHP+code+style+issues"+branch%3Amain)
[![Total Downloads](https://img.shields.io/packagist/dt/nativephp/desktop?style=flat-square)](https://packagist.org/packages/nativephp/desktop)

Write native desktop applications using PHP.
To learn more, visit the [official website](https://nativephp.com).

## Tauri Driver (Experimental)

NativePHP now ships with an experimental Rust-based Tauri driver, providing a lightweight, memory-efficient alternative to the default Electron driver.

### Installation

To install and use the Tauri driver for your NativePHP application, run:

```bash
php artisan native:install --driver=tauri
```

This command will install the necessary Rust backend scaffolding into `nativephp/tauri` and automatically set `NATIVE_DRIVER=tauri` in your `.env` file. You can switch back to Electron at any time by changing this variable back to `electron`.

### Features Supported via Tauri Bridge
- **Window Management**: Opening, closing, resizing, and positioning windows.
- **Dynamic Menus**: Full native menu and submenu bridging.
- **System Tray**: Cross-platform system tray integration.
- **Notifications & Clipboard**: Native desktop notifications and clipboard access.

### Custom Tauri Updater Process

Unlike Electron, which uses `electron-updater` tied directly to GitHub releases by NativePHP, Tauri requires an explicit JSON endpoint for its updater. Because this isn't natively absorbed by the NativePHP ecosystem yet, you can implement a custom updater process:

1. **Configure `tauri.conf.json`**:
   Add the following to the `updater` section of your `tauri.conf.json`:
   ```json
   "updater": {
     "active": true,
     "endpoints": [
       "https://your-server.com/api/updater/{{target}}/{{current_version}}"
     ],
     "dialog": true,
     "pubkey": "YOUR_TAURI_UPDATER_PUBLIC_KEY"
   }
   ```
2. **Build and Sign**:
   When you build your Tauri app (`npm run tauri build`), it produces a `.tar.gz`/`.zip` app bundle alongside a `.sig` (signature) file.
3. **Host the Update JSON**:
   Build a simple Laravel route on your live server that returns the required JSON structure:
   ```json
   {
     "version": "1.1.0",
     "notes": "Bug fixes and performance improvements",
     "pub_date": "2023-10-10T12:00:00Z",
     "platforms": {
       "darwin-aarch64": {
         "signature": "CONTENT_OF_SIG_FILE",
         "url": "https://your-server.com/downloads/app-aarch64.tar.gz"
       }
     }
   }
   ```
   Tauri will natively fetch this JSON, verify the signature, and seamlessly prompt the user to restart and install the update.

## Documentation

You can find the NativePHP [documentation on the website](https://nativephp.com).
Check out the [Getting Started](https://nativephp.com/docs/desktop/2/getting-started/introduction) page for a quick overview.
- [Getting Started](https://nativephp.com/docs/desktop/2/getting-started/introduction)
- [Installation](https://nativephp.com/docs/desktop/2/getting-started/installation)
- [Configuration](https://nativephp.com/docs/desktop/2/getting-started/configuration)
- [Application Lifecycle](https://nativephp.com/docs/desktop/2/the-basics/app-lifecycle)
- [Contributing Guide](https://github.com/nativephp/desktop/blob/main/CONTRIBUTING.md)

## Sponsors

Thanks to the following sponsors for funding NativePHP development. Please consider [sponsoring](https://nativephp.com/sponsor).

- [Bifrost](https://bifrost.nativephp.com) - Build your NativePHP Desktop and Mobile apps—for any platform—all in one place.
- [Laradevs](https://laradevs.com/?ref=nativephp-docs) - Connecting the best Laravel Developers with the best Laravel Teams.

## Changelog

Please see [CHANGELOG](CHANGELOG.md) for more information on what has changed recently.

## Contributing

Please see [CONTRIBUTING](CONTRIBUTING.md) for details.

## Security Vulnerabilities

Please review [our security policy](../../security/policy) on how to report security vulnerabilities.

## Credits

- [Simon Hamp](https://github.com/simonhamp)
- [Marcel Pociot](https://github.com/mpociot)
- [All Contributors](../../contributors)

## License

The MIT License (MIT). Please see [License File](LICENSE.md) for more information.
