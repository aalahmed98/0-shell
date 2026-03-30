# Testing 0-Shell

## Option 1: Test in WSL (Recommended for Unix features)

1. **Open WSL terminal** (Ubuntu/Debian/etc.)

2. **Navigate to your project** (Windows files are accessible via `/mnt/c/`)
   ```bash
   cd /mnt/c/Users/nutsu/Desktop/Coding/0-shell
   ```

3. **Install Rust in WSL** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

4. **Build the project**:
   ```bash
   cargo build --release
   ```

5. **Run the shell**:
   ```bash
   ./target/release/zero-shell
   ```

6. **Test commands**:
   ```
   $ pwd
   $ ls -la
   $ cd /tmp
   $ mkdir test_dir
   $ cd test_dir
   $ echo "Hello World" > test.txt
   $ cat test.txt
   $ ls -l
   $ cd ..
   $ rm -r test_dir
   $ exit
   ```

## Option 2: Test on Windows (Limited functionality)

You can test basic functionality on Windows, but Unix-specific features won't work properly:

1. **Build** (already done):
   ```powershell
   cargo build --release
   ```

2. **Run**:
   ```powershell
   .\target\release\zero-shell.exe
   ```

Note: File permissions, UID/GID, and some path handling will be different on Windows.

## What to Test

- ✅ Basic commands: `pwd`, `ls`, `echo`, `cd`
- ✅ `ls` flags: `ls -l`, `ls -a`, `ls -F`
- ✅ File operations: `cat`, `cp`, `mv`, `rm`
- ✅ Directory operations: `mkdir`, `cd`, `rm -r`
- ✅ Error handling: `cd nonexistent`, `cat missing_file`
- ✅ Unrecognized commands: `invalid_command`
- ✅ Ctrl+D (EOF) to exit

