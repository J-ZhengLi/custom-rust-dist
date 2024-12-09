#!/bin/bash

NODE_VERSION="v18.19.0"

if [ "$(uname -s)" = "Linux" ]; then
    echo "Skip..."
else
    NODE_URL="https://nodejs.org/dist/$NODE_VERSION/node-$NODE_VERSION-win-x64.zip"
    USER_NODEJS_PATH="/c/Users/$(whoami)/nodejs"
    DOWNLOAD_PATH="$USER_NODEJS_PATH/node-$NODE_VERSION-x64.zip"
    mkdir -p "$USER_NODEJS_PATH"
    curl -L -o "$DOWNLOAD_PATH" "$NODE_URL"
    unzip "$DOWNLOAD_PATH" -d "$USER_NODEJS_PATH"
    rm "$DOWNLOAD_PATH"
    NODE_BIN_PATH="$USER_NODEJS_PATH/node-$NODE_VERSION-x64"
    # export PATH="$NODE_BIN_PATH:$PATH"
    CURRENT_PATH=$(powershell -Command "[System.Environment]::GetEnvironmentVariable('PATH', 'User')")
    NEW_PATH="$CURRENT_PATH;$NODE_BIN_PATH"
    powershell -Command "[System.Environment]::SetEnvironmentVariable('PATH', '$NEW_PATH', 'User')"
fi

