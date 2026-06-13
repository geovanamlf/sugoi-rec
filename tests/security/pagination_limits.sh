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
  echo "Exemplo:"
  echo 'TEST_EMAIL="email@teste.com" TEST_PASSWORD="senha" ./tests/security/pagination_limits.sh'
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

login_response_file="/tmp/sugoi_security_login_response.json"

login_status=$(curl -s -o "$login_response_file" -w "%{http_code}" \
  -X POST "$API_BASE_URL/auth/login" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  --data-urlencode "username=$TEST_EMAIL" \
  --data-urlencode "password=$TEST_PASSWORD")

if [ "$login_status" != "200" ]; then
  fail "login deveria retornar 200, mas retornou $login_status"
  cat "$login_response_file"
  echo
  exit 1
fi

access_token=$(python3 - "$login_response_file" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as file:
    data = json.load(file)

print(data["access_token"])
PY
)

echo "Testando limites de paginação em $API_BASE_URL"
echo

default_response_file="/tmp/sugoi_security_list_default.json"

default_status=$(curl -s -o "$default_response_file" -w "%{http_code}" \
  "$API_BASE_URL/list" \
  -H "Authorization: Bearer $access_token")

if [ "$default_status" = "200" ]; then
  if python3 - "$default_response_file" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as file:
    data = json.load(file)

assert isinstance(data.get("items"), list)
assert data.get("limit") == 20
assert data.get("offset") == 0
assert isinstance(data.get("total"), int)
assert len(data.get("items")) <= 20
PY
  then
    pass "GET /list retorna resposta paginada padrão"
  else
    fail "GET /list não retornou o formato paginado esperado"
  fi
else
  fail "GET /list deveria retornar 200, mas retornou $default_status"
fi

limit_one_response_file="/tmp/sugoi_security_list_limit_one.json"

limit_one_status=$(curl -s -o "$limit_one_response_file" -w "%{http_code}" \
  "$API_BASE_URL/list?limit=1&offset=0" \
  -H "Authorization: Bearer $access_token")

if [ "$limit_one_status" = "200" ]; then
  if python3 - "$limit_one_response_file" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as file:
    data = json.load(file)

assert isinstance(data.get("items"), list)
assert data.get("limit") == 1
assert data.get("offset") == 0
assert len(data.get("items")) <= 1
PY
  then
    pass "GET /list?limit=1&offset=0 retorna no máximo 1 item"
  else
    fail "GET /list?limit=1&offset=0 retornou paginação inválida"
  fi
else
  fail "GET /list?limit=1&offset=0 deveria retornar 200, mas retornou $limit_one_status"
fi

check_status() {
  local endpoint="$1"
  local expected="$2"

  response_file="/tmp/sugoi_security_pagination_check.json"

  status=$(curl -s -o "$response_file" -w "%{http_code}" \
    "$API_BASE_URL$endpoint" \
    -H "Authorization: Bearer $access_token")

  if [ "$status" = "$expected" ]; then
    pass "GET $endpoint retornou $expected"
  else
    fail "GET $endpoint deveria retornar $expected, mas retornou $status"
    cat "$response_file"
    echo
  fi
}

check_status "/list?limit=101" "400"
check_status "/list?limit=0" "400"
check_status "/list?offset=-1" "400"

echo
echo "Resumo:"
echo "PASS: $pass_count"
echo "FAIL: $fail_count"

if [ "$fail_count" -gt 0 ]; then
  exit 1
fi
