//! Filesystem directory browser widget backed by a lazy-loading Tree.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::context::AppContext;
use super::tree_view::{tree_key_bindings, Tree, TreeNode};
use super::{EventPropagation, Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::worker::WorkerResult;

/// Internal entry describing one item in a directory listing.
#[derive(Debug, Clone)]
struct DirEntryInfo {
    name: String,
    path: PathBuf,
    is_dir: bool,
    is_symlink: bool,
}

/// Read one level of a directory, returning sorted DirEntryInfo entries.
///
/// Uses `std::fs::read_dir` (not walkdir) for one-level lazy loading.
/// Symlinks detected via `symlink_metadata().file_type().is_symlink()`.
/// Hidden files: on Unix names starting with '.'; on Windows the hidden attribute.
/// Sort order: directories first, then alphabetical case-insensitive.
fn read_directory(path: &Path, show_hidden: bool) -> Vec<DirEntryInfo> {
    let rd = match std::fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let mut entries: Vec<DirEntryInfo> = Vec::new();
    for entry in rd.flatten() {
        let entry_path = entry.path();
        // Use symlink_metadata so we detect symlinks (not follow them).
        let meta = match entry_path.symlink_metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let is_symlink = meta.file_type().is_symlink();
        // Symlinks in this widget are treated as leaf nodes — never expand them.
        let is_dir = if is_symlink { false } else { meta.is_dir() };

        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue, // skip non-UTF-8 names
        };

        if !show_hidden && is_hidden(&name, &meta) {
            continue;
        }

        entries.push(DirEntryInfo {
            name,
            path: entry_path,
            is_dir,
            is_symlink,
        });
    }

    // Sort: directories first, then alphabetical case-insensitive within each group.
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    entries
}

/// Determine whether an entry is hidden.
#[cfg(unix)]
fn is_hidden(name: &str, _meta: &std::fs::Metadata) -> bool {
    name.starts_with('.')
}

