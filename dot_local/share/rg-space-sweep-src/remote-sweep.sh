SCRIPT_VERSION='space-sweep-remote-v1'
CANDIDATES="$(mktemp)"
SIZES="$(mktemp)"
SORTED="$(mktemp)"
trap 'rm -f "$CANDIDATES" "$SIZES" "$SORTED"' EXIT

format_bytes() {
  awk -v bytes="$1" 'BEGIN { printf "%6.1fG", bytes / 1024 / 1024 / 1024 }'
}

json_escape() {
  sed 's/\\/\\\\/g; s/"/\\"/g; s/	/\\t/g' <<<"$1"
}

display_path() {
  local path="$1"
  if [[ "$path" == "$HOME_DIR" ]]; then
    printf '~\n'
  elif [[ "$path" == "$HOME_DIR/"* ]]; then
    printf '~/%s\n' "${path#"$HOME_DIR"/}"
  else
    printf '%s\n' "$path"
  fi
}

path_is_under_roots() {
  local path="$1"
  local root
  for root in "${ROOTS[@]}"; do
    [[ "$path" == "$root" || "$path" == "$root/"* ]] && return 0
  done
  return 1
}

path_is_excluded() {
  local path="$1"
  local excluded
  for excluded in "${EXCLUDE_PATHS[@]}"; do
    [[ "$path" == "$excluded" || "$path" == "$excluded/"* ]] && return 0
  done
  return 1
}

add_candidate() {
  local category="$1"
  local path="$2"
  [[ -d "$path" ]] || return 0
  path_is_under_roots "$path" || return 0
  path_is_excluded "$path" && return 0
  printf '%s\t%s\n' "$category" "$path" >> "$CANDIDATES"
}

find_dirs_in_root() {
  local root="$1"
  shift
  local prune_args=()
  local prune
  for prune in "${PRUNE_PATHS[@]}" "${EXCLUDE_PATHS[@]}"; do
    [[ -n "${prune:-}" ]] || continue
    prune_args+=(-o -path "$prune")
  done
  find "$root" -xdev \
    \( -path "$HOME_DIR/.cache" \
    -o -path "$HOME_DIR/.cargo" \
    -o -path "$HOME_DIR/.local/share/containers" \
    -o -path "$HOME_DIR/.local/share/Trash" \
    -o -name .git \
    -o -name .direnv \
    -o -name .pixi \
    -o -name .nox \
    -o -name __pycache__ \
    "${prune_args[@]}" \) -prune \
    -o -type d "$@" -prune -print 2>/dev/null || true
}

find_dirs() {
  local root
  for root in "${ROOTS[@]}"; do
    [[ -d "$root" ]] || continue
    find_dirs_in_root "$root" "$@"
  done
}

find_named() {
  find_dirs -name "$1"
}

find_path() {
  find_dirs -path "$1"
}

find_python_caches() {
  local root
  for root in "${ROOTS[@]}"; do
    [[ -d "$root" ]] || continue
    find "$root" -xdev \
      \( -path "$HOME_DIR/.cache" \
      -o -path "$HOME_DIR/.cargo" \
      -o -path "$HOME_DIR/.local/share/containers" \
      -o -path "$HOME_DIR/.local/share/Trash" \
      -o -name .git \
      -o -name .direnv \
      -o -name .pixi \
      -o -name .nox \
      -o -name __pycache__ \
      -o -name target \
      -o -name node_modules \
      -o -name .venv \
      -o -name .tox \) -prune \
      -o -type d \( -name .pytest_cache \
      -o -name .mypy_cache \
      -o -name .ruff_cache \
      -o -name .hypothesis \) -prune -print 2>/dev/null || true
  done
}

has_cargo_parent() {
  local path="$1"
  local parent
  parent="$(dirname "$path")"
  [[ -f "$parent/Cargo.toml" || -f "$parent/Cargo.lock" ]]
}

looks_like_rust_target() {
  local path="$1"
  [[ -d "$path/debug" || -d "$path/release" || -d "$path/.fingerprint" || -e "$path/.rustc_info.json" ]]
}

