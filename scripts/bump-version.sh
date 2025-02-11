# Get the current version
VERSION=$(rg --no-filename '^version = "(.*)"' Cargo.toml --replace '$1')

# Increment the patch version
MAJOR=$(echo $VERSION | cut -d. -f1)
MINOR=$(echo $VERSION | cut -d. -f2)
PATCH=$(echo $VERSION | cut -d. -f3)
PATCH=$((PATCH + 1))
VERSION="$MAJOR.$MINOR.$PATCH"

# Update the version in Cargo.toml and demo.cwrt
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
sed -i "s/title: CastWright Demo (v.*)/title: CastWright Demo (v$VERSION)/" tests/demo.cwrt
cargo generate-lockfile

# Generate demo asciicast
cargo build --release
cp ./target/release/castwright ~/.cargo/bin/castwright
castwright -i tests/demo.cwrt -o /tmp/demo.cast -t -x

# Upload demo asciicast
ASCIICAST_LINK=$(asciinema upload /tmp/demo.cast | rg --no-filename 'https://asciinema.org/a/\w+')
# Escape the slashes
ASCIICAST_LINK_ESCAPE=$(echo $ASCIICAST_LINK | sed 's/\//\\\//g')

# Update the version and link in README.md
sed -i "s/\[!\[CastWright Demo (v.*)\](.*)\](.*)/\[!\[CastWright Demo (v$VERSION)\]($ASCIICAST_LINK_ESCAPE.svg)\]($ASCIICAST_LINK_ESCAPE)/" README.md
echo "CastWright Demo (v$VERSION): $ASCIICAST_LINK"

# Commit the change
git add Cargo.toml
git add Cargo.lock
git add tests/demo.cwrt
git add README.md
git commit -S -m "Bump version to v$VERSION"

# Tag
git tag -s "v$VERSION" -m "Version v$VERSION"

# Display
# echo "Bump version to v$VERSION"
