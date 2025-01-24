# Call rustfmt on `src/*.rs` and `tests/*.rs` files.
# Usage: `./scripts/format.sh`

# Exit on error
set -e

# Check if rustfmt is installed
if ! command -v rustfmt &> /dev/null; then
    echo "rustfmt is not installed."
    exit 1
fi

# Format the code
echo "Formatting the code..."
rustfmt src/*.rs
echo "Done."
