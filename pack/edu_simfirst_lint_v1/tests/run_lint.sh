#!/usr/bin/env bash
set -euo pipefail
python3 pack/edu_simfirst_lint_v1/tools/edu_simfirst_lint.py --root docs/ssot/pack --mode error
