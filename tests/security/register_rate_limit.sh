#!/usr/bin/env bash

set -euo pipefail

API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"

pass_count=0
fail_count=0
saw_429=0

echo "Testando rate limit de cadastro em $API_BASE_URL"
echo "Este teste força várias tentativas em /auth/register."
echo "Ele pode criar um usuário local de teste se ainda não existir."
echo

for attempt in 1 2 3 4 5; do
  response_file="/tmp/sugoi_register_rate_limit_response.json"

  status=$(curl -s -o "$response_file" -w "%{http_code}" \
    -X POST "$API_BASE_URL/auth/register" \
    -H "Content-Type: application/json" \
    --data '{"email":"rate-limit-register@example.com","password":"123456"}')

  echo "tentativa=$attempt status=$status"

  if [ "$status" = "429" ]; then
    saw_429=1
  fi
done

echo

if [ "$saw_429" = "1" ]; then
  echo "PASS: register rate limit retornou 429"
  pass_count=$((pass_count + 1))
else
  echo "FAIL: register rate limit não retornou 429"
  fail_count=$((fail_count + 1))
fi

echo
echo "Resumo:"
echo "PASS: $pass_count"
echo "FAIL: $fail_count"

if [ "$fail_count" -gt 0 ]; then
  exit 1
fi