collect_category() {
  local category="$1"
  local path
  case "$category" in
    rust)
      while IFS= read -r path; do
        if has_cargo_parent "$path" && looks_like_rust_target "$path"; then
          add_candidate rust "$path"
        fi
      done < <(find_named target)
      add_candidate rust "$HOME_DIR/.cargo/registry/cache"
      add_candidate rust "$HOME_DIR/.cargo/git/db"
      ;;
    python)
      while IFS= read -r path; do
        add_candidate python "$path"
      done < <(find_python_caches)
      add_candidate python "$HOME_DIR/.cache/pip"
      add_candidate python "$HOME_DIR/.cache/uv"
      add_candidate python "$HOME_DIR/.cache/pre-commit"
      add_candidate python "$HOME_DIR/.local/share/hatch"
      ;;
    pixi)
      add_candidate pixi "$HOME_DIR/.cache/rattler/cache"
      ;;
    tox)
      while IFS= read -r path; do
        add_candidate tox "$path"
      done < <(find_named .tox)
      ;;
    venv)
      while IFS= read -r path; do
        add_candidate venv "$path"
      done < <(find_named .venv)
      ;;
    js)
      while IFS= read -r path; do
        add_candidate js "$path"
      done < <(find_path "*/node_modules/.cache")
      add_candidate js "$HOME_DIR/.npm"
      ;;
    *)
      echo "unknown category in remote script: $category" >&2
      exit 64
      ;;
  esac
}

collect_candidates() {
  local category
  for category in $CATEGORIES; do
    collect_category "$category"
  done
  sort -u "$CANDIDATES" -o "$CANDIDATES"
}

size_candidates() {
  local category path size
  : > "$SIZES"
  while IFS=$'\t' read -r category path; do
    [[ -n "${category:-}" && -n "${path:-}" ]] || continue
    size="$(du -s --block-size=1 "$path" 2>/dev/null | awk '{print $1}')" || continue
    [[ -n "$size" ]] || continue
    printf '%s\t%s\t%s\n' "$size" "$category" "$path" >> "$SIZES"
  done < "$CANDIDATES"
  sort -rn "$SIZES" > "$SORTED"
}

print_report_text() {
  local category total count shown size label path display total_all count_all
  echo "Category totals"
  for category in $CATEGORIES; do
    read -r total count < <(awk -F '\t' -v cat="$category" '$2 == cat { total += $1; count += 1 } END { printf "%s %s\n", total + 0, count + 0 }' "$SORTED")
    if [[ "$count" != "0" ]]; then
      label="$(format_bytes "$total")"
      printf '%s  %-6s (%2d paths)\n' "$label" "$category" "$count"
    fi
  done

  echo
  echo "Top paths"
  shown=0
  while IFS=$'\t' read -r size category path; do
    [[ -n "${size:-}" ]] || continue
    label="$(format_bytes "$size")"
    display="$(display_path "$path")"
    printf '%s  %-6s  %s\n' "$label" "$category" "$display"
    shown=$((shown + 1))
    [[ "$shown" -ge "$LIMIT" ]] && break
  done < "$SORTED"

  read -r total_all count_all < <(awk -F '\t' '{ total += $1; count += 1 } END { printf "%s %s\n", total + 0, count + 0 }' "$SORTED")
  echo
  printf 'Grand total: %s across %d matched paths\n' "$(format_bytes "$total_all")" "$count_all"
}

