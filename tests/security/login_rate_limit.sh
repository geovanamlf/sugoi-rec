#!/usr/bin/env bash

set -euo pipefail

API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"

pass_count=0
fail_count=0
saw_429=0

echo "Testando rate limit de login em $API_BASE_URL"
echo "Este teste força várias tentativas inválidas de login."
echo

for attempt in 1 2 3 4 5 6 7; do
  response_file="/tmp/sugoi_login_rate_limit_response.json"

  status=$(curl -s -o "$response_file" -w "%{http_code}" \
    -X POST "$API_BASE_URL/auth/login" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode "username=rate-limit-test@example.com" \
    --data-urlencode "password=wrong-password")

  echo "tentativa=$attempt status=$status"

  if [ "$status" = "429" ]; then
    saw_429=1
  fi
done

echo

if [ "$saw_429" = "1" ]; then
  echo "PASS: login rate limit retornou 429"
  pass_count=$((pass_count + 1))
else
  echo "FAIL: login rate limit não retornou 429"
  fail_count=$((fail_count + 1))
fi

echo
echo "Resumo:"
echo "PASS: $pass_count"
echo "FAIL: $fail_count"

if [ "$fail_count" -gt 0 ]; then
  exit 1
fi
