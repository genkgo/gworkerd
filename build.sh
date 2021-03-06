#!/bin/bash
if [ -z "$1" ]; then
    echo "You forgot the version number"
    exit 1
fi

# make sure we are in the right directory
cd "$(dirname "$0")"

# update monitor submodule to latest commit
git submodule update --remote --merge

# install monitor dependencies and build monitor
cd monitor && npm install && bower install && ember build --environment=production && cd ..

# build daemon
cargo clean && cargo build --release

# tar gz daemon and monitor
tar --transform "s|monitor/dist|monitor|" \
    --transform "s|target/release/gworkerd|gworkerd|" \
    -cvzf release/gworkerd.$1.tar.gz \
    target/release/gworkerd \
    monitor/dist

# zip file for github
zip -j release/gworkerd.$1.zip target/release/gworkerd && \
    cd monitor/ && \
    mv dist monitor && \
    zip -r ../release/gworkerd.$1.zip monitor && \
    mv monitor dist && \
    cd ..