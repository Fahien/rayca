#!/bin/sh
# Removing sensitive data from a repository
# https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository
# ..Run multiple times to be effective?
FILE_NAME="big-file.png"
git filter-branch --force --index-filter \
    "git rm --cached --ignore-unmatch $FILE_NAME" \
    --prune-empty --tag-name-filter cat -- --all
