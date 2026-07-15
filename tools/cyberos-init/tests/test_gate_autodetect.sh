#!/usr/bin/env bash
# test_gate_autodetect.sh - TASK-CUO-207 §5 suite (t01-t08 -> AC 1-8).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$repo/tools/cyberos-init/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }

initrepo() { # initrepo <dir> ; markers already placed by caller
  ( cd "$1" && git init -q . 2>/dev/null; bash "$TMP/payload/install.sh" "$1" >/dev/null 2>&1 )
}
genv() { grep "^$2=" "$1/.cyberos/gates.env" | head -1 | cut -d= -f2- | tr -d '"'; }

t01_stack_matrix() {                                                   # AC 1
  local all=1
  mkdir -p "$TMP/go" && echo "module x" > "$TMP/go/go.mod" && initrepo "$TMP/go"
  [ "$(genv "$TMP/go" BUILD_CMD)" = "go build ./..." ] && [ "$(genv "$TMP/go" COVERAGE_CMD)" = "go test -coverprofile=coverage.out ./..." ] || { fail t01-go "$(genv "$TMP/go" BUILD_CMD)"; all=0; }
  mkdir -p "$TMP/mvn" && echo "<project/>" > "$TMP/mvn/pom.xml" && initrepo "$TMP/mvn"
  [ "$(genv "$TMP/mvn" BUILD_CMD)" = "mvn -q -DskipTests package" ] && [ "$(genv "$TMP/mvn" TEST_CMD)" = "mvn -q verify" ] && [ -z "$(genv "$TMP/mvn" COVERAGE_CMD)" ] || { fail t01-mvn "x"; all=0; }
  mkdir -p "$TMP/gr" && touch "$TMP/gr/build.gradle" && initrepo "$TMP/gr"
  [ "$(genv "$TMP/gr" BUILD_CMD)" = "gradle build" ] || { fail t01-gradle "$(genv "$TMP/gr" BUILD_CMD)"; all=0; }
  mkdir -p "$TMP/grw" && touch "$TMP/grw/build.gradle.kts" && touch "$TMP/grw/gradlew" && chmod +x "$TMP/grw/gradlew" && initrepo "$TMP/grw"
  [ "$(genv "$TMP/grw" BUILD_CMD)" = "./gradlew build" ] || { fail t01-gradlew "$(genv "$TMP/grw" BUILD_CMD)"; all=0; }
  mkdir -p "$TMP/net" && touch "$TMP/net/app.csproj" && initrepo "$TMP/net"
  [ "$(genv "$TMP/net" BUILD_CMD)" = "dotnet build" ] && [ "$(genv "$TMP/net" TEST_CMD)" = "dotnet test" ] || { fail t01-dotnet "x"; all=0; }
  mkdir -p "$TMP/rb/spec" && touch "$TMP/rb/Gemfile" && initrepo "$TMP/rb"
  [ "$(genv "$TMP/rb" TEST_CMD)" = "bundle exec rspec" ] || { fail t01-rspec "$(genv "$TMP/rb" TEST_CMD)"; all=0; }
  mkdir -p "$TMP/rk" && touch "$TMP/rk/Gemfile" "$TMP/rk/Rakefile" && initrepo "$TMP/rk"
  [ "$(genv "$TMP/rk" TEST_CMD)" = "bundle exec rake test" ] || { fail t01-rake "$(genv "$TMP/rk" TEST_CMD)"; all=0; }
  mkdir -p "$TMP/phpu/vendor/bin" && echo '{}' > "$TMP/phpu/composer.json" && touch "$TMP/phpu/vendor/bin/phpunit" && initrepo "$TMP/phpu"
  [ "$(genv "$TMP/phpu" TEST_CMD)" = "vendor/bin/phpunit" ] || { fail t01-phpunit "$(genv "$TMP/phpu" TEST_CMD)"; all=0; }
  [ "$all" -eq 1 ] && ok t01
}
t02_multistack_union() {                                               # AC 2
  mkdir -p "$TMP/multi" && echo "module m" > "$TMP/multi/go.mod" && echo '{"scripts":{"test":"jest"}}' > "$TMP/multi/package.json"
  initrepo "$TMP/multi"
  [ "$(genv "$TMP/multi" SRC_TEST)" = "node" ] && [ "$(genv "$TMP/multi" SRC_BUILD)" = "go" ] \
    && grep -q "node" "$TMP/multi/.cyberos/gates.env" && grep -q "go" "$TMP/multi/.cyberos/gates.env" \
    && ok t02 || fail t02 "test=$(genv "$TMP/multi" SRC_TEST) build=$(genv "$TMP/multi" SRC_BUILD)"
}
t03_marker_gating() {                                                  # AC 3
  mkdir -p "$TMP/php" && echo '{}' > "$TMP/php/composer.json" && initrepo "$TMP/php"
  [ "$(genv "$TMP/php" LINT_CMD)" = "composer validate --strict" ] && [ -z "$(genv "$TMP/php" TEST_CMD)" ] \
    && ok t03 || fail t03 "lint=$(genv "$TMP/php" LINT_CMD) test=$(genv "$TMP/php" TEST_CMD)"
}
rungates() { ( cd "$1" && bash "$1/.cyberos/cuo/gates/run-gates.sh" 2>&1 ); }
t04_config_per_key_override() {                                        # AC 4
  d="$TMP/go"  # reuse go fixture (real gates.env with SRC_*)
  printf 'gates:\n  lint: "echo lint-from-config"\n' > "$d/.cyberos/config.yaml"
  sed -i.bak 's/^BUILD_CMD=.*/BUILD_CMD="echo build-ok"/;s/^TEST_CMD=.*/TEST_CMD="echo test-ok"/;s/^COVERAGE_CMD=.*/COVERAGE_CMD="echo cov-ok"/' "$d/.cyberos/gates.env" && rm -f "$d/.cyberos/gates.env.bak"
  out="$(rungates "$d")"
  grep -q "gate lint: echo lint-from-config (source: config)" <<<"$out" \
    && grep -q "gate build: echo build-ok (source: autodetect:go)" <<<"$out" \
    && ok t04 || fail t04 "$out"
}
t05_scaffold_once() {                                                  # AC 5
  d="$TMP/go"
  grep -q "autodetected: go" "$d/.cyberos/config.yaml.orig" 2>/dev/null || cp "$d/.cyberos/config.yaml" "$d/.cyberos/config.yaml.orig"
  echo "# operator edit" >> "$d/.cyberos/config.yaml"
  before="$(sha256sum "$d/.cyberos/config.yaml" | cut -d' ' -f1)"
  bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
  after="$(sha256sum "$d/.cyberos/config.yaml" | cut -d' ' -f1)"
  [ "$before" = "$after" ] && ok t05 || fail t05 "config clobbered on re-init"
}
t06_threshold_env() {                                                  # AC 6
  d="$TMP/go"
  printf 'coverage_threshold: 85\ngates:\n  coverage: "echo thr=$CYBEROS_COVERAGE_THRESHOLD"\n' > "$d/.cyberos/config.yaml"
  out="$(rungates "$d")"
  grep -q "thr=85" <<<"$out" || { fail t06 "$out"; return; }
  printf 'gates:\n  coverage: "echo thr=$CYBEROS_COVERAGE_THRESHOLD"\n' > "$d/.cyberos/config.yaml"
  out="$(rungates "$d")"
  grep -q "thr=90" <<<"$out" && ok t06 || fail t06 "default: $out"
}
t07_reduced_floor_message() {                                          # AC 7
  mkdir -p "$TMP/empty" && initrepo "$TMP/empty"
  out="$(rungates "$TMP/empty")"
  grep -q "config.yaml" <<<"$out" && grep -q "floor only" <<<"$out" \
    && ok t07 || fail t07 "$out"
}
t08_malformed_config_loud() {                                          # AC 8
  d="$TMP/go"
  printf 'gates:\n\tlint: "tabbed"\n' > "$d/.cyberos/config.yaml"
  sed -i.bak 's|^BUILD_CMD=.*|BUILD_CMD="touch '"$d"'/ran-anyway"|' "$d/.cyberos/gates.env" && rm -f "$d/.cyberos/gates.env.bak"
  out="$(rungates "$d")"; rc=$?
  [ "$rc" -eq 2 ] && grep -q "MALFORMED" <<<"$out" && grep -q "line 2" <<<"$out" && [ ! -f "$d/ran-anyway" ] \
    && ok t08 || fail t08 "rc=$rc $out"
  rm -f "$d/.cyberos/config.yaml"
}

t01_stack_matrix; t02_multistack_union; t03_marker_gating; t04_config_per_key_override
t05_scaffold_once; t06_threshold_env; t07_reduced_floor_message; t08_malformed_config_loud
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
