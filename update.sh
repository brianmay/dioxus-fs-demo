#!/bin/sh
set -e

#if ! git diff-index --quiet HEAD --; then
#  echo "ERROR: Something has changed" >&2
#  exit 1
#fi

npm --prefix ./ --lockfile-version 2 update
rm -rf node_modules
hash="$(prefetch-npm-deps ./package-lock.json)"
echo "updated npm dependency hash: $hash" >&2
echo "$hash" >npm-deps-hash

git add package-lock.json
git add npm-deps-hash
