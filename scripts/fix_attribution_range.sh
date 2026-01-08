#!/usr/bin/env bash
set -euo pipefail

# Rewrite author/committer to devicemanager for all commits in a range
# that GitHub shows with a specific committer login (default: shubh2294).
# Usage: ./scripts/fix_attribution_range.sh [BASE_SHA_PREFIX] [HEAD_SHA_PREFIX]
# Env overrides: DM_NAME, DM_EMAIL, REMOTE, COMMITTER_LOGIN, NO_PROMPT=1

DM_NAME="${DM_NAME:-devicemanager}"
DM_EMAIL="${DM_EMAIL:-1844069+devicemanager@users.noreply.github.com}"
REMOTE="${REMOTE:-origin}"
COMMITTER_LOGIN="${COMMITTER_LOGIN:-shubh2294}"
BASE="${1:-d7cd326}"
HEAD="${2:-3787a93}"

# Preconditions
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "Error: run this from the git repository root" >&2; exit 1
fi
if ! git remote get-url "$REMOTE" >/dev/null 2>&1; then
  echo "Error: remote '$REMOTE' not found" >&2; exit 1
fi
if ! command -v gh >/dev/null 2>&1; then
  echo "Error: GitHub CLI (gh) is required" >&2; exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required" >&2; exit 1
fi

# Derive owner/repo from remote URL
REMOTE_URL=$(git remote get-url "$REMOTE")
OWNER=""; REPO=""
case "$REMOTE_URL" in
  git@github.com:*/*.git)
    OWNER=$(printf "%s" "$REMOTE_URL" | sed -E 's#^git@github.com:([^/]+)/([^/]+)\.git$#\1#')
    REPO=$(printf "%s" "$REMOTE_URL" | sed -E 's#^git@github.com:([^/]+)/([^/]+)\.git$#\2#') ;;
  https://github.com/*/*.git)
    OWNER=$(printf "%s" "$REMOTE_URL" | sed -E 's#^https://github.com/([^/]+)/([^/]+)\.git$#\1#')
    REPO=$(printf "%s" "$REMOTE_URL" | sed -E 's#^https://github.com/([^/]+)/([^/]+)\.git$#\2#') ;;
  https://github.com/*/*)
    OWNER=$(printf "%s" "$REMOTE_URL" | sed -E 's#^https://github.com/([^/]+)/([^/]+)$#\1#')
    REPO=$(printf "%s" "$REMOTE_URL" | sed -E 's#^https://github.com/([^/]+)/([^/]+)$#\2#') ;;
  *) echo "Error: cannot parse owner/repo from '$REMOTE_URL'" >&2; exit 1 ;;
esac

# Build ancestry path between BASE and HEAD using local git
SHAS_RANGE=$(git rev-list --ancestry-path "${BASE}..${HEAD}" --reverse || true)
if [ -z "$SHAS_RANGE" ]; then
  # Try reversed order
  SHAS_RANGE=$(git rev-list --ancestry-path "${HEAD}..${BASE}" --reverse || true)
fi
if [ -z "$SHAS_RANGE" ]; then
  echo "Error: could not resolve commit range between ${BASE} and ${HEAD}" >&2; exit 1
fi

echo "Scanning range for commits with committer.login='${COMMITTER_LOGIN}' on GitHub..."
MATCHES=()
for s in $SHAS_RANGE; do
  # Query GitHub for this commit's resolved committer login
  J=$(gh api "repos/${OWNER}/${REPO}/commits/${s}" 2>/dev/null || true)
  [ -z "$J" ] && continue
  LOGIN=$(printf '%s' "$J" | jq -r '.committer.login // empty')
  if [ "$LOGIN" = "$COMMITTER_LOGIN" ]; then
    MATCHES+=("$s")
  fi
done

if [ "${#MATCHES[@]}" -eq 0 ]; then
  echo "No matching commits found in range. Diagnostic (sha -> login):" >&2
  for s in $SHAS_RANGE; do
    J=$(gh api "repos/${OWNER}/${REPO}/commits/${s}" 2>/dev/null || true)
    L=$(printf '%s' "$J" | jq -r '.committer.login // ""')
    printf '  %s -> %s\n' "$s" "$L" >&2
  done
  exit 0
fi

echo "Commits to rewrite (oldest->newest):"
for s in "${MATCHES[@]}"; do
  git show -s --format="%h | %an <%ae> | %cn <%ce> | %s" "$s"
done

if [ "${NO_PROMPT:-}" != "1" ]; then
  echo
  read -r -p "Proceed to rewrite these commits to ${DM_NAME} <${DM_EMAIL}> and force-push to '${REMOTE}'? Type 'yes' to continue: " ANSWER
  if [ "$ANSWER" != "yes" ]; then
    echo "Aborted."; exit 1
  fi
fi

# Build env-filter script matching target SHAs
TMP_ENV=$(mktemp)
{
  echo 'case "$GIT_COMMIT" in'
  for s in "${MATCHES[@]}"; do
    echo "  ${s})"
    echo "    export GIT_AUTHOR_NAME=${DM_NAME}"
    echo "    export GIT_AUTHOR_EMAIL=${DM_EMAIL}"
    echo "    export GIT_COMMITTER_NAME=${DM_NAME}"
    echo "    export GIT_COMMITTER_EMAIL=${DM_EMAIL}"
    echo "    ;;"
  done
  echo 'esac'
} > "$TMP_ENV"

# Remove Co-Authored-By trailers case-insensitively
MSG_FILTER='sed -E "/^[Cc]o-[Aa]uthored-[Bb]y:/d"'

# Rewrite only targeted commits
GIT_AUTHOR_NAME="$DM_NAME" GIT_AUTHOR_EMAIL="$DM_EMAIL" \
GIT_COMMITTER_NAME="$DM_NAME" GIT_COMMITTER_EMAIL="$DM_EMAIL" \
  git filter-branch -f \
    --env-filter "$(cat "$TMP_ENV")" \
    --msg-filter "$MSG_FILTER" \
    --tag-name-filter cat -- --all
rm -f "$TMP_ENV"

# Force push rewritten history
echo "Force-pushing rewritten history to '${REMOTE}'..."
git push --force-with-lease --tags "$REMOTE" --all

# Verify
echo "Verification (last 20 commits on ${REMOTE}/main):"
git fetch -q "$REMOTE"
git log "$REMOTE/main" -n 20 --format='%h | %an <%ae> | %cn <%ce> | %s'
