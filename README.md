
## BinRW
### CLI development
* git clone https://github.com/jackalope-code/desktop-apps
* cd binary-view-edit/binrw-cli
* cargo install
* cargo run

### Build CLI
* git clone https://github.com/jackalope-code/desktop-apps
* cd binary-view-edit/binrw-cli
* cargo build
* Use build in target directory

# Run app with GUI in Tauri
* cd into project directory
* cd client
* npm install
* npm run tauri dev

### Test
* cd into project directory
* cd binary-view-edit/binrw-cli
* cargo test

### Bugs from tests


# Active To-Do
* Add file write (with overwrite/~~splice~~ options)
  * Add an exact write mode to write overwrite that writes exactly over a buffer, perhaps optionally padding one side or the other with zeroes.
* Utilities
  * Add text and binary diffing
  * Add a "strings" reading/printing utility
  * Integration with disassembly/assembly utils to modify binaries?
  * DLL edit/replacement?
* CLI args and parsing (split into a module)
* Testing
* ~~Get file size in bytes~~
  * Incorporate into a size command
  * Incorporate into negative offsets for file read/write (not negative byte length, negative offsets)... DO THIS ACROSS THE APP
* How far do I want to go with file type detection?
* Add a GUI by using a Rust API as an interface that can be called from Typescript (Tauri?). Bundle the application GUI in Electron.
* ~~Make a file detection algorithm from magic numbers at the start and (at least for jpgs) end of files~~
* Add metadata parse (read/write) options (not to be confused with metadata in the Rust library, which would be useful here in differentiating symlinks/dirs/files).
  * ID3
    * v1
      * READ LAST 128 bytes into struct somehow
      * After getting the rest working, parse the genre table and get a genre somehow

  * EXIF
    * Add EXIF reading
    * Add EXIF clearing for e.g. time, timezone, place. Allow writing/rewriting if there are comments.
* Review, test, and document
* Read the Rust book
* Read about design patterns that are useful in Rust

# Done
~~* Add quadruple write test case and fix bug (try writing hello and it breaks after two times) FIXED~~
~~* Added a temp file module that wasn't on the to-do~~

# Stuff to work on
* Figure out how to keep targets in different directories out of .gitignore
* Parse just the beginning, just the end, or both to figure out file types
* Compare sequences of byte strings, not concatenated byte strings
* Figure out strings, printing, string formatting in Rust
* Add file and metadata processing

# Planned features
* Read all data
* Read a sequence of data
* Attempt to detect file type from known file types
* Overwrite one or several portions of data (overwrite or splice), given a start position, length, and flag to overwrite or splice.
* Attempt disassembly?
* Interface with Tauri and add a GUI?
* Text editing?
* Edit java class files from a GUI by plugging into javap with Rust
* Strings text utility to read from the executable/data, not headers/relocation by default


    // TODO: Add tests.
    // Tests:
    // Check correctness and edge cases.
    // ****Read****
    // Negative offset to eof (tested manually to work.. add a case for an easy pass)
    // Large positive offset (larger than i64) to eof
    // Two negative offsets going the correct direction (currently failing)
    // Two negative offsets going in the incorrect direction should fail
    // Two large positive offsets (each larger than i64)
    // Negative and large positive offsets
    // Large positive and negative offsets
    // ****Write****
    // Test same offsets as read (negative to eof, large positive to eof,two negative negative cases, negative large positive, large positive negative, large positive large positive)
    // Test each write operation with each offset (overwrite, splice)
    // Data write verify mode
    // ****Header****
    // Test on range of files
    // ****Type****
    // Test on range of files
    // ****Size****
    // Test on range of files
    // ****Metadata****
    // Add file-specific and metadata-specific metadata checks as metadata functionality is added. Test read, write, write-verify.
    // Metadata write verify mode
    // TODO: Read breaks with negative offsets when not hitting the "eof" true case on the if statement.
