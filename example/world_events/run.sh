#!/bin/bash
cd "$(dirname "$0")"/../..

echo "1. Downloading raw dataset sample from Huggingface (HuggingFaceFW/finephrase)..."
curl -s "https://datasets-server.huggingface.co/rows?dataset=HuggingFaceFW/finephrase&config=all&split=train&offset=0&length=5" -o example/world_events/hf_raw_sample.json

if command -v cargo &> /dev/null
then
  echo "2. Running FastMemory Clustering Engine on the parsed input.md ATFs..."
  cargo run -q -- "example/world_events/input.md" > "example/world_events/output.json"
  
  echo "3. Refreshing Javascript memory state for UI..."
  echo "const fastMemoryData = $(cat example/world_events/output.json);" > "example/world_events/output.js"
  
  echo "Successfully regenerated example/world_events/output.json and output.js!"
else
  echo "Warning: cargo CLI not found. Skipping rust regeneration."
fi
