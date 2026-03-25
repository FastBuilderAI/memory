#!/bin/bash
cd "$(dirname "$0")"/../..

echo "1. Downloading raw dataset sample from Huggingface (allenai/c4)..."
curl -s "https://datasets-server.huggingface.co/rows?dataset=allenai/c4&config=af&split=train&offset=0&length=5" -o example/email_analysis/hf_raw_sample.json

if command -v cargo &> /dev/null
then
  echo "2. Running FastMemory Clustering Engine on the parsed input.md ATFs..."
  cargo run -q -- "example/email_analysis/input.md" > "example/email_analysis/output.json"
  
  echo "3. Refreshing Javascript memory state for UI..."
  echo "const fastMemoryData = $(cat example/email_analysis/output.json);" > "example/email_analysis/output.js"
  
  echo "Successfully regenerated example/email_analysis/output.json and output.js!"
else
  echo "Warning: cargo CLI not found. Skipping rust regeneration."
fi
