#!/usr/bin/env bats

load 'bats-support/load.bash'
load 'bats-assert/load.bash'
load 'bats-file/load.bash'

# BATS test for Trimorph core functionality

setup() {
  # Enable test mode to disable sudo commands in scripts
  export TRIMORPH_TEST_MODE=1

  # Make scripts executable for the test run
  chmod +x ./usr/local/sbin/*

  # Create a temporary directory for our test environment
  export BATS_TMPDIR="$(mktemp -d -t trimorph-test.XXXXXX)"
  export TRIMORPH_ETC_DIR="$BATS_TMPDIR/etc/trimorph"
  export TRIMORPH_BASE_DIR="$BATS_TMPDIR/usr/local/trimorph/base"
  export TRIMORPH_LOG_DIR="$BATS_TMPDIR/var/log/trimorph"
  export TRIMORPH_CACHE_DIR="$BATS_TMPDIR/var/cache/trimorph/packages"
  export TRIMORPH_TMP_DIR="$BATS_TMPDIR/var/tmp"

  mkdir -p "$TRIMORPH_ETC_DIR/jails.d"
  mkdir -p "$TRIMORPH_BASE_DIR"
  mkdir -p "$TRIMORPH_LOG_DIR"
  mkdir -p "$TRIMORPH_CACHE_DIR"
  mkdir -p "$TRIMORPH_TMP_DIR"

  # Create mocks for system commands, but use the REAL parser
  export PATH="$BATS_TMPDIR/bin:$PATH"
  mkdir -p "$BATS_TMPDIR/bin"

  # Mock systemd-nspawn
  cat > "$BATS_TMPDIR/bin/systemd-nspawn" <<'EOF'
#!/bin/bash
echo "systemd-nspawn called with: $@" >&2
echo "Mock nspawn output"
EOF
  chmod +x "$BATS_TMPDIR/bin/systemd-nspawn"

  # Mock systemd-run
  cat > "$BATS_TMPDIR/bin/systemd-run" <<'EOF'
#!/bin/bash
# Pass through to the underlying command (our mock nspawn)
shift
while [[ "$1" != "systemd-nspawn" ]]; do
  shift
done
"$@"
EOF
  chmod +x "$BATS_TMPDIR/bin/systemd-run"

  # Create a COMPLETE fake jail config for the REAL parser to read
  cat > "$TRIMORPH_ETC_DIR/jails.d/test-jail.conf" <<EOF
root=$BATS_TMPDIR/fakeroot
pkgmgr=/bin/echo
pkgmgr_args=hello
bootstrap=mkdir -p {root} && touch {root}/bootstrapped
EOF
}

teardown() {
  # No sudo needed here anymore, as all files are owned by the user
  rm -rf "$BATS_TMPDIR"
}

@test "bootstrap creates base image and initialization marker" {
  run ./usr/local/sbin/trimorph-bootstrap test-jail
  assert_success
  [ -d "$TRIMORPH_BASE_DIR/test-jail" ]
  [ -f "$TRIMORPH_BASE_DIR/test-jail/bootstrapped" ]
  [ -f "$TRIMORPH_ETC_DIR/runtime/test-jail.initialized" ]
}

@test "solo --check passes with a bootstrapped jail" {
  ./usr/local/sbin/trimorph-bootstrap test-jail
  run ./usr/local/sbin/trimorph-solo --check test-jail
  assert_success
  assert_output --partial "Checks Passed"
}

@test "solo --check fails without a bootstrapped jail" {
  run ./usr/local/sbin/trimorph-solo --check test-jail
  assert_failure
  assert_output --partial "Base image is missing"
}

@test "solo --dry-run prints the command" {
  ./usr/local/sbin/trimorph-bootstrap test-jail
  run ./usr/local/sbin/trimorph-solo --dry-run test-jail my-command
  assert_success
  assert_output --partial "systemd-run"
  assert_output --partial "systemd-nspawn"
  assert_output --partial "--overlay="
  assert_output --partial "my-command"
}

@test "solo executes a command in a jail" {
  ./usr/local/sbin/trimorph-bootstrap test-jail

  # Explicitly redirect stderr to a file to bypass bats capture issues
  local stderr_file="$BATS_TMPDIR/stderr.log"
  run sh -c "./usr/local/sbin/trimorph-solo test-jail 2> '$stderr_file'"

  assert_success
  assert_file_contains "$stderr_file" "systemd-nspawn called with"
  assert_output --partial "Mock nspawn output"
}