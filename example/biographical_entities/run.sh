#!/bin/bash
cd "$(dirname "$0")"/../..

echo "1. Downloading raw dataset sample from Huggingface (Salesforce/wikitext)..."
curl -s "https://datasets-server.huggingface.co/rows?dataset=Salesforce/wikitext&config=wikitext-103-raw-v1&split=test&offset=0&length=5" -o example/health_science/hf_raw_sample.json

if command -v cargo &> /dev/null
then
  echo "2. Running FastMemory Clustering Engine on the parsed input.md ATFs..."
  cargo run -q -- "example/health_science/input.md" > "example/health_science/output.json"
  
  echo "3. Refreshing Javascript memory state for UI..."
  echo "const fastMemoryData = $(cat example/health_science/output.json);" > "example/health_science/output.js"
  
  echo "Successfully regenerated example/health_science/output.json and output.js!"
else
  echo "Warning: cargo CLI not found. Skipping rust regeneration."
fi
