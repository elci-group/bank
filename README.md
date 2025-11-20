# bank 🏦

**Bank** is a command-line utility that combines the functionality of `mkdir` and `touch`. It allows you to create directories or empty files with automatic parent creation, permission management, and interactive prompts for ambiguous paths.
![Bank Demo](assets/bank.gif)
## ✨ Features

- **🗼️ Smart Detection**: Automatically determines whether to create files or directories based on context
- **📁 Parent Creation**: Creates parent directories as needed with `-p` flag
- **🔒 Permission Control**: Set file/directory permissions with `-m` flag
- **💬 Interactive Mode**: Prompts for clarification on ambiguous paths with `-i` flag
- **📝 Touch Functionality**: Updates timestamps for existing files like `touch`
- **🖥️ Verbose Output**: Detailed feedback with `-v` flag
- **⚡ Multi-Path Support**: Create multiple files and directories in a single command
- **⏰ Custom Timestamps**: Set specific dates/times with `--date`, `-t`, or `-r` flags
- **🚫 No-Create Mode**: Update timestamps without creating files using `-c`
- **🎯 Fine-grained Control**: Separate access/modification time control with `-a`/`--mtime`
- **🔗 Symlink Support**: Handle symbolic links with `--no-dereference`

## 🚀 Installation

From the npxr workspace root:

```bash
cargo build --release -p bank
# Binary will be available at ./target/release/bank
```

## 📖 Usage

```bash
bank [OPTIONS] <PATH>...
```

**Note**: Bank now supports multiple paths! You can create multiple files and/or directories in a single command.

### Options

**Creation Control:**
- `-d, --directory`: Force creation as directory (mkdir mode)
- `-f, --file`: Force creation as file (touch mode)
- `-p, --parents`: Create parent directories as needed
- `-m, --mode <MODE>`: Set file/directory permissions (octal format, e.g., 755)
- `-i, --interactive`: Interactive mode for ambiguous paths

**Timestamp Control:**
- `-c, --no-create`: Do not create files, only update timestamps if they exist
- `--date <STRING>`: Parse date string and use it instead of current time
- `-t, --timestamp <STAMP>`: Use timestamp format [[CC]YY]MMDDhhmm[.ss]
- `-r, --reference <FILE>`: Use this file's times instead of current time
- `-a, --atime`: Change only the access time
- `--mtime`: Change only the modification time
- `--no-dereference`: Affect symbolic links instead of referenced files

**General:**
- `-v, --verbose`: Verbose output
- `-h, --help`: Print help
- `-V, --version`: Print version

## 💡 Examples

### Create a file
```bash
# Creates a file (detected by extension)
bank myfile.txt

# Force file creation
bank -f myfile

# Create file with specific permissions
bank -m 755 executable_script.sh
```

### Create a directory
```bash
# Creates a directory (detected by trailing slash)
bank mydir/

# Force directory creation
bank -d mydir

# Create directory with parent directories
bank -p deep/nested/directory/
```

### Advanced usage
```bash
# Interactive mode for ambiguous paths
bank -i ambiguous_name

# Verbose output with parent creation
bank -v -p deep/path/to/file.txt

# Create directory with specific permissions
bank -d -m 755 -v my_executable_dir

# Create multiple files at once
bank file1.txt file2.txt file3.txt

# Create multiple directories
bank -d dir1 dir2 dir3

# Mixed file and directory creation
bank config.json scripts/ data.txt logs/

# Bulk creation with parent directories
bank -p src/components/Button.tsx src/utils/helpers.js tests/unit/button.test.js
```

### Advanced timestamp control
```bash
# Create file with custom date
bank --date "2023-12-25 15:30:00" holiday_log.txt

# Create file with timestamp format
bank -t 202312251530 timestamp_file.txt

# Copy timestamps from another file
bank -r template.txt new_file.txt

# Update timestamps without creating (touch mode)
bank -c existing_file.txt

# Update only access time
bank -a --date "2024-01-01 10:00:00" access_test.txt

# Update only modification time
bank --mtime --date "2024-01-01 10:00:00" mod_test.txt

# Handle symbolic links
bank --no-dereference symlink_target

# Combine features: create with custom time and permissions
bank --date "2024-06-15 14:30:00" -m 755 script.sh
```

## 🔍 Smart Detection Rules

When neither `-f` nor `-d` is specified, Bank uses these heuristics:

1. **Explicit flags**: `-f` or `-d` take precedence
2. **Existing paths**: Maintains the type of existing files/directories
3. **File extensions**: Paths with extensions (e.g., `.txt`) become files
4. **Trailing separators**: Paths ending with `/` become directories
5. **Interactive mode**: Prompts user for ambiguous cases when `-i` is used
6. **Default**: Ambiguous paths default to files

## 🧪 Testing

Run the test suite:

```bash
cargo test -p bank
```

The utility includes comprehensive unit tests covering:
- File and directory creation
- Smart type detection
- Permission setting
- Error handling
- Multi-argument functionality
- Mixed file and directory creation
- Custom timestamp parsing and setting
- No-create mode functionality
- Access/modification time control
- Reference file timestamp copying
- Argument validation and conflict detection

## 🏗️ Architecture

The bank utility is structured around:

- **CLI Parsing**: Uses `clap` for argument parsing with multi-path support
- **Type Detection**: Smart heuristics for file vs directory creation
- **Batch Processing**: Efficient handling of multiple paths in a single invocation
- **File Operations**: Cross-platform file/directory creation
- **Permission Management**: Unix permission handling
- **Timestamp Control**: Advanced timestamp parsing and setting using `chrono`
- **Reference File Support**: Copy timestamps from existing files
- **Time Granularity**: Separate access and modification time control
- **Interactive UI**: Uses `dialoguer` for user prompts
- **Error Handling**: Robust error handling with context using `anyhow`
- **Symlink Awareness**: Proper handling of symbolic links

## 🤝 Contributing

Contributions are welcome! The utility is part of the npxr workspace and follows the same development practices.

## 📄 License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.
