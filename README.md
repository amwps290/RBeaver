# RBeaver - PostgreSQL Database Management Tool

![Build Status](https://github.com/your-username/rbeaver/workflows/Build%20RBeaver%20(Windows%20&%20Linux)/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

RBeaver is a modern, cross-platform PostgreSQL database management tool built with Rust and GPUI. It provides a native, high-performance interface for managing PostgreSQL databases with a focus on simplicity and efficiency.

## ✨ Features

- 🔗 **PostgreSQL Connection Management**: Secure connection storage with SSL/TLS support
- 🗄️ **Database Schema Explorer**: Browse tables, views, indexes, and database structure
- 📝 **SQL Query Editor**: Execute SQL queries with syntax highlighting (planned)
- 🔄 **Connection Pooling**: High-performance database connections using deadpool-postgres
- 🛡️ **Security First**: Multiple SSL modes and secure credential storage
- 🎨 **Modern UI**: Native interface built with GPUI for optimal performance
- 🚀 **Cross-Platform**: Support for Windows, Linux, and macOS

## 🚀 Quick Start

### Prerequisites

- PostgreSQL server (any version)
- Operating System: Windows 10+, Linux (X11/Wayland), or macOS 10.15+

### Download & Installation

#### From GitHub Releases (Recommended)

1. Visit the [Releases page](https://github.com/your-username/rbeaver/releases)
2. Download the appropriate binary for your platform:
   - **Linux**: `rbeaver-linux-x86_64`
   - **Windows**: `rbeaver-windows-x86_64.exe`
3. Follow platform-specific installation instructions below

#### Linux Installation

```bash
# Download the binary
wget https://github.com/your-username/rbeaver/releases/latest/download/rbeaver-linux-x86_64

# Make it executable
chmod +x rbeaver-linux-x86_64

# Run the application
./rbeaver-linux-x86_64
```

#### Windows Installation

1. Download `rbeaver-windows-x86_64.exe`
2. Double-click to run (Windows Defender may show a warning for unsigned binaries)

### First Run

1. Click **"New Connection"** in the File menu
2. Enter your PostgreSQL connection details:
   - **Host**: Your PostgreSQL server address
   - **Port**: Usually 5432
   - **Database**: Database name to connect to
   - **Username**: Your PostgreSQL username
   - **Password**: Your PostgreSQL password
   - **SSL Mode**: Choose appropriate SSL mode for your setup
3. Click **"Test Connection"** to verify settings
4. Save the connection and start exploring your database

## 🏗️ Building from Source

### System Requirements

- Rust 1.70 or later
- Platform-specific dependencies (see below)

### Linux Dependencies

```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libfontconfig1-dev libfreetype6-dev \
    libxcb-composite0-dev libxcb-damage0-dev libxcb-dpms0-dev libxcb-dri2-0-dev \
    libxcb-glx0-dev libxcb-present-dev libxcb-randr0-dev libxcb-render0-dev \
    libxcb-shape0-dev libxcb-shm0-dev libxcb-sync-dev libxcb-xfixes0-dev \
    libxcb-xinput-dev libxcb-xkb-dev libxcb1-dev libxkbcommon-x11-dev \
    libxkbcommon-dev libwayland-dev libssl-dev libpq-dev

# Fedora/RHEL
sudo dnf install gcc pkg-config fontconfig-devel freetype-devel \
    libxcb-devel libxkbcommon-x11-devel libxkbcommon-devel \
    wayland-devel openssl-devel postgresql-devel
```

### Windows Dependencies

- Visual Studio Build Tools or Visual Studio with C++ support
- Dependencies are automatically handled by cargo

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/your-username/rbeaver.git
cd rbeaver

# Build in release mode
cargo build --release

# Run the application
./target/release/rbeaver
```

### Development Build

```bash
# Build in debug mode
cargo build

# Run with cargo
cargo run

# Run tests
cargo test

# Run database tests (requires PostgreSQL server)
DATABASE_URL=postgresql://username:password@localhost/testdb cargo test --features database-tests
```

## 🔧 Configuration

RBeaver stores connection configurations in your system's config directory:

- **Linux**: `~/.config/rbeaver/connections.json`
- **Windows**: `%APPDATA%\rbeaver\connections.json`
- **macOS**: `~/Library/Application Support/rbeaver/connections.json`

### Connection Configuration

```json
{
  "connections": {
    "connection-id": {
      "name": "My Database",
      "host": "localhost",
      "port": 5432,
      "database": "mydb",
      "username": "myuser",
      "ssl_mode": "Prefer",
      "connection_timeout": 30
    }
  }
}
```

### SSL Modes

- `Disable`: No SSL encryption
- `Allow`: SSL if available
- `Prefer`: Prefer SSL (default)
- `Require`: Require SSL
- `VerifyCa`: Verify certificate authority
- `VerifyFull`: Full certificate verification

## 🚀 GitHub Actions Build

This project includes automated build workflows for easy distribution:

### Manual Build Trigger

Navigate to the **Actions** tab in your GitHub repository and select one of the workflows:

1. **Build RBeaver (Windows & Linux)** - Simple build for Windows and Linux
2. **Build RBeaver (Advanced)** - Full-featured build with more options
3. **Build RBeaver** - Basic build workflow

#### Workflow Options

- **Release Tag**: Optional version tag for creating GitHub releases
- **Build Type**: Choose between `release` (optimized) or `debug`
- **Platforms**: Select which platforms to build for

#### Creating a Release

1. Go to **Actions** → **Build RBeaver (Windows & Linux)**
2. Click **Run workflow**
3. Set **Release Tag** (e.g., `v1.0.0`)
4. Set **Build Type** to `release`
5. Click **Run workflow**

The workflow will build binaries and create a GitHub release with downloadable assets.

## 📊 Database Features

### Supported Operations

- ✅ Database connection management
- ✅ Connection testing and validation
- ✅ Schema exploration (tables, views, indexes)
- ✅ Basic SQL query execution
- 🔄 Advanced query editor (in development)
- 🔄 Data visualization (planned)
- 🔄 Schema comparison (planned)
- 🔄 Data export/import (planned)

### Connection Pooling

RBeaver uses `deadpool-postgres` for efficient connection management:

```rust
// Example connection pool configuration
let pool = connection.create_connection_pool().await?;
let conn = pool.get().await?;
```

### SQL Query Support

- Standard PostgreSQL SQL syntax
- Multiple statement execution
- Query result formatting
- Error handling and display

## 🧪 Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
# Requires PostgreSQL server
export DATABASE_URL=postgresql://postgres:password@localhost/test_db
cargo test --test '*'
```

### Database Functionality Tests

```bash
# Test actual database connections and operations
cargo run --bin rbeaver -- --test-database
```

## 🛠️ Development

### Project Structure

```
rbeaver/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library exports
│   ├── actions.rs           # Menu actions and event handlers
│   ├── database.rs          # Database connection and operations
│   ├── database_test.rs     # Database testing utilities
│   ├── menubar.rs           # Application menu bar
│   ├── mainwindow.rs        # Main application window
│   ├── database_navigator.rs # Database structure browser
│   ├── connection_dialog.rs # Connection configuration dialog
│   ├── statusbar.rs         # Status bar component
│   └── toolbar.rs           # Toolbar component
├── assets/                  # Application assets
├── .github/workflows/       # GitHub Actions workflows
└── target/                  # Build output (generated)
```

### Architecture

RBeaver follows a modular architecture:

- **GPUI Frontend**: Native UI components with GPUI
- **Database Layer**: PostgreSQL integration with multiple client libraries
- **Action System**: Event-driven architecture for UI interactions
- **Configuration Management**: JSON-based configuration storage

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use clippy for linting (`cargo clippy`)
- Write tests for new functionality
- Update documentation as needed

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [GPUI](https://github.com/zed-industries/gpui) - Modern Rust GUI framework
- [tokio-postgres](https://github.com/sfackler/rust-postgres) - PostgreSQL client
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [deadpool](https://github.com/bikeshedder/deadpool) - Connection pooling

## 📞 Support

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/your-username/rbeaver/issues)
- 💡 **Feature Requests**: [GitHub Discussions](https://github.com/your-username/rbeaver/discussions)
- 📚 **Documentation**: [Wiki](https://github.com/your-username/rbeaver/wiki)

## 🗺️ Roadmap

### v1.0.0 (Current)
- ✅ Basic PostgreSQL connection management
- ✅ Database schema browsing
- ✅ Simple SQL query execution
- ✅ Cross-platform builds

### v1.1.0 (Planned)
- 🔄 Enhanced SQL editor with syntax highlighting
- 🔄 Query history and favorites
- 🔄 Table data browser and editor
- 🔄 Export functionality (CSV, JSON, SQL)

### v1.2.0 (Future)
- 🔄 Visual query builder
- 🔄 Database schema comparison
- 🔄 Performance monitoring
- 🔄 Plugin system

---

**Made with ❤️ using Rust and GPUI**