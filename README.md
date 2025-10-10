# PathFinder 🚀

A powerful Raycast-inspired launcher application built with **Tauri** and **React**. PathFinder provides instant access to your most-used tools and information through a beautiful, keyboard-driven interface.

![PathFinder Demo](https://img.shields.io/badge/Status-In%20Development-yellow) ![Platform](https://img.shields.io/badge/Platform-Windows-blue) ![License](https://img.shields.io/badge/License-MIT-green)

## ✨ Features

### 🎯 Current Features

- **🌐 Global Hotkey Access** - Press `Ctrl + Shift + Space` from anywhere to launch
- **⌨️ Keyboard Navigation** - Full keyboard control with arrow keys and Enter
- **🔍 Fuzzy Search** - Intelligent search using Fuse.js for all options
- **📋 Clipboard Management** - View and search through clipboard history
- **🌍 Online Search** - Quick access to web search functionality
- **📁 File System Search** - Find and open files quickly
- **🎨 Beautiful UI** - Modern, transparent overlay with blur effects
- **⚡ Lightning Fast** - Built with Tauri for native performance
- **🔄 Hot Reload** - Instant development feedback

### 🚀 Planned Features (Raycast-inspired)

- **📋 Advanced Clipboard** - Full clipboard history with rich content support
- **🔍 Deep File Search** - Index and search entire file system
- **🌐 Smart Web Search** - Multiple search engines and instant results
- **📱 App Launcher** - Launch applications and system tools
- **⚙️ System Controls** - Volume, brightness, WiFi, Bluetooth controls
- **📊 Calculator** - Built-in calculator with expression evaluation
- **📅 Calendar Integration** - View and manage calendar events
- **🎵 Media Controls** - Control music and video playback
- **📝 Quick Notes** - Create and search notes instantly
- **🔗 URL Shortcuts** - Custom URL schemes and deep links
- **🎨 Themes** - Multiple UI themes and customization options
- **🔌 Extensions** - Plugin system for custom functionality
- **📈 Usage Analytics** - Track most-used commands and features
- **🌙 Dark/Light Mode** - Automatic theme switching
- **🔐 Security** - Secure credential storage and management

## 🛠️ Tech Stack

- **Frontend**: React 18 + Vite
- **Backend**: Tauri 2.0 (Rust)
- **Search**: Fuse.js for fuzzy search
- **Styling**: CSS3 with modern features
- **Build**: Vite + Tauri CLI
- **Platform**: Cross-platform (Windows, macOS, Linux)

## 📋 Prerequisites

Before running PathFinder, ensure you have:

- **Node.js** (v16 or higher) - [Download here](https://nodejs.org/)
- **Rust** (latest stable) - [Install here](https://rustup.rs/)
- **Git** - [Download here](https://git-scm.com/)

### Windows Additional Requirements
- Microsoft Visual Studio C++ Build Tools
- Windows SDK

### macOS Additional Requirements
- Xcode Command Line Tools: `xcode-select --install`

### Linux Additional Requirements
- `libwebkit2gtk-4.0-dev`
- `build-essential`
- `curl`
- `wget`
- `libssl-dev`
- `libgtk-3-dev`
- `libayatana-appindicator3-dev`
- `librsvg2-dev`

## 🚀 Quick Start

### 1. Clone the Repository
```bash
git clone https://github.com/yourusername/pathfinder.git
cd pathfinder
```

### 2. Install Dependencies
```bash
npm install
```

### 3. Run in Development Mode
```bash
npm run tauri dev
```

### 4. Build for Production
```bash
npm run tauri build
```

## 🎮 How to Use

### Global Hotkey
- **Launch PathFinder**: `Ctrl + Shift + Space` (from anywhere)
- **Hide PathFinder**: `Escape` or `Ctrl + Shift + Space` again

### Navigation
- **Arrow Keys**: Navigate up/down through options
- **Enter**: Select highlighted option
- **Escape**: Go back to home or hide application
- **Type**: Start typing to search/filter options

### Available Commands

#### 🏠 Home Screen
- **Clipboard** - Access clipboard history
- **Online Search** - Search the web
- **Open File** - Find and open files

#### 📋 Clipboard Management
- View recent clipboard items
- Search through clipboard history
- Click or press Enter to copy item back to clipboard

#### 🌍 Online Search
- Type your search query
- Press Enter to open search in default browser
- Supports all major search engines

#### 📁 File System Search
- Search through files by name
- Fuzzy matching for partial names
- Quick file opening

## 🏗️ Development

### Project Structure
```
pathfinder/
├── src/                    # React frontend
│   ├── components/         # React components
│   │   ├── HomeOptions.jsx
│   │   ├── ClipboardPage.jsx
│   │   ├── OnlineSearchPage.jsx
│   │   └── OpenFilePage.jsx
│   ├── hooks/             # Custom React hooks
│   │   └── useKeyboardNavigation.js
│   ├── App.jsx            # Main app component
│   └── App.css            # Styles
├── src-tauri/             # Tauri backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   └── lib.rs         # Main logic
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
└── package.json           # Node.js dependencies
```

### Available Scripts

- `npm run dev` - Start Vite development server
- `npm run tauri dev` - Run Tauri app in development mode
- `npm run tauri build` - Build production app
- `npm run build` - Build frontend only
- `npm run preview` - Preview production build

### Development Workflow

1. **Frontend Changes**: Edit files in `src/` - changes reflect immediately
2. **Backend Changes**: Edit Rust files in `src-tauri/src/` - app restarts automatically
3. **Configuration**: Modify `src-tauri/tauri.conf.json` for app settings

### Adding New Features

1. **New Page**: Create component in `src/components/`
2. **Add to Home**: Update `OPTIONS` array in `HomeOptions.jsx`
3. **Navigation**: Add route in `App.jsx`
4. **Styling**: Update `App.css` for new components

## 🔧 Configuration

### Global Shortcut
Modify the global shortcut in `src-tauri/src/lib.rs`:
```rust
let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
```

### Window Settings
Adjust window properties in `src-tauri/tauri.conf.json`:
- Size: `width` and `height`
- Position: `center` and `alwaysOnTop`
- Appearance: `transparent` and `decorations`

### Search Settings
Modify search behavior in component files:
- Threshold: `threshold: 0.4` (lower = more strict)
- Keys: `keys: ["title"]` (searchable fields)

## 🐛 Troubleshooting

### Common Issues

**App won't start:**
- Ensure Rust is installed: `rustc --version`
- Check Node.js version: `node --version`
- Try clearing cache: `npm run tauri dev -- --verbose`

**Global shortcut not working:**
- Check if another app is using the same shortcut
- Try running as administrator (Windows)
- Verify shortcut registration in console

**Build fails:**
- Update dependencies: `npm update`
- Clean build: `npm run tauri build -- --verbose`
- Check system requirements

**Performance issues:**
- Close other applications
- Check available memory
- Update graphics drivers

### Debug Mode
Run with verbose logging:
```bash
npm run tauri dev -- --verbose
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Test thoroughly
5. Commit: `git commit -m 'Add amazing feature'`
6. Push: `git push origin feature/amazing-feature`
7. Open a Pull Request

## 🙏 Acknowledgments

- Inspired by [Raycast](https://raycast.com/) for macOS
- Built with [Tauri](https://tauri.app/) framework
- Uses [Fuse.js](https://fusejs.io/) for fuzzy search

**Made with ❤️ for productivity enthusiasts**