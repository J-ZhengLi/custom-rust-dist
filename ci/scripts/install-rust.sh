#!/bin/bash

# Install rust via rustup.
echo "insecure" > ~/.curlrc

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y