print_report_json() {
  local size category path count total_all first display total paths
  read -r total_all count < <(awk -F '\t' '{ total += $1; count += 1 } END { printf "%s %s\n", total + 0, count + 0 }' "$SORTED")
  printf '{"mode":"%s","target":"%s","grand_total_bytes":%s,"matched_paths":%s,"totals":[' "$MODE" "$(json_escape "$TARGET_NAME")" "$total_all" "$count"
  first=1
  while IFS=$'\t' read -r category total paths; do
    [[ -n "${category:-}" ]] || continue
    if [[ "$first" == 0 ]]; then printf ','; fi
    first=0
    printf '{"category":"%s","size_bytes":%s,"paths":%s}' "$(json_escape "$category")" "$total" "$paths"
  done < <(awk -F '\t' '{ total[$2] += $1; count[$2] += 1 } END { for (category in total) printf "%s\t%s\t%s\n", category, total[category] + 0, count[category] + 0 }' "$SORTED" | sort)
  printf '],"top_paths":['
  first=1
  while IFS=$'\t' read -r size category path; do
    [[ -n "${size:-}" ]] || continue
    display="$(display_path "$path")"
    if [[ "$first" == 0 ]]; then printf ','; fi
    first=0
    printf '{"size_bytes":%s,"category":"%s","path":"%s"}' "$size" "$(json_escape "$category")" "$(json_escape "$display")"
  done < <(head -n "$LIMIT" "$SORTED")
  printf ']'
  if [[ "$MODE" == "clean" && "$DRY_RUN" == "1" ]]; then
    printf ',"dry_run":true,"would_remove":['
    first=1
    while IFS=$'\t' read -r size category path; do
      [[ -n "${size:-}" ]] || continue
      display="$(display_path "$path")"
      if [[ "$first" == 0 ]]; then printf ','; fi
      first=0
      printf '{"size_bytes":%s,"category":"%s","path":"%s"}' "$size" "$(json_escape "$category")" "$(json_escape "$display")"
    done < "$SORTED"
    printf ']'
  fi
  printf '}\n'
}

safe_to_remove() {
  local category="$1"
  local path="$2"
  local name parent
  path_is_under_roots "$path" || return 1
  [[ "$path" != "$HOME_DIR" ]] || return 1
  path_is_excluded "$path" && return 1
  case "$path" in
    "$HOME_DIR/.local/bin"|"$HOME_DIR/.local/bin/"*|"$HOME_DIR/.cargo/bin"|"$HOME_DIR/.cargo/bin/"*)
      return 1
      ;;
    "$HOME_DIR/.cargo/registry/cache"|"$HOME_DIR/.cargo/git/db"|"$HOME_DIR/.cache/pip"|"$HOME_DIR/.cache/uv"|"$HOME_DIR/.cache/rattler/cache"|"$HOME_DIR/.cache/pre-commit"|"$HOME_DIR/.local/share/hatch"|"$HOME_DIR/.npm")
      return 0
      ;;
  esac

  name="${path##*/}"
  case "$category" in
    rust)
      [[ "$name" == "target" ]] && has_cargo_parent "$path" && looks_like_rust_target "$path"
      ;;
    python)
      [[ "$name" == ".pytest_cache" || "$name" == ".mypy_cache" || "$name" == ".ruff_cache" || "$name" == ".hypothesis" ]]
      ;;
    pixi)
      [[ "$path" == "$HOME_DIR/.cache/rattler/cache" ]]
      ;;
    tox)
      [[ "$name" == ".tox" ]]
      ;;
    venv)
      [[ "$name" == ".venv" ]]
      ;;
    js)
      parent="$(basename "$(dirname "$path")")"
      [[ "$name" == ".cache" && "$parent" == "node_modules" ]]
      ;;
    *)
      return 1
      ;;
  esac
}

clean_paths() {
  local size category path label display
  if [[ "${YES:-0}" != "1" ]]; then
    echo "refusing to clean without --yes; use --dry-run to preview first" >&2
    exit 64
  fi

  while IFS=$'\t' read -r size category path; do
    [[ -n "${path:-}" ]] || continue
    if ! safe_to_remove "$category" "$path"; then
      echo "refusing to remove unexpected path: $path" >&2
      exit 65
    fi
  done < "$SORTED"

  while IFS=$'\t' read -r size category path; do
    [[ -n "${path:-}" ]] || continue
    label="$(format_bytes "$size")"
    display="$(display_path "$path")"
    printf 'removing %s  %-6s  %s\n' "$label" "$category" "$display"
    rm -rf -- "$path"
  done < "$SORTED"
}

