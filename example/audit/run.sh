#!/bin/bash
cd "$(dirname "$0")"/../..

echo "1. Downloading raw dataset sample from Huggingface (epfml/FineWeb-HQ)..."
curl -s "https://datasets-server.huggingface.co/rows?dataset=epfml/FineWeb-HQ&config=default&split=train&offset=0&length=5" -o example/audit/hf_raw_sample.json

if command -v cargo &> /dev/null
then
  echo "2. Running FastMemory Clustering Engine on the parsed input.md ATFs..."
  cargo run -q -- "example/audit/input.md" > "example/audit/output.json"
  
  echo "3. Refreshing Javascript memory state for UI..."
  echo "const fastMemoryData = $(cat example/audit/output.json);" > "example/audit/output.js"
  
  echo "Successfully regenerated example/audit/output.json and output.js!"
else
  echo "Warning: cargo CLI not found. Skipping rust regeneration."
fi
