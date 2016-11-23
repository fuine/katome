#!/bin/bash
PKGID="$(cargo pkgid)"
[ -z "$PKGID" ] && exit 1
ORIGIN="${PKGID%#*}"
ORIGIN="${ORIGIN:7}"
PKGNAMEVER="${PKGID#*#}"
PKGNAME="${PKGNAMEVER%:*}"
shift
cargo test --no-run || exit $?
EXE=($ORIGIN/target/debug/$PKGNAME-*)
if [ ${#EXE[@]} -ne 1 ]; then
    echo 'Non-unique test file, retrying...' >2
    rm -f ${EXE[@]}
    cargo test --no-run || exit $?
fi
rm -rf $ORIGIN/target/cov
rm -rf $ORIGIN/target/cov_all
mkdir $ORIGIN/target/cov
mkdir $ORIGIN/target/cov_all
for f in $ORIGIN/target/debug/*-*; do
    BASE=$(basename $f)
    kcov --exclude-pattern=/.cargo $ORIGIN/target/cov/$BASE/ $f "$@"
done
kcov --merge $ORIGIN/target/cov_all $ORIGIN/target/cov/*
