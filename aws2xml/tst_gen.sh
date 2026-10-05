#!/usr/bin/env bash

set -e

# cargo run -- --namespace my-namespace
cargo run | tee aws2xml.log
sad2md temporary_sad_aws_account_dump.xml

