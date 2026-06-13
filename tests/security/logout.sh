#!/usr/bin/env bash

set -euo pipefail

API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"
TEST_EMAIL="${TEST_EMAIL:-}"
TEST_PASSWORD="${TEST_PASSWORD:-}"

pass_count=0
fail_count=0

pass() {
  echo "PASS: $1"
  pass_count=$((pass_count + 1))
}

fail() {
  echo "FAIL: $1"
  fail_count=$((fail_count + 1))
}

require_env() {
  if [ -z "$TEST_EMAIL" ] || [ -z "$TEST_PASSWORD" ]; then
    echo "Erro: defina TEST_EMAIL e TEST_PASSWORD."
    echo
    echo "Exemplo:"
    echo 'TEST_EMAIL="teste6@email.com" TEST_PASSWORD="123456" ./tests/security/logout.sh'
    exit 1
  fi
}

extract_json_field() {
  local json="$1"
  local field="$2"

  python3 - "$field" <<PY
import json
import sys

field = sys.argv[1]
data = json.loads("""$json""")
print(data.get(field, ""))
PY
}

check_status() {
  local label="$1"
  local actual_status="$2"
  local expected_status="$3"

  if [ "$actual_status" = "$expected_status" ]; then
    pass "$label retornou $expected_status"
  else
    fail "$label deveria retornar $expected_status, mas retornou $actual_status"
  fi
}

require_env

echo "Testando logout em $API_BASE_URL"
echo

login_response=$(curl -s -w "\n%{http_code}" \
  -X POST "$API_BASE_URL/auth/login" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  --data "username=$TEST_EMAIL&password=$TEST_PASSWORD")

login_body=$(echo "$login_response" | sed '$d')
login_status=$(echo "$login_response" | tail -n 1)

check_status "login" "$login_status" "200"

access_token=$(extract_json_field "$login_body" "access_token")
refresh_token=$(extract_json_field "$login_body" "refresh_token")

if [ -n "$access_token" ]; then
  pass "login retornou access_token"
else
  fail "login não retornou access_token"
fi

if [ -n "$refresh_token" ]; then
  pass "login retornou refresh_token"
else
  fail "login não retornou refresh_token"
fi

logout_without_auth_status=$(curl -s -o /tmp/sugoi_logout_without_auth.json -w "%{http_code}" \
  -X POST "$API_BASE_URL/auth/logout" \
  -H "Content-Type: application/json" \
  --data "{\"refresh_token\":\"$refresh_token\"}")

check_status "logout sem access token" "$logout_without_auth_status" "401"

logout_status=$(curl -s -o /tmp/sugoi_logout_response.json -w "%{http_code}" \
  -X POST "$API_BASE_URL/auth/logout" \
  -H "Authorization: Bearer $access_token" \
  -H "Content-Type: application/json" \
  --data "{\"refresh_token\":\"$refresh_token\"}")

check_status "logout com access token e refresh token" "$logout_status" "204"

refresh_after_logout_status=$(curl -s -o /tmp/sugoi_refresh_after_logout.json -w "%{http_code}" \
  -X POST "$API_BASE_URL/auth/refresh" \
  -H "Content-Type: application/json" \
  --data "{\"refresh_token\":\"$refresh_token\"}")

check_status "refresh token depois do logout" "$refresh_after_logout_status" "401"

me_after_logout_status=$(curl -s -o /tmp/sugoi_me_after_logout.json -w "%{http_code}" \
  "$API_BASE_URL/auth/me" \
  -H "Authorization: Bearer $access_token")

check_status "access token depois do logout" "$me_after_logout_status" "200"

echo
echo "Resumo:"
echo "PASS: $pass_count"
echo "FAIL: $fail_count"

if [ "$fail_count" -gt 0 ]; then
  exit 1
fi
