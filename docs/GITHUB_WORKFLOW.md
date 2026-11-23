# Push changes and merge through a pull request

The `main` branch is protected: GitHub rejects direct pushes to it. Push your
work to a feature branch, open a pull request (PR) targeting `main`, and merge
the PR after required checks and reviews pass.

## Move commits already on local `main` to a feature branch

If you committed changes while on local `main` and the push was rejected, those
commits remain on your computer. Create a branch at the current commit; this
keeps the commits and gives you a branch you can push:

```sh
git status --short --branch
git switch -c auto-counter-fix
git push -u origin auto-counter-fix
gh pr create --base main --head auto-counter-fix --title "Auto-counter fix" --fill
```

Use a descriptive branch name and PR title for your work. The `gh pr create`
command prints a link to the new PR. If GitHub CLI (`gh`) is not installed or
authenticated, push the branch with Git and open a PR from that branch on
GitHub.

If the changes are not committed yet, commit them on the feature branch before
pushing. To avoid committing unrelated changes, inspect `git status` and stage
only the intended files.

## Review and merge the pull request

1. Open the PR link printed by `gh pr create` (or open the repository's Pull
   requests page on GitHub).
2. Confirm the base is `main`, review the changed files and description, and
   wait for required checks and approvals.
3. Address any requested changes by committing and pushing to the same feature
   branch. The PR updates automatically.
4. When GitHub enables it, use **Merge pull request** and confirm the merge.
   Follow repository policy if it requires a specific merge method or a
   maintainer to merge. Do not push directly to `main` to bypass a blocked PR.

## Update local `main` after the PR is merged

Once the PR is merged on GitHub, update your local branch:

```sh
git switch main
git pull --ff-only origin main
```

`--ff-only` updates local `main` only when it can advance without creating a
merge commit. If it refuses because local `main` has commits not present on
GitHub, do not reset or discard those commits. Keep them on a feature branch or
ask the repository maintainer how to reconcile them.

## Common outcomes

- **Push rejected with “Changes must be made through a pull request”:** the
  remote `main` is protected. Keep local commits, create/push a feature branch,
  and open a PR as shown above.
- **PR cannot be merged:** inspect the PR page for failing checks, missing
  approvals, conflicts, or repository-specific requirements. Fix conflicts on
  the feature branch, then push the updated branch.
- **`git switch -c` says the branch already exists:** choose a different branch
  name, or switch to the existing branch only after confirming it contains the
  intended work.
- **The feature branch is behind `main`:** update the feature branch using the
  merge/rebase process allowed by the repository before asking for review; do
  not rewrite a shared branch without coordination.
