# External Integrations

**Analysis Date:** 2026-03-24

## APIs & External Services

**None Detected**

Textual does not integrate with external web APIs or cloud services. It is a self-contained TUI framework.

## Data Storage

**Databases:**
- None - Textual is a UI framework, not a data persistence layer
- Applications using Textual may integrate their own databases (not built-in)

**File Storage:**
- Local filesystem only
  - User downloads path via `platformdirs.user_downloads_path()` in `src/textual/app.py`
  - Configuration directory resolution via platformdirs (cross-platform)
  - No cloud storage integrations

**Caching:**
- Internal caching: `src/textual/_styles_cache.py` - In-memory CSS style cache
- No external cache backends (Redis, Memcached, etc.)

## Terminal Rendering Backends

**POSIX (Linux/macOS):**
- Backend: Direct terminal control via file descriptors and termios
- Driver: `src/textual/drivers/linux_driver.py`
- Protocols:
  - Input: Raw input mode via termios/tty modules
  - Output: ANSI escape sequences to stderr
  - TTY detection via `isatty()` system call
  - Mouse support: XTerm mouse protocol (SGR extended mode)
  - Terminal size: SIGWINCH signal handling + `stty size` equivalent
  - Signal handling: SIGTSTP/SIGCONT for suspend/resume

**Windows:**
- Backend: Native Windows Console API (Win32) via ctypes
- Driver: `src/textual/drivers/windows_driver.py`
- Console DLL: `kernel32.dll` (loaded via `ctypes.WinDLL`)
- Modes Controlled:
  - Input: `ENABLE_VIRTUAL_TERMINAL_INPUT`, `ENABLE_MOUSE_INPUT`, `ENABLE_EXTENDED_FLAGS`
  - Output: `ENABLE_VIRTUAL_TERMINAL_PROCESSING` (for ANSI support)
  - Console structures: `KEY_EVENT_RECORD`, `MOUSE_EVENT_RECORD`, `INPUT_RECORD` (ctypes structures)
  - File handles: `STD_INPUT_HANDLE`, `STD_OUTPUT_HANDLE` constants
  - Mode configuration via `SetConsoleMode()` Win32 API

**Web/Remote:**
- Driver: `src/textual/drivers/web_driver.py`
- Protocol: Binary packet format (custom)
  - Packet structure: 1 byte type ("D"=data, "M"=meta) + 4-byte little-endian length + payload
  - Used for: Remote text terminal communication (textual-web bridge)
  - Input handling: Remoting input events from browser/remote terminal

**Headless:**
- Driver: `src/textual/drivers/headless_driver.py`
- Use: Testing, CI/CD pipelines, snapshot testing
- No actual terminal output (in-memory rendering)

## Syntax Highlighting Integration

**Tree-Sitter (Python 3.10+):**
- Tree-Sitter Core: `tree-sitter >= 0.25.0` (optional)
- Language Grammars (all optional, Python 3.10+ only):
  - `tree-sitter-python >= 0.23.0`
  - `tree-sitter-markdown >= 0.3.0`
  - `tree-sitter-json >= 0.24.0`
  - `tree-sitter-toml >= 0.6.0`
  - `tree-sitter-yaml >= 0.6.0`
  - `tree-sitter-html >= 0.23.0`
  - `tree-sitter-css >= 0.23.0`
  - `tree-sitter-javascript >= 0.23.0`
  - `tree-sitter-rust >= 0.23.0`
  - `tree-sitter-go >= 0.23.0`
  - `tree-sitter-regex >= 0.24.0`
  - `tree-sitter-xml >= 0.7.0`
  - `tree-sitter-sql >= 0.3.11`
  - `tree-sitter-java >= 0.23.0`
  - `tree-sitter-bash >= 0.23.0`
- Location: `src/textual/document/_syntax_aware_document.py`
- Features:
  - Syntax tree querying via Tree-Sitter Query language
  - Language detection and parser management
  - Incremental tree updates on document changes
  - Byte offset conversion for terminal cell positions

