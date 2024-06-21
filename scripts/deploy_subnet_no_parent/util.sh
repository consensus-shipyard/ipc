#!/bin/bash

get_os() {
    unameOut="$(uname -s)"
    case "${unameOut}" in
        Linux*)     os=Linux;;
        Darwin*)    os=Mac;;
        CYGWIN*)    os=Cygwin;;
        MINGW*)     os=MinGw;;
        *)          os="UNKNOWN:${unameOut}"
    esac
    echo "${os}"
}
export -f get_os

xbase64() {
    os=$(get_os)
    if [[ "$os" == "Linux" ]]; then
        base64 -w0
    elif [[ "$os" == "Mac" ]]; then
        base64
    else
        echo "unsupported OS: $os"
        exit 1
    fi
}
export -f xbase64
