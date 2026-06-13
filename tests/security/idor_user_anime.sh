#!/usr/bin/env bash

set -euo pipefail

API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"

TEST_VICTIM_EMAIL="${TEST_VICTIM_EMAIL:-}"
TEST_VICTIM_PASSWORD="${TEST_VICTIM_PASSWORD:-}"
TEST_ATTACKER_EMAIL="${TEST_ATTACKER_EMAIL:-}"
TEST_ATTACKER_PASSWORD="${TEST_ATTACKER_PASSWORD:-}"
TEST_VICTIM_ANIME_ID="${TEST_VICTIM_ANIME_ID:-}"

pass_count=0
fail_count=0

if [ -z "$TEST_VICTIM_EMAIL" ] || \
   [ -z "$TEST_VICTIM_PASSWORD" ] || \
   [ -z "$TEST_ATTACKER_EMAIL" ] || \
   [ -z "$TEST_ATTACKER_PASSWORD" ] || \
   [ -z "$TEST_VICTIM_ANIME_ID" ]; then
  echo "Erro: defina as variáveis necessárias."
  echo
  echo "Exemplo:"
  echo 'TEST_VICTIM_EMAIL="vitima@email.com" \'
  echo 'TEST_VICTIM_PASSWORD="senha" \'
  echo 'TEST_ATTACKER_EMAIL="atacante@email.com" \'
  echo 'TEST_ATTACKER_PASSWORD="senha" \'
  echo 'TEST_VICTIM_ANIME_ID="246" \'
  echo './tests/security/idor_user_anime.sh'
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

login() {
  local email="$1"
  local password="$2"
  local output_file="$3"

  curl -s -o "$output_file" -w "%{http_code}" \
    -X POST "$API_BASE_URL/auth/login" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode "username=$email" \
    --data-urlencode "password=$password"
}

extract_access_token() {
  local file_path="$1"

  python3 - "$file_path" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as file:
    data = json.load(file)

print(data["access_token"])
PY
}

find_anime_in_list() {
  local access_token="$1"
  local anime_id="$2"
  local output_payload_file="$3"

  list_file="/tmp/sugoi_idor_list.json"

  status=$(curl -s -o "$list_file" -w "%{http_code}" \
    "$API_BASE_URL/list?limit=100&offset=0" \
    -H "Authorization: Bearer $access_token")

  if [ "$status" != "200" ]; then
    return 2
  fi

  python3 - "$list_file" "$anime_id" "$output_payload_file" <<'PY'
import json
import sys

list_file = sys.argv[1]
anime_id = int(sys.argv[2])
payload_file = sys.argv[3]

with open(list_file, "r", encoding="utf-8") as file:
    data = json.load(file)

for item in data.get("items", []):
    if item.get("anime_id") == anime_id:
        payload = {
            "status": item.get("status"),
            "rating": item.get("rating"),
            "is_favorite": item.get("is_favorite"),
        }

        with open(payload_file, "w", encoding="utf-8") as output:
            json.dump(payload, output)

        sys.exit(0)

sys.exit(1)
PY
}

echo "Testando IDOR em /list/{anime_id}"
echo

victim_login_file="/tmp/sugoi_idor_victim_login.json"
victim_login_status=$(login "$TEST_VICTIM_EMAIL" "$TEST_VICTIM_PASSWORD" "$victim_login_file")

if [ "$victim_login_status" != "200" ]; then
  fail "login da vítima deveria retornar 200, mas retornou $victim_login_status"
  cat "$victim_login_file"
  echo
  exit 1
fi

attacker_login_file="/tmp/sugoi_idor_attacker_login.json"
attacker_login_status=$(login "$TEST_ATTACKER_EMAIL" "$TEST_ATTACKER_PASSWORD" "$attacker_login_file")

if [ "$attacker_login_status" != "200" ]; then
  fail "login do atacante deveria retornar 200, mas retornou $attacker_login_status"
  cat "$attacker_login_file"
  echo
  exit 1
fi

victim_access_token=$(extract_access_token "$victim_login_file")
attacker_access_token=$(extract_access_token "$attacker_login_file")

victim_payload_file="/tmp/sugoi_idor_victim_payload.json"

if find_anime_in_list "$victim_access_token" "$TEST_VICTIM_ANIME_ID" "$victim_payload_file"; then
  pass "anime $TEST_VICTIM_ANIME_ID existe na lista da vítima"
else
  fail "anime $TEST_VICTIM_ANIME_ID não foi encontrado na lista da vítima nos primeiros 100 itens"
  exit 1
fi

attacker_payload_file="/tmp/sugoi_idor_attacker_payload.json"

if find_anime_in_list "$attacker_access_token" "$TEST_VICTIM_ANIME_ID" "$attacker_payload_file"; then
  fail "atacante também tem o anime $TEST_VICTIM_ANIME_ID na própria lista; escolha outro anime_id"
  exit 1
else
  pass "anime $TEST_VICTIM_ANIME_ID não existe na lista do atacante"
fi

attack_response_file="/tmp/sugoi_idor_attack_response.json"

attack_status=$(curl -s -o "$attack_response_file" -w "%{http_code}" \
  -X PATCH "$API_BASE_URL/list/$TEST_VICTIM_ANIME_ID" \
  -H "Authorization: Bearer $attacker_access_token" \
  -H "Content-Type: application/json" \
  --data-binary "@$victim_payload_file")

if [ "$attack_status" = "404" ]; then
  pass "atacante tentando alterar anime da vítima recebeu 404"
else
  fail "atacante deveria receber 404, mas recebeu $attack_status"
  cat "$attack_response_file"
  echo
fi

echo
echo "Resumo:"
echo "PASS: $pass_count"
echo "FAIL: $fail_count"

if [ "$fail_count" -gt 0 ]; then
  exit 1
fi
