# Issues & Lessons Learned (as of Feb 26, 2026)

## Migration and Refactoring Challenges
- Migrating from manual CLI parsing to `clap` 4.x required careful translation of argument handling and subcommand logic.
- Initial migration attempts led to build errors (missing imports, global-scope lets, incorrect macro usage) that required iterative fixes.
- Ensuring all business logic (especially for write/overwrite/insert) was preserved during CLI modernization was critical to avoid regressions.

## Test Failures and Debugging
- After migration, several tests in `write_overwrite` failed due to missing or incomplete restoration of canonical logic from `main reference.rs`.
- Edge cases for descending ranges, reverse flag, and zero-padding were not handled identically to the canonical implementation, causing output mismatches (e.g., "FOOfoo" instead of "FOOBAR").
- Overwrite/insert logic must strictly enforce: data length must match range length for range operations, and for descending ranges, data must be reversed and written at the lower index. No file modification should occur for invalid/mismatched ranges.
- Zero-padding and write-past-EOF logic must match test expectations exactly; otherwise, tests will fail.

## Lessons for Future Contributors
- Always compare new logic to the canonical implementation (see `main reference.rs`) for edge cases and test requirements.
- After any refactor or migration, run the full test suite and check for subtle output mismatches, not just build success.
- If any previously passing test fails, revert or fix the change immediately—no code change is allowed to break existing tests.
- Document all non-obvious edge cases and test requirements in this file for future onboarding and maintenance.
# Copilot Instructions for AI Agents

## Project Overview
This workspace contains Rust CLI tools for binary file viewing and editing, primarily in `binary-view-edit/binrw-cli`. **Always use `C://Documents/GitHub/desktop-apps/binary-view-edit/binrw-cli` as the project root for all build, test, and debug commands.** The main focus is on file manipulation (read, write, diff, metadata parsing) with extensible utilities for text/binary operations and future GUI integration.

## Architecture & Key Components
- **binrw-cli**: Main CLI tool. Entry point: `src/main.rs`. Core logic in `src/lib.rs` and utility modules under `src/utils/` (e.g., `argparse.rs`, `id3v1.rs`, `tempfile.rs`).
- **Testing**: All tests are in `tests/` (e.g., `copy.rs`, `diff.rs`). Test data is in `tests/data/`.
- **CLI Args**: Argument parsing is modularized in `src/utils/argparse.rs`.
- **Metadata**: ID3v1 genre data and parsing logic in `src/utils/id3v1.rs` and `idv3v1_genre_data.txt`.


## Developer Workflows
**Build/Test/Debug:**
  - Always run commands from `C://Documents/GitHub/desktop-apps/binary-view-edit/binrw-cli`.
  - Build: `cargo install` (optional, for global install)
  - Run: `cargo run` (to execute CLI)
  - Test: `cargo test`
  - Debug: Use `debug_log.txt` for logging/debug output. Check for EOF write bugs and buffer handling (see README notes).
  - Custom file writing example: `cargo run write insert tests/data/alphabet.txt alphabet_copy.txt --append-zero-past-eof 0 "FOOBAR"`

**Test Passing Requirement:**
  - You may only make or keep code changes if all currently passing tests remain passing. If any previously passing test fails after your change, you must revert or fix the change until all previously passing tests pass again.
  - **No code change is allowed to be merged or kept if it causes any existing test to fail.** Always run the full test suite after any change, and only keep changes if all tests pass. If any test fails, you must fix or revert the change immediately.


## Patterns & Conventions
- **File Operations**: Overwrite/splice logic is handled in write commands. Special care is required for:
  - **Descending Ranges & Reverse**: For overwrite/insert, if a descending range is specified (stop < start), the data must be reversed and written at the lower index. For overwrite, the data length must exactly match the range length; otherwise, the file must not be modified.
  - **Zero-Padding (write-past-eof)**: When writing past EOF with the append-zero flag, the buffer must be extended with zeros up to the specified offset before appending data. The resulting file length must match test expectations exactly.
  - **Invalid/Mismatched Ranges**: If an overwrite/insert operation is invalid (e.g., data length does not match range, or out-of-bounds without the correct flag), the file must not be modified.
- **Testing**: Use dedicated test files and data. Add new tests in `tests/` and update test data as needed. **All test cases must be matched exactly, including edge cases for descending, reverse, and zero-padding.**
- **Utilities**: Utility modules are in `src/utils/`. Add new utilities here and update `mod.rs` for module registration.
- **Metadata Parsing**: Extend ID3/EXIF logic in respective utility files. Use genre tables and byte parsing for ID3v1.

## Integration Points
- **External**: Planned integration with Tauri/Electron for GUI. Current focus is CLI and Rust-based utilities.
- **Data Flow**: CLI args → main.rs → lib.rs → utils → file operations/tests.

## References
- See [README.md](../../README.md) for install, test, and feature roadmap.
- Key files: `src/main.rs`, `src/lib.rs`, `src/utils/`, `tests/`, `debug_log.txt`.

## Example Workflow
1. Add new file operation in `src/lib.rs`.
3. Add/modify tests in `tests/`.
4. Run `cargo test` and check `debug_log.txt` for issues.

---
_Iterate and update this file as project structure or workflows change. Ask for feedback if any section is unclear or incomplete._
