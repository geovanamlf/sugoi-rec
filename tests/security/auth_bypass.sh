#!/usr/bin/env bash

set -euo pipefail

API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"

pass_count=0
fail_count=0

check_status() {
  local label="$1"
  local endpoint="$2"
  local expected_status="$3"
  shift 3

  response_file="/tmp/sugoi_auth_bypass_response.json"

  status=$(curl -s -o "$response_file" -w "%{http_code}" \
    "$API_BASE_URL$endpoint" \
    "$@")

  if [ "$status" = "$expected_status" ]; then
    echo "PASS: $label retornou $expected_status"
    pass_count=$((pass_count + 1))
  else
    echo "FAIL: $label deveria retornar $expected_status, mas retornou $status"
    echo "Resposta:"
    cat "$response_file"
    echo
    fail_count=$((fail_count + 1))
  fi
}

echo "Testando auth bypass em $API_BASE_URL"
echo

protected_endpoints=(
  "/auth/me"
  "/list"
  "/list/"
  "/analytics/genres"
  "/analytics/ratings"
  "/analytics/status"
  "/anime/search?q=bleach"
  "/anime/id/1"
  "/recommendations/"
  "/recommendations/?refresh=true"
)

for endpoint in "${protected_endpoints[@]}"; do
  check_status "GET $endpoint sem token" "$endpoint" "401"
done

echo

for endpoint in "${protected_endpoints[@]}"; do
  check_status "GET $endpoint com token inválido" "$endpoint" "401" \
    -H "Authorization: Bearer invalid-token"
done

echo
echo "Resumo:"
echo "PASS: $pass_count"
echo "FAIL: $fail_count"

if [ "$fail_count" -gt 0 ]; then
  exit 1
fi
