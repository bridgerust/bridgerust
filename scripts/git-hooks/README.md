# Git Hooks

This directory contains git hooks that automatically run checks before commits.

## Available Hooks

### `pre-commit`

Automatically runs before each commit:

1. **rustfmt**: Formats all staged Rust files
2. **clippy**: Runs clippy linting on the workspace

The hook will:
- ✅ Format staged `.rs` files automatically
- ✅ Re-stage formatted files
- ✅ Run `cargo clippy` to check for linting issues
- ❌ Block the commit if clippy finds issues

## Setup

Run the setup script to install the hooks:

```bash
./scripts/setup-git-hooks.sh
```

This creates symlinks from `.git/hooks/` to `scripts/git-hooks/`.

## Manual Installation

If you prefer to install manually:

```bash
ln -s ../../scripts/git-hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## Bypassing Hooks

If you need to bypass the hook (not recommended):

```bash
git commit --no-verify -m "your message"
```

## Troubleshooting

### Hook not running

1. Check if the hook is executable:
   ```bash
   ls -la .git/hooks/pre-commit
   ```

2. Re-run setup:
   ```bash
   ./scripts/setup-git-hooks.sh
   ```

### rustfmt/clippy not found

The hook will automatically install missing components:
- `rustup component add rustfmt`
- `rustup component add clippy`

### Clippy errors

Fix the issues shown by clippy, or run:
```bash
cargo clippy --fix
```

## Customization

To modify the hook behavior, edit `scripts/git-hooks/pre-commit` and re-run the setup script.

