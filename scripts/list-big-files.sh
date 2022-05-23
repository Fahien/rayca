#!/bin/sh
# Displays all blob objects in the repository, sorted from smallest to largest
# https://stackoverflow.com/a/42544963/3636299
git rev-list --objects --all | \
  git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | \
    sed -n 's/^blob //p' | \
      sort --numeric-sort --key=2 | \
        cut -c 1-12,41- | \
          $(command -v gnumfmt || echo numfmt) --field=2 --to=iec-i --suffix=B --padding=7 --round=nearest
