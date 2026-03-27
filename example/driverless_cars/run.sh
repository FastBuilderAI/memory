#!/bin/bash
cd "$(dirname "$0")"/../..

echo "1. Downloading raw dataset sample from Huggingface (openai/gsm8k)..."
curl -s "https://datasets-server.huggingface.co/rows?dataset=openai/gsm8k&config=main&split=train&offset=0&length=5" -o example/driverless_cars/hf_raw_sample.json

if command -v cargo &> /dev/null
then
  echo "2. Running FastMemory Clustering Engine on the parsed input.md ATFs..."
  cargo run -q -- "example/driverless_cars/input.md" > "example/driverless_cars/output.json"
  
  echo "3. Refreshing Javascript memory state for UI..."
  echo "const fastMemoryData = $(cat example/driverless_cars/output.json);" > "example/driverless_cars/output.js"
  
  echo "Successfully regenerated example/driverless_cars/output.json and output.js!"
else
  echo "Warning: cargo CLI not found. Skipping rust regeneration."
fi
