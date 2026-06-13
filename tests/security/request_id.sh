#!/usr/bin/env bash

set -euo pipefail

API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"

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

echo "Testando X-Request-Id em $API_BASE_URL"
echo

generated_headers_file="/tmp/sugoi_request_id_generated_headers.txt"

curl -s -D "$generated_headers_file" -o /tmp/sugoi_request_id_generated_body.json \
  "$API_BASE_URL/health"

generated_request_id=$(grep -i '^x-request-id:' "$generated_headers_file" | awk -F': ' '{print $2}' | tr -d '\r')

if [ -n "$generated_request_id" ]; then
  pass "backend gerou x-request-id quando cliente não enviou"
else
  fail "backend não retornou x-request-id gerado"
fi

custom_headers_file="/tmp/sugoi_request_id_custom_headers.txt"

curl -s -D "$custom_headers_file" -o /tmp/sugoi_request_id_custom_body.json \
  "$API_BASE_URL/health" \
  -H "X-Request-Id: teste-security-123"

custom_request_id=$(grep -i '^x-request-id:' "$custom_headers_file" | awk -F': ' '{print $2}' | tr -d '\r')

if [ "$custom_request_id" = "teste-security-123" ]; then
  pass "backend reaproveitou X-Request-Id enviado pelo cliente"
else
  fail "backend deveria retornar teste-security-123, mas retornou '$custom_request_id'"
fi

expose_header=$(grep -i '^access-control-expose-headers:' "$custom_headers_file" | awk -F': ' '{print $2}' | tr -d '\r')

if echo "$expose_header" | grep -qi 'x-request-id'; then
  pass "CORS expõe x-request-id para o frontend"
else
  fail "CORS não expõe x-request-id"
fi

echo
echo "Resumo:"
echo "PASS: $pass_count"
echo "FAIL: $fail_count"

if [ "$fail_count" -gt 0 ]; then
  exit 1
fi
