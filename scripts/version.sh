#!/bin/bash

# If no arguments are passed, print the version
if [ $# -eq 0 ]; then
    # Exit with git's exit code
    git describe --tags --abbrev=0
    exit $?
fi

# If more than one argument is passed, print an error message
if [ $# -gt 1 ]; then
    echo "Error: Too many arguments"
    exit 1
fi

# Otherwise, set the version
version=$1
# Check if the version is valid
if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Invalid version format. Use X.Y.Z"
    exit 1
fi
# Check if the version already exists
if git tag | grep -q "v$version"; then
    echo "Error: Version $version already exists"
    exit 1
fi

# Update the version in Cargo.toml and demo.cwrt
sed -i "s/^version = \".*\"/version = \"$version\"/" Cargo.toml
sed -i "s/title: CastWright Demo (v.*)/title: CastWright Demo (v$version)/" tests/demo.cwrt
cargo update

# Generate demo asciicast
cargo build --release --bin castwright --features="cli"
cp ./target/release/castwright ~/.cargo/bin/castwright
castwright -i tests/demo.cwrt -o /tmp/demo.cast -t -x
# Upload demo asciicast
ASCIICAST_LINK=$(asciinema upload /tmp/demo.cast | rg --no-filename 'https://asciinema.org/a/\w+')
# Escape the slashes
ASCIICAST_LINK_ESCAPE=$(echo $ASCIICAST_LINK | sed 's/\//\\\//g')
# Update the version and link in README.md
sed -i "s/\[!\[CastWright Demo (v.*)\](.*)\](.*)/\[!\[CastWright Demo (v$VERSION)\]($ASCIICAST_LINK_ESCAPE.svg)\]($ASCIICAST_LINK_ESCAPE)/" README.md
echo "CastWright Demo (v$VERSION): $ASCIICAST_LINK"

# Commit the changes
git add Cargo.toml Cargo.lock tests/demo.cwrt README.md
git commit -S -m "Bump version to v$VERSION"
# Create a new tag
git tag -s v$version -m "Version $version"

# Display
# echo "Bump version to v$VERSION"
