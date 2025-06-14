#!/bin/bash
# Test for standardized documentation format in network docs
set -e
REQUIRED_SECTIONS=("Overview" "Types" "Implementation" "Examples" "Testing")
FILES=("network-manager.md" "metrics.md" "protocols.md" "network-types.md")
cd docs/implementation/network
fail=0
for file in "${FILES[@]}"; do
  for section in "${REQUIRED_SECTIONS[@]}"; do
    if ! grep -q "^##* $section" "$file"; then
      echo "[FAIL] $file missing section: $section"
      fail=1
    fi
  done
done
if [ $fail -eq 0 ]; then
  echo "[PASS] All documentation files have required sections."
  exit 0
else
  exit 1
fi
