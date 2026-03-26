#!/bin/bash
cd "$(dirname "$0")"/../..

echo "1. Downloading raw dataset sample from Huggingface (OpenSQZ/AutoMathText-V2)..."
curl -s "https://datasets-server.huggingface.co/rows?dataset=OpenSQZ/AutoMathText-V2&config=automathtext-v2-ultra&split=train&offset=0&length=5" -o example/business_analytics/hf_raw_sample.json

if command -v cargo &> /dev/null
then
  echo "2. Running FastMemory Clustering Engine on the parsed input.md ATFs..."
  cargo run -q -- "example/business_analytics/input.md" > "example/business_analytics/output.json"
  
  echo "3. Refreshing Javascript memory state for UI..."
  echo "const fastMemoryData = $(cat example/business_analytics/output.json);" > "example/business_analytics/output.js"
  
  echo "Successfully regenerated example/business_analytics/output.json and output.js!"
else
  echo "Warning: cargo CLI not found. Skipping rust regeneration."
fi