**Pygments (Fallback):**
- Library: `pygments >= 2.19.2`
- Used in: `src/textual/highlight.py`
- Features:
  - Lexer-based syntax highlighting
  - Lexer discovery: `get_lexer_by_name()`, `guess_lexer_for_filename()`
  - Token types from `pygments.token.Token`

## Markdown Processing

**markdown-it-py:**
- Version: `>= 2.1.0` (with `linkify` extra enabled)
- Location: `src/textual/widgets/_markdown.py`
- Features:
  - Token stream parsing (produces `Token` objects)
  - Link detection and processing (via linkify)
  - Extensible plugin system
- Dependencies: `mdit-py-plugins` (optional extensions)

## Rich Library Integration

**Core Integration Points:**
- Location: Deep integration throughout entire codebase
- Rendering pipeline: `src/textual/_compositor.py`, `src/textual/content.py`
- Classes leveraged:
  - `Console` - Main rendering target
  - `Segment` - Atomic rendering unit
  - `Style` - Text styling (colors, attributes)
  - `Control` - Cursor movement and terminal control
  - `TerminalTheme` - Terminal color theme detection
  - `Text` - Rich text objects with styles
  - `RenderableType` - Protocol for renderable objects

**Rich Renderables Used:**
- `Panel`, `Table`, `Pretty`, `Traceback` (in app.py and widgets)
- Text layout and wrapping via `rich._wrap.divide_line`
- Cell width calculation via `rich.cells.set_cell_size`

## Authentication & Identity

**Auth Provider:**
- None - Textual is a UI framework without built-in authentication
- Applications using Textual handle authentication separately

## Monitoring & Observability

**Error Tracking:**
- None - No external error tracking integration

**Logs:**
- Internal logging: `src/textual/_log.py`
- Uses Python standard logging (not explicitly visible in imports, but infrastructure exists)
- Debug mode available at driver level

## Input Handling

**Keyboard:**
- XTerm parser: `src/textual/drivers/_xterm_parser.py`
- Kitty keyboard protocol support (via `\x1b[>1u` escape sequence)
- Input readers:
  - Linux: `src/textual/drivers/_input_reader_linux.py` (selector-based)
  - Windows: `src/textual/drivers/_input_reader_windows.py` (Win32 API)
  - Generic: `src/textual/drivers/_input_reader.py`

**Mouse:**
- Protocols supported:
  - VT200 Mouse (`SET_VT200_MOUSE` - `\x1b[?1000h`)
  - Any Event Mouse (`SET_ANY_EVENT_MOUSE` - `\x1b[?1003h`)
  - VT200 Highlight Mouse (`SET_VT200_HIGHLIGHT_MOUSE` - `\x1b[?1015h`)
  - SGR Extended Mode (`SET_SGR_EXT_MODE_MOUSE` - `\x1b[?1006h`)
- Implementation: Windows driver enables all modes for maximum compatibility

**Paste:**
- Bracketed paste mode (`\x1b[?2004h`) for safe paste detection

## ANSI Escape Sequence Support

**Features Enabled:**
- Alt-screen buffer (`\x1b[?1049h`)
- Cursor visibility control (`\x1b[?25l` hide, `\x1b[?25h` show)
- Focus in/focus out events (`\033[?1004h`)
- Kitty keyboard protocol (`\x1b[>1u`)
- Mouse reporting (multiple protocols)
- Bracketed paste mode

**Output Mode:**
- `ENABLE_VIRTUAL_TERMINAL_PROCESSING` on Windows (enables ANSI processing)
- ANSI escape sequences on POSIX systems (native support)

## Thread Management

**Writer Thread:**
- Class: `src/textual/drivers/_writer_thread.py`
- Purpose: Queued output writing to prevent blocking
- Used by: All drivers (Linux, Windows, Web)

**Input Thread:**
- Platform-specific input readers running in background threads
- Event loop coordination via asyncio

## Configuration Files & Paths

**No Config File Standard:**
- platformdirs used for standardized user directories
- Applications using Textual define their own configuration

## Webhooks & Callbacks

**Incoming:**
- None - Not a server framework

**Outgoing:**
- None - Not integrated with external services

---

*Integration audit: 2026-03-24*
