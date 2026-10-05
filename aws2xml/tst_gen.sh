#!/usr/bin/env bash

set -e

export AWS2XML_LOG_LEVEL=debug

export OTLP_TRACE_BACKEND_URL=http://127.0.0.1:4319
export OTLP_METRICS_BACKEND_URL=http://127.0.0.1:4319
export OTLP_LOGGING_BACKEND_URL=http://127.0.0.1:4319

# cargo run -- --namespace my-namespace
cargo run | tee aws2xml.log
sad2md temporary_sad_aws_account_dump.xml

