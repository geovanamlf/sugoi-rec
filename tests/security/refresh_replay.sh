#!/usr/bin/env bash

set -euo pipefail

API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"
TEST_EMAIL="${TEST_EMAIL:-}"
TEST_PASSWORD="${TEST_PASSWORD:-}"

pass_count=0
fail_count=0

if [ -z "$TEST_EMAIL" ] || [ -z "$TEST_PASSWORD" ]; then
  echo "Erro: defina TEST_EMAIL e TEST_PASSWORD antes de rodar."
  echo
  echo "Use uma conta de teste. Não use sua conta principal."
  echo
  echo 'Exemplo:'
  echo 'TEST_EMAIL="teste@example.com" TEST_PASSWORD="senha" ./tests/security/refresh_replay.sh'
  exit 1
fi

pass() {
  echo "PASS: $1"
  pass_count=$((pass_count + 1))
}

fail() {
  echo "FAIL: $1"
  fail_count=$((fail_count + 1))
}

json_field() {
  local file_path="$1"
  local field_name="$2"

  python3 - "$file_path" "$field_name" <<'PY'
import json
import sys

file_path = sys.argv[1]
field_name = sys.argv[2]

with open(file_path, "r", encoding="utf-8") as file:
    data = json.load(file)

print(data[field_name])
PY
}

echo "Testando refresh token replay em $API_BASE_URL"
echo "Atenção: este teste revoga refresh tokens ativos do usuário de teste."
echo

login_file="/tmp/sugoi_refresh_replay_login.json"

login_status=$(curl -s -o "$login_file" -w "%{http_code}" \
  -X POST "$API_BASE_URL/auth/login" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  --data-urlencode "username=$TEST_EMAIL" \
  --data-urlencode "password=$TEST_PASSWORD")

if [ "$login_status" != "200" ]; then
  fail "login deveria retornar 200, mas retornou $login_status"
  cat "$login_file"
  echo
  exit 1
fi

old_refresh_token=$(json_field "$login_file" "refresh_token")

first_refresh_file="/tmp/sugoi_refresh_replay_first.json"

first_refresh_status=$(curl -s -o "$first_refresh_file" -w "%{http_code}" \
  -X POST "$API_BASE_URL/auth/refresh" \
  -H "Content-Type: application/json" \
  --data "{\"refresh_token\":\"$old_refresh_token\"}")

if [ "$first_refresh_status" = "200" ]; then
  pass "primeiro refresh retornou 200"
else
  fail "primeiro refresh deveria retornar 200, mas retornou $first_refresh_status"
  cat "$first_refresh_file"
  echo
  exit 1
fi

new_refresh_token=$(json_field "$first_refresh_file" "refresh_token")

replay_file="/tmp/sugoi_refresh_replay_old_again.json"

replay_status=$(curl -s -o "$replay_file" -w "%{http_code}" \
  -X POST "$API_BASE_URL/auth/refresh" \
  -H "Content-Type: application/json" \
  --data "{\"refresh_token\":\"$old_refresh_token\"}")

if [ "$replay_status" = "401" ]; then
  pass "reutilizar refresh token antigo retornou 401"
else
  fail "reutilizar refresh token antigo deveria retornar 401, mas retornou $replay_status"
  cat "$replay_file"
  echo
fi

new_after_replay_file="/tmp/sugoi_refresh_replay_new_after_replay.json"

new_after_replay_status=$(curl -s -o "$new_after_replay_file" -w "%{http_code}" \
  -X POST "$API_BASE_URL/auth/refresh" \
  -H "Content-Type: application/json" \
  --data "{\"refresh_token\":\"$new_refresh_token\"}")

if [ "$new_after_replay_status" = "401" ]; then
  pass "refresh token novo também foi revogado após replay"
else
  fail "refresh token novo deveria retornar 401 após replay, mas retornou $new_after_replay_status"
  cat "$new_after_replay_file"
  echo
fi

echo
echo "Resumo:"
echo "PASS: $pass_count"
echo "FAIL: $fail_count"

if [ "$fail_count" -gt 0 ]; then
  exit 1
fi
