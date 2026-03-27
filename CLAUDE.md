## Shared context
Before starting any task, run `collab list` to check for notes from other sessions.
After completing a task, run `collab add "<one-line summary of what you did and where you left off>"`.

## Git discipline
- `git pull` before starting any work each session.
- Commit changes to master after every passing `cargo test` — small, frequent commits.
- Never commit if `cargo test` fails.
- This keeps Windows (MBPC) and macOS (yubitui) in sync automatically.