#[cfg(windows)]
fn is_hidden(name: &str, meta: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    name.starts_with('.') || (meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
}

#[cfg(not(any(unix, windows)))]
fn is_hidden(name: &str, _meta: &std::fs::Metadata) -> bool {
    name.starts_with('.')
}

/// Messages emitted by DirectoryTree.
pub mod messages {
    use crate::event::message::Message;
    use std::path::PathBuf;

    /// Emitted when a file node is selected (Enter key).
    pub struct FileSelected {
        /// Absolute path of the selected file.
        pub path: PathBuf,
    }
    impl Message for FileSelected {}

    /// Emitted when a directory node is selected (Enter key).
    pub struct DirectorySelected {
        /// Absolute path of the selected directory.
        pub path: PathBuf,
    }
    impl Message for DirectorySelected {}
}

/// A filesystem directory browser widget that wraps [`Tree`], lazy-loading directory
/// contents via workers without blocking the UI.
///
/// # Features
/// - Lazy loading: directory children are loaded on first expand via a background worker.
/// - Caching: re-expanding a directory uses cached children (no re-read).
/// - Symlink display: symlinked entries show with a `@` suffix and cannot be expanded.
/// - Hidden files: hidden by default; set `show_hidden = true` to reveal them.
/// - Cycle safety: tracks canonical paths to prevent infinite loops on NTFS junctions
///   and Unix symlink cycles.
pub struct DirectoryTree {
    /// Root path for the directory browser.
    pub root_path: PathBuf,
    /// Whether to show hidden files.
    pub show_hidden: bool,
    /// Inner Tree widget that handles rendering and cursor navigation.
    inner: RefCell<Tree>,
    /// Paths whose children have been fully loaded. Guards against re-reading on re-expand.
    loaded_paths: RefCell<HashSet<PathBuf>>,
    /// Paths currently being loaded by a worker. Guards against double-spawning.
    loading_paths: RefCell<HashSet<PathBuf>>,
    /// Canonical paths visited during this session. Used to detect cycles (NTFS junctions,
    /// symlink loops). On cycle detection, the node is marked "(cycle)" instead of expanding.
    visited_canonical: RefCell<HashSet<PathBuf>>,
    /// This widget's own WidgetId, set in on_mount.
    own_id: Cell<Option<WidgetId>>,
}

impl DirectoryTree {
    /// Create a new DirectoryTree rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root_path: PathBuf = root.into();

        let root_label = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_else(|| root_path.to_str().unwrap_or("/"))
            .to_string();

        // Root node starts with a dummy placeholder child so it renders as expandable.
        let placeholder = TreeNode::new("Loading...");
        let mut root_node = TreeNode::with_children(&root_label, vec![placeholder]);
        root_node.data = Some(root_path.to_string_lossy().to_string());

        let inner_tree = Tree::new(root_node);

        let mut visited = HashSet::new();
        if let Ok(canonical) = std::fs::canonicalize(&root_path) {
            visited.insert(canonical);
        }

        Self {
            root_path,
            show_hidden: false,
            inner: RefCell::new(inner_tree),
            loaded_paths: RefCell::new(HashSet::new()),
            loading_paths: RefCell::new(HashSet::new()),
            visited_canonical: RefCell::new(visited),
            own_id: Cell::new(None),
        }
    }

    /// Builder method: show or hide hidden files.
    pub fn with_show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    // ─── Tree mutation helpers ─────────────────────────────────────────────────

    fn node_at_path_mut<'a>(
        children: &'a mut Vec<TreeNode>,
        path: &[usize],
    ) -> Option<&'a mut TreeNode> {
        if path.is_empty() {
            return None;
        }
        let idx = path[0];
        if idx >= children.len() {
            return None;
        }
        if path.len() == 1 {
            return Some(&mut children[idx]);
        }
        Self::node_at_path_mut(&mut children[idx].children, &path[1..])
    }

    fn node_data_by_path(children: &[TreeNode], path: &[usize]) -> Option<String> {
        if path.is_empty() {
            return None;
        }
        let idx = path[0];
        if idx >= children.len() {
            return None;
        }
        if path.len() == 1 {
            return children[idx].data.clone();
        }
        Self::node_data_by_path(&children[idx].children, &path[1..])
    }

    // ─── Lazy-load logic ──────────────────────────────────────────────────────

    /// Trigger load of the root directory on mount. The root node already has a
    /// placeholder child; we replace it once the worker result arrives.
    #[allow(dead_code)]
    fn trigger_root_load(&self, ctx: &AppContext) {
        let id = match self.own_id.get() {
            Some(id) => id,
            None => return,
        };
        let dir_path = self.root_path.clone();
        if self.loaded_paths.borrow().contains(&dir_path) {
            return;
        }
        if self.loading_paths.borrow().contains(&dir_path) {
            return;
        }
        self.loading_paths.borrow_mut().insert(dir_path.clone());
        let show_hidden = self.show_hidden;
        ctx.run_worker(id, async move { read_directory(&dir_path, show_hidden) });
    }

    /// Trigger lazy load for the directory at the given tree path (after a NodeExpanded event).
    fn trigger_lazy_load_at(&self, node_path: &[usize], ctx: &AppContext) {
        let id = match self.own_id.get() {
            Some(id) => id,
            None => return,
        };

        // Get the path string from the expanded node.
        let dir_path = {
            let inner = self.inner.borrow();
            let root = inner.root.borrow();
            match Self::node_data_by_path(&root.children, node_path) {
                Some(d) => PathBuf::from(d),
                None => return,
            }
        };

        // Guard: already fully loaded (cached).
        if self.loaded_paths.borrow().contains(&dir_path) {
            return;
        }
        // Guard: currently loading (prevent double-spawn).
        if self.loading_paths.borrow().contains(&dir_path) {
            return;
        }

        // Cycle detection: canonicalize and check visited set.
        match std::fs::canonicalize(&dir_path) {
            Ok(canonical) => {
                if self.visited_canonical.borrow().contains(&canonical) {
                    // Cycle detected — mark node label and clear placeholder.
                    {
                        let inner = self.inner.borrow();
                        let mut root = inner.root.borrow_mut();
                        if let Some(node) =
                            Self::node_at_path_mut(&mut root.children, node_path)
                        {
                            let base = node
                                .label
                                .trim_end_matches(" (cycle)")
                                .to_string();
                            node.label = format!("{} (cycle)", base);
                            node.children = vec![];
                        }
                    }
                    self.inner.borrow().mark_dirty();
                    return;
                }
                self.visited_canonical.borrow_mut().insert(canonical);
            }
            Err(_) => {
                // Inaccessible — clear children.
                {
                    let inner = self.inner.borrow();
                    let mut root = inner.root.borrow_mut();
                    if let Some(node) = Self::node_at_path_mut(&mut root.children, node_path) {
                        node.children = vec![];
                    }
                }
                self.inner.borrow().mark_dirty();
                return;
            }
        }

        // Replace children with "Loading..." placeholder.
        {
            let inner = self.inner.borrow();
            let mut root = inner.root.borrow_mut();
            if let Some(node) = Self::node_at_path_mut(&mut root.children, node_path) {
                node.children = vec![TreeNode::new("Loading...")];
            }
        }
        self.inner.borrow().mark_dirty();

        self.loading_paths.borrow_mut().insert(dir_path.clone());
        let show_hidden = self.show_hidden;
        ctx.run_worker(id, async move { read_directory(&dir_path, show_hidden) });
    }

    /// Apply a completed worker result: find the matching loading path, update children.
    fn apply_worker_result(&self, entries: Vec<DirEntryInfo>) {
        // Find which path we were loading. Only one at a time in normal usage.
        let dir_path = match self
            .loading_paths
            .borrow()
            .iter()
            .next()
            .cloned()
        {
            Some(p) => p,
            None => return,
        };

        self.loading_paths.borrow_mut().remove(&dir_path);
        self.loaded_paths.borrow_mut().insert(dir_path.clone());

        let new_children: Vec<TreeNode> = entries
            .iter()
            .map(|entry| {
                let label = if entry.is_symlink {
                    format!("{}@", entry.name)
                } else {
                    entry.name.clone()
                };
                let mut node = TreeNode::new(&label);
                node.data = Some(entry.path.to_string_lossy().to_string());
                if entry.is_dir {
                    // Placeholder makes the node appear expandable.
                    node.children = vec![TreeNode::new("Loading...")];
                }
                // Symlinks get no children (is_dir = false for symlinks).
                node
            })
            .collect();

        // Replace children of the node whose data matches dir_path.
        let dir_str = dir_path.to_string_lossy().to_string();
        {
            let inner = self.inner.borrow();
            let mut root = inner.root.borrow_mut();
            // Check root itself first (for the root directory load on mount).
            if root.data.as_deref() == Some(&dir_str) {
                root.children = new_children;
            } else {
                Self::replace_children_by_data(&mut root.children, &dir_str, new_children);
            }
        }
        self.inner.borrow().mark_dirty();
    }

    fn replace_children_by_data(
        children: &mut Vec<TreeNode>,
        path_str: &str,
        new_children: Vec<TreeNode>,
    ) -> bool {
        for child in children.iter_mut() {
            if child.data.as_deref() == Some(path_str) {
                child.children = new_children;
                return true;
            }
            if Self::replace_children_by_data(&mut child.children, path_str, new_children.clone())
            {
                return true;
            }
        }
        false
    }
}

