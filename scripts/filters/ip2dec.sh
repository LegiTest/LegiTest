#!/bin/bash

# Convert IPs to decimal format for faster server checks
ip2dec () {
    ip=`echo $@ | sed 's/\(.*\)\/.*/\1/g'`
    cidr=`echo $@ | sed 's/.*\/\(.*\)/\1/g'`
    local a b c d
    IFS=. read -r a b c d <<< "$ip"
    if [[ "$@" != */* ]] || [ $cidr == "32" ]; then
        printf '%d,\n' "$((a * 256 ** 3 + b * 256 ** 2 + c * 256 + d))"
    else
        printf '%d,%d\n' "$((a * 256 ** 3 + b * 256 ** 2 + c * 256 + d))" "$((a * 256 ** 3 + b * 256 ** 2 + c * 256 + d + 2 ** (32 - $cidr) - 1))"
    fi
}

lines=$(cat $@)
for line in $lines
do
    ip2dec "$line"
done
