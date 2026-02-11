#!/bin/bash
# Quick Calculator Plugin — evaluates math expressions using bc.
#
# Protocol:
#   - Reads JSON from stdin: { command, query, preferences }
#   - Writes JSON to stdout: { items: [...] }

# Read stdin
read -r input

# Extract query using simple string manipulation (no jq dependency)
query=$(echo "$input" | sed 's/.*"query":"\([^"]*\)".*/\1/')

if [ -z "$query" ]; then
  echo '{"items":[{"uid":"help","title":"Type a math expression","subtitle":"Example: 2+2, sqrt(16), 3*4/2","icon":{"emoji":"🧮"},"actions":[]}]}'
  exit 0
fi

# Try to evaluate with bc (arbitrary precision calculator)
result=$(echo "scale=6; $query" 2>/dev/null | bc 2>/dev/null)

if [ -n "$result" ]; then
  # Remove trailing zeros after decimal
  clean_result=$(echo "$result" | sed 's/\.0*$//;s/\(\.[0-9]*[1-9]\)0*$/\1/')

  cat <<RESPONSE
{"items":[{"uid":"result","title":"= ${clean_result}","subtitle":"${query}","icon":{"emoji":"🧮"},"badge":"Result","actions":[{"type":"copy","text":"${clean_result}","title":"Copy result"}]},{"uid":"expr","title":"${query} = ${clean_result}","subtitle":"Copy full expression","icon":{"emoji":"📋"},"actions":[{"type":"copy","text":"${query} = ${clean_result}","title":"Copy expression"}]}]}
RESPONSE
else
  echo '{"items":[{"uid":"error","title":"Invalid expression","subtitle":"Could not evaluate: '"$query"'","icon":{"emoji":"❌"},"actions":[]}]}'
fi