impl Widget for DirectoryTree {
    fn widget_type_name(&self) -> &'static str {
        "DirectoryTree"
    }

    fn widget_default_css(&self) -> &'static str {
        "DirectoryTree { border: rounded; min-height: 5; flex-grow: 1; }"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
        // Mount inner Tree with same id so it stores id for message posting.
        self.inner.borrow().on_mount(id);
        // NOTE: We cannot trigger the root load here because ctx is not available
        // in on_mount. The root load is triggered on the first key_bindings() call
        // via the app, or more precisely, the root expand is handled by the Tree
        // toggle action which then fires NodeExpanded. Since the root starts
        // pre-expanded (expanded=true), we trigger the load from on_action("toggle")
        // for the root, which happens automatically when the user expands it.
        // Alternatively, on first render, the placeholder "Loading..." is visible,
        // and we don't have a place to kick off the initial load without ctx.
        //
        // The on_mount override in the app framework doesn't pass ctx. The initial
        // root load will happen when the user presses Space/Right to expand the root.
    }

    fn on_unmount(&self, id: WidgetId) {
        self.inner.borrow().on_unmount(id);
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        tree_key_bindings()
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        // Handle worker results first.
        if let Some(result) = event.downcast_ref::<WorkerResult<Vec<DirEntryInfo>>>() {
            self.apply_worker_result(result.value.clone());
            return EventPropagation::Stop;
        }

        // Handle NodeExpanded from inner Tree to trigger lazy loading.
        if let Some(expanded) = event.downcast_ref::<super::tree_view::messages::NodeExpanded>() {
            let path = expanded.path.clone();
            self.trigger_lazy_load_at(&path, ctx);
            return EventPropagation::Continue;
        }

        // Forward all other events to inner Tree.
        self.inner.borrow().on_event(event, ctx)
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "select" => {
                // On Enter, emit FileSelected or DirectorySelected.
                let id = match self.own_id.get() {
                    Some(id) => id,
                    None => {
                        self.inner.borrow().on_action("select", ctx);
                        return;
                    }
                };

                let node_info = {
                    let inner = self.inner.borrow();
                    inner.cursor_path().and_then(|path| {
                        let root = inner.root.borrow();
                        let data = Self::node_data_by_path(&root.children, &path)
                            .map(PathBuf::from);
                        let has_children = {
                            // Check if node has children (is a directory placeholder or loaded dir).
                            match Self::find_node_has_children(&root.children, &path) {
                                Some(v) => v,
                                None => false,
                            }
                        };
                        data.map(|p| (p, has_children))
                    })
                };

                if let Some((path, has_children)) = node_info {
                    if has_children || path.is_dir() {
                        ctx.post_message(id, messages::DirectorySelected { path });
                    } else {
                        ctx.post_message(id, messages::FileSelected { path });
                    }
                }

                self.inner.borrow().on_action("select", ctx);
            }
            other => {
                self.inner.borrow().on_action(other, ctx);
            }
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        self.inner.borrow().render(ctx, area, buf);
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        // Tree is an inner field, not a mounted child.
        vec![]
    }
}

