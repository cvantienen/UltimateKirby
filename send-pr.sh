#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Create a branch from the current checkout, commit all working-tree changes,
push it to origin, and open a pull request targeting main.

Usage:
  ./send-pr.sh --title "PR title" [options]

Options:
  -t, --title TITLE          Pull request title (required)
  -m, --body TEXT            Pull request description (optional)
      --description TEXT     Alias for --body
  -b, --branch NAME          Branch name (default: generated from title)
      --base BRANCH          Pull request base branch (default: main)
      --draft                Create a draft pull request
  -h, --help                 Show this help

Examples:
  ./send-pr.sh -t "Improve spot dodge handling" -m "Adds safer timing checks."
  ./send-pr.sh -t "Fix ledge input" -b fix/ledge-input --draft

The script stages and commits all non-ignored changes in the repository.
It requires git, an origin remote, GitHub CLI (gh), and an authenticated gh session.
EOF
}

fail() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

TITLE=''
BODY=''
BRANCH=''
BASE='main'
DRAFT=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    -t|--title)
      [ "$#" -ge 2 ] || fail "--title requires a value"
      TITLE=$2
      shift 2
      ;;
    --title=*)
      TITLE=${1#*=}
      shift
      ;;
    -m|--body|--description)
      [ "$#" -ge 2 ] || fail "$1 requires a value"
      BODY=$2
      shift 2
      ;;
    --body=*|--description=*)
      BODY=${1#*=}
      shift
      ;;
    -b|--branch)
      [ "$#" -ge 2 ] || fail "--branch requires a value"
      BRANCH=$2
      shift 2
      ;;
    --branch=*)
      BRANCH=${1#*=}
      shift
      ;;
    --base)
      [ "$#" -ge 2 ] || fail "--base requires a value"
      BASE=$2
      shift 2
      ;;
    --base=*)
      BASE=${1#*=}
      shift
      ;;
    --draft)
      DRAFT=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1 (use --help for usage)"
      ;;
  esac
done

[ -n "$TITLE" ] || fail "--title is required (use --help for usage)"
[ -n "$BASE" ] || fail "base branch cannot be empty"

if [ -z "$BRANCH" ]; then
  slug=$(printf '%s' "$TITLE" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//')
  [ -n "$slug" ] || slug=changes
  BRANCH="copilot/$slug"
fi

git rev-parse --is-inside-work-tree >/dev/null 2>&1 || fail "run this from inside a git repository"
repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

git remote get-url origin >/dev/null 2>&1 || fail "git remote 'origin' is not configured"
command -v gh >/dev/null 2>&1 || fail "GitHub CLI 'gh' is required"
gh auth status >/dev/null 2>&1 || fail "GitHub CLI is not authenticated; run 'gh auth login' first"

git check-ref-format --branch "$BRANCH" >/dev/null 2>&1 || fail "invalid branch name: $BRANCH"
git show-ref --verify --quiet "refs/heads/$BRANCH" && fail "local branch already exists: $BRANCH"
remote_branch=$(git ls-remote --heads origin "refs/heads/$BRANCH") || fail "could not check branches on origin"
[ -z "$remote_branch" ] || fail "remote branch already exists: $BRANCH"
remote_base=$(git ls-remote --heads origin "refs/heads/$BASE") || fail "could not check base branch on origin"
[ -n "$remote_base" ] || fail "base branch '$BASE' was not found on origin"

[ -n "$(git status --porcelain)" ] || fail "there are no changes to commit"

git switch -c "$BRANCH"
git add -A
git diff --cached --quiet && fail "there are no changes to commit after staging"

if [ -n "$BODY" ]; then
  git commit -m "$TITLE" -m "$BODY"
else
  git commit -m "$TITLE"
fi

git push -u origin "$BRANCH"

if [ "$DRAFT" = true ]; then
  gh pr create --base "$BASE" --head "$BRANCH" --title "$TITLE" --body "${BODY:-Submitted with send-pr.sh.}" --draft
else
  gh pr create --base "$BASE" --head "$BRANCH" --title "$TITLE" --body "${BODY:-Submitted with send-pr.sh.}"
fi
