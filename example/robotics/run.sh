#!/bin/bash
cd "$(dirname "$0")"/../..

echo "1. Downloading raw dataset sample from Huggingface (m-a-p/FineFineWeb)..."
curl -s "https://datasets-server.huggingface.co/rows?dataset=m-a-p/FineFineWeb&config=default&split=train&offset=0&length=5" -o example/robotics/hf_raw_sample.json

if command -v cargo &> /dev/null
then
  echo "2. Running FastMemory Clustering Engine on the parsed input.md ATFs..."
  cargo run -q -- "example/robotics/input.md" > "example/robotics/output.json"
  
  echo "3. Refreshing Javascript memory state for UI..."
  echo "const fastMemoryData = $(cat example/robotics/output.json);" > "example/robotics/output.js"
  
  echo "Successfully regenerated example/robotics/output.json and output.js!"
else
  echo "Warning: cargo CLI not found. Skipping rust regeneration."
fi