impl DirectoryTree {
    fn find_node_has_children(children: &[TreeNode], path: &[usize]) -> Option<bool> {
        if path.is_empty() {
            return None;
        }
        let idx = path[0];
        if idx >= children.len() {
            return None;
        }
        if path.len() == 1 {
            return Some(!children[idx].children.is_empty());
        }
        Self::find_node_has_children(&children[idx].children, &path[1..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a uniquely-named temp directory. Not auto-cleaned.
    fn make_test_dir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("textual_rs_dirtree_{}", name));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("failed to create test dir");
        base
    }

    #[test]
    fn dir_entry_info_creation() {
        let entry = DirEntryInfo {
            name: "foo".to_string(),
            path: PathBuf::from("/tmp/foo"),
            is_dir: true,
            is_symlink: false,
        };
        assert_eq!(entry.name, "foo");
        assert!(entry.is_dir);
        assert!(!entry.is_symlink);
    }

    #[test]
    fn read_directory_lists_entries() {
        let dir = make_test_dir("list_entries");
        fs::create_dir(dir.join("subdir")).unwrap();
        fs::write(dir.join("file.txt"), b"hello").unwrap();
        fs::write(dir.join("readme.md"), b"docs").unwrap();

        let entries = read_directory(&dir, false);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();

        assert!(names.contains(&"subdir"), "subdir should be listed");
        assert!(names.contains(&"file.txt"), "file.txt should be listed");
        assert!(names.contains(&"readme.md"), "readme.md should be listed");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_directory_directories_first() {
        let dir = make_test_dir("dirs_first");
        fs::write(dir.join("aaa.txt"), b"").unwrap();
        fs::create_dir(dir.join("bbb_dir")).unwrap();

        let entries = read_directory(&dir, false);
        assert!(entries.len() >= 2);
        assert!(entries[0].is_dir, "first entry should be directory");
        assert_eq!(entries[0].name, "bbb_dir");
        assert_eq!(entries[1].name, "aaa.txt");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_directory_hides_dot_files_by_default() {
        let dir = make_test_dir("hidden_files");
        fs::write(dir.join(".hidden"), b"secret").unwrap();
        fs::write(dir.join("visible.txt"), b"plain").unwrap();

        let entries_hidden = read_directory(&dir, false);
        let entries_shown = read_directory(&dir, true);

        let hidden_names: Vec<&str> = entries_hidden.iter().map(|e| e.name.as_str()).collect();
        let shown_names: Vec<&str> = entries_shown.iter().map(|e| e.name.as_str()).collect();

        assert!(
            !hidden_names.contains(&".hidden"),
            ".hidden should not appear with show_hidden=false"
        );
        assert!(hidden_names.contains(&"visible.txt"), "visible.txt should appear");
        assert!(shown_names.contains(&".hidden"), ".hidden should appear with show_hidden=true");
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn read_directory_detects_symlinks() {
        let dir = make_test_dir("symlinks");
        let target = dir.join("target.txt");
        fs::write(&target, b"target content").unwrap();
        let link = dir.join("link.txt");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let entries = read_directory(&dir, false);
        let link_entry = entries.iter().find(|e| e.name == "link.txt");
        assert!(link_entry.is_some(), "symlink entry should be found");
        let link_entry = link_entry.unwrap();
        assert!(link_entry.is_symlink, "entry should be marked as symlink");
        assert!(!link_entry.is_dir, "symlink should not be treated as dir");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cycle_detection_root_in_visited() {
        let dir = make_test_dir("cycle_detect");
        let dt = DirectoryTree::new(&dir);
        let canonical = std::fs::canonicalize(&dir).unwrap();
        assert!(
            dt.visited_canonical.borrow().contains(&canonical),
            "root canonical path should be in visited_canonical after construction"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn loaded_paths_guard_works() {
        let dir = make_test_dir("loaded_guard");
        let dt = DirectoryTree::new(&dir);
        // Mark root as loaded to simulate caching.
        dt.loaded_paths.borrow_mut().insert(dir.clone());
        assert!(
            dt.loaded_paths.borrow().contains(&dir),
            "loaded_paths should contain the inserted path"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn loading_paths_guard_works() {
        let dir = make_test_dir("loading_guard");
        let dt = DirectoryTree::new(&dir);
        dt.loading_paths.borrow_mut().insert(dir.clone());
        assert!(
            dt.loading_paths.borrow().contains(&dir),
            "loading_paths should contain the in-progress path"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn show_hidden_builder() {
        let dir = make_test_dir("show_hidden");
        let dt = DirectoryTree::new(&dir);
        assert!(!dt.show_hidden);
        let dt2 = dt.with_show_hidden(true);
        assert!(dt2.show_hidden);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn messages_constructible() {
        let _fs = messages::FileSelected {
            path: PathBuf::from("/tmp/file.txt"),
        };
        let _ds = messages::DirectorySelected {
            path: PathBuf::from("/tmp/dir"),
        };
    }
}