pressure_text() {
  echo "Target: $TARGET_NAME"
  echo
  echo "Filesystem"
  df -h "$HOME_DIR" 2>/dev/null || true
  echo
  echo "Filesystem type"
  stat -f -c '%T' "$HOME_DIR" 2>/dev/null || true
  echo
  echo "Top cache roots"
  du -sh "$HOME_DIR/.cache" "$HOME_DIR/.cargo" "$HOME_DIR/.npm" "$HOME_DIR/.local/share/containers" 2>/dev/null | sort -h || true
  echo
  echo "Top Git children"
  du -sh "$HOME_DIR/Git"/* "$HOME_DIR/Git"/*/* 2>/dev/null | sort -h | tail -20 || true
}

pressure_json() {
  local free fs_type
  free="$(df -B1 --output=avail "$HOME_DIR" 2>/dev/null | awk 'NR == 2 { print $1 + 0 }')"
  fs_type="$(stat -f -c '%T' "$HOME_DIR" 2>/dev/null || true)"
  printf '{"mode":"pressure","target":"%s","home":"%s","free_bytes":%s,"min_free_gb":%s,"fs_type":"%s"}\n' \
    "$(json_escape "$TARGET_NAME")" "$(json_escape "$HOME_DIR")" "${free:-0}" "$MIN_FREE_GB" "$(json_escape "$fs_type")"
}

target_check() {
  local missing=0 tool fs_type
  if [[ "$OUTPUT_FORMAT" == "json" ]]; then
    fs_type="$(stat -f -c '%T' "$HOME_DIR" 2>/dev/null || true)"
    printf '{"mode":"target-check","target":"%s","home":"%s","fs_type":"%s","tools":{' "$(json_escape "$TARGET_NAME")" "$(json_escape "$HOME_DIR")" "$(json_escape "$fs_type")"
    local first=1
    for tool in bash find du awk sort df stat rm; do
      if [[ "$first" == 0 ]]; then printf ','; fi
      first=0
      if command -v "$tool" >/dev/null 2>&1; then
        printf '"%s":true' "$tool"
      else
        printf '"%s":false' "$tool"
        missing=1
      fi
    done
    printf '},"ok":'
    if [[ "$missing" == 0 && -d "$HOME_DIR" ]]; then printf 'true'; else printf 'false'; fi
    printf '}\n'
  else
    echo "Target: $TARGET_NAME"
    echo "Home: $HOME_DIR"
    echo "Host: $(hostname 2>/dev/null || true)"
    echo "Tools:"
    for tool in bash find du awk sort df stat rm; do
      if command -v "$tool" >/dev/null 2>&1; then
        echo "  $tool: ok"
      else
        echo "  $tool: missing"
        missing=1
      fi
    done
    echo "Filesystem:"
    df -h "$HOME_DIR" 2>/dev/null || true
  fi
  return "$missing"
}

case "$MODE" in
  target-check)
    target_check
    ;;
  pressure)
    if [[ "$OUTPUT_FORMAT" == "json" ]]; then
      pressure_json
    else
      pressure_text
    fi
    ;;
  report|clean)
    collect_candidates
    size_candidates
    if [[ "$MODE" == "report" ]]; then
      if [[ "$OUTPUT_FORMAT" == "json" ]]; then print_report_json; else print_report_text; fi
    elif [[ "$DRY_RUN" == "1" ]]; then
      if [[ "$OUTPUT_FORMAT" == "json" ]]; then
        print_report_json
      else
        print_report_text
        echo
        echo "Dry run"
        while IFS=$'\t' read -r size category path; do
          [[ -n "${path:-}" ]] || continue
          printf 'would remove %s  %-6s  %s\n' "$(format_bytes "$size")" "$category" "$(display_path "$path")"
        done < "$SORTED"
      fi
    else
      clean_paths
    fi
    ;;
  *)
    echo "unsupported remote mode: $MODE" >&2
    exit 64
    ;;
esac
