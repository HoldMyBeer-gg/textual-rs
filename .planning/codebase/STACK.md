# Technology Stack

**Analysis Date:** 2026-03-24

## Languages

**Primary:**
- Python 3.9+ (minimum 3.9, tested up to 3.14)
  - Core framework, widgets, event system, rendering engine
  - Type hints required throughout codebase

## Runtime

**Environment:**
- Python 3.9, 3.10, 3.11, 3.12, 3.13, 3.14 (development status: Production/Stable)
- asyncio for asynchronous event loop and task management

**Package Manager:**
- Poetry (poetry-core >= 1.2.0)
- Lockfile: `poetry.lock` (285KB+, comprehensive dependency pinning)

## Frameworks

**Core TUI Framework:**
- textual 8.1.1 - Modern Text User Interface framework for Python
  - DOM-based architecture for widgets
  - CSS-based styling engine (custom parser)
  - Event-driven architecture with message pump system
  - Reactive properties system for state management

**Terminal Rendering:**
- rich >= 14.2.0 - Primary rendering and console output library
  - Used for: segment-based rendering, ANSI styling, console theme management
  - Classes used: `Console`, `ConsoleOptions`, `RenderableType`, `Segment`, `Style`, `Control`, `TerminalTheme`
  - Located in: `src/textual/_compositor.py`, `src/textual/content.py`, throughout app.py

**Markdown Processing:**
- markdown-it-py >= 2.1.0 (with linkify extra)
  - Markdown parsing for `Markdown` widget (`src/textual/widgets/_markdown.py`)
  - Token-based parsing, extensible via plugins
- mdit-py-plugins - Extensions for markdown-it-py

**Syntax Highlighting & Code:**
- tree-sitter >= 0.25.0 (optional, requires Python 3.10+)
  - Multi-language syntax highlighting support
  - Language support: Python, Markdown, JSON, TOML, YAML, HTML, CSS, JavaScript, Rust, Go, Regex, XML, SQL, Java, Bash
  - Each language has corresponding tree-sitter-{language} package (>= 0.23.0)
  - Used in: `src/textual/document/_syntax_aware_document.py`, syntax highlighting system
  - Requires binary wheels from upstream (pre-built for Windows, MacOS, Linux)

- pygments >= 2.19.2 - Fallback syntax highlighting library
  - Lexer-based highlighting via `pygments.lexer`, `pygments.lexers`

**Markup & Content:**
- typing-extensions >= 4.4.0 - Backported type features for Python 3.9 compatibility

**Platform Utilities:**
- platformdirs >= 3.6.0, < 5 - Cross-platform user directory resolution
  - Used for file paths in `src/textual/app.py`

## Key Dependencies

**Critical - Always Installed:**
- markdown-it-py - Required for Markdown widget functionality
- rich - Essential for terminal rendering and styling
- platformdirs - File system integration
- typing-extensions - Type system compatibility

**Optional - Feature-Specific:**
- tree-sitter ecosystem - Only loaded when syntax highlighting is needed
  - Python 3.10+ only (requires type annotations enhancements unavailable in 3.9)
  - Installed via `syntax` extra: `pip install textual[syntax]`

**Development Dependencies:**
- pytest >= 8.3.1 - Testing framework
- pytest-asyncio - Async test support
- pytest-xdist >= 3.6.1 - Parallel test execution
- pytest-cov >= 5.0.0 - Coverage reporting
- pytest-textual-snapshot >= 1.0.0 - Snapshot testing for TUI outputs
- mypy >= 1.0.0 - Type checking
- black 24.4.2 - Code formatting
- isort >= 5.13.2 - Import sorting
- textual-dev >= 1.7.0 - Development utilities
- mkdocs >= 1.3.0, mkdocs-material >= 9.0.11 - Documentation generation
- griffe 0.32.3 - API documentation extraction
- httpx >= 0.23.1 - HTTP client for testing
- pre-commit >= 2.13.0 - Pre-commit hooks

## Configuration

**Build Configuration:**
- `pyproject.toml` - Poetry-based package definition
  - Package source location: `src/textual/`
  - Includes type stub marker: `src/textual/py.typed`
  - Examples directory included in distribution (sdist format only)

**Environment Detection:**
- Platform detection via `sys.platform` (Windows, Linux/MacOS differentiation)
- TTY detection via `isatty()` calls on stdin/stdout
- Terminal size detection via system calls (`stty` on POSIX, Win32 API on Windows)

**Type Checking Configuration:**
- `mypy.ini` present for strict type checking

**Ruff Configuration:**
- Target version: Python 3.9

**Pytest Configuration:**
- `asyncio_mode = "auto"` - Automatic fixture loop scope management
- Test paths: `tests/`
- Custom marker: `syntax` - Marks tests requiring syntax highlighting (optional skip)
- Markers enforced with `--strict-markers`

## Platform Requirements

**Development:**
- Python 3.9+ (interpreter)
- POSIX-compliant terminal for Linux/MacOS (termios, tty modules)
- Windows 10+ with VT100 emulation support for Windows
- FFI support via ctypes (no external C compiler required for base package)

**Production:**
- Python 3.9+ runtime
- Terminal emulator with:
  - ANSI escape sequence support
  - Mouse event reporting (optional)
  - Alt-screen buffer (optional)
  - Bracketed paste mode support (optional)
- Platform-specific console APIs:
  - Linux/MacOS: POSIX terminal interface (termios, signal handling, file descriptors)
  - Windows: Win32 Console API (via ctypes + kernel32.dll calls)
  - Web: Remote driver for headless/web deployment

**Deployment Targets:**
- Linux (x86_64, ARM)
- macOS (Intel, Apple Silicon)
- Windows 10, 11
- Web-based terminals (via WebDriver)
- Headless environments (via HeadlessDriver)

---

*Stack analysis: 2026-03-24*
