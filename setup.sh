#!/bin/bash

PROJECT_ROOT=`dirname $(realpath setup.sh)`

usage() {
cat << EOF
Project root is $PROJECT_ROOT.

    Usage:  $0 <client|client-images|filters|all> [--regenerate]

    client: Generate client-side configuration files in data/
    based on configuration in config/client/. Will download open data
    from the Assemblée Nationale in sources/.

    client-images: Checks generated client-side configuration for missing
    deputes pictures in static/img/deputes/. Downloads missing faces from
    the Assemblée Nationale website.

    filters: Downloads IP and ASN filters for server-side IP-based
    filtering. Generated lists are placed in config/filters/generated/.
    Also applies custom filters defined in config/filters/custom.

    all: Executes all of the above.

    --regenerate: by default, the files won't be regenerated if they
    already exist. This option forces redownload and regeneration of all
    data. No effect on client-images.
EOF
}

gen_all() {
    gen_client $2
    gen_client_images $2
    gen_filters $2
}

check_exists() {
    if [ ! -f $1 ]; then
        echo "$1 does not exist.\nAborting."
        exit 1
    fi
}

gen_client() {
    check_exists $PROJECT_ROOT/config/client/picks.json
    check_exists $PROJECT_ROOT/config/client/organes.json

    if [ "$1" == "--regenerate" ] || \
       [ ! -f $PROJECT_ROOT/data/acteurs.json ] || \
       [ ! -f $PROJECT_ROOT/data/scrutins.json ]; then
        echo "Generating client configuration."
        mkdir -p $PROJECT_ROOT/data/
        ./scripts/client/download-opendata.sh
        ./scripts/client/compile-results.py
    else
        echo "Client configuration already present; skipping generation."
    fi
}

gen_client_images() {
    check_exists $PROJECT_ROOT/data/acteurs.json

    echo "Checking for missing images."
    ./scripts/client/download-images.sh
}

gen_filters() {
    check_exists $PROJECT_ROOT/.env
    check_exists $PROJECT_ROOT/config/filters/custom/asn-blacklist.csv
    check_exists $PROJECT_ROOT/config/filters/custom/asn-list.csv
    check_exists $PROJECT_ROOT/config/filters/custom/ip-blacklist.csv
    check_exists $PROJECT_ROOT/config/filters/custom/ip-whitelist.csv

    if [ "$1" == "--regenerate" ] || \
       [ ! -f $PROJECT_ROOT/config/filters/generated/asn-blacklist.csv ] || \
       [ ! -f $PROJECT_ROOT/config/filters/generated/asn-list.csv ] || \
       [ ! -f $PROJECT_ROOT/config/filters/generated/ip-blacklist.csv ] || \
       [ ! -f $PROJECT_ROOT/config/filters/generated/ip-whitelist.csv ]; then
        echo "Generating IP filters."
        $PROJECT_ROOT/scripts/filters/generate.sh
    else
        echo "IP filters already present; skipping generation."
    fi
}

check_exists_summary() {
    if [ ! -f $PROJECT_ROOT/$1 ]; then
        echo -ne '\033[0;31m'$1 is missing'\033[0m\n'
    else
        echo -ne '\033[0;32m'$1 found'\033[0m\n'
    fi
}

summary() {
    echo -e "\nQuelParti assets status\n"

    echo "Server-side configuration"
    check_exists_summary config/server/config.toml
    check_exists_summary config/server/platforms.json

    echo -e "\nClient-side configuration"
    check_exists_summary config/client/organes.json
    check_exists_summary config/client/picks.json
    
    echo -e "\nClient-side outputs"
    check_exists_summary data/acteurs.json
    check_exists_summary data/scrutins.json

    echo -e "\nFilters configuration"
    check_exists_summary config/filters/custom/asn-blacklist.csv
    check_exists_summary config/filters/custom/asn-list.csv
    check_exists_summary config/filters/custom/ip-blacklist.csv
    check_exists_summary config/filters/custom/ip-whitelist.csv

    echo -e "\nFilters outputs"
    check_exists_summary config/filters/generated/asn-blacklist.csv
    check_exists_summary config/filters/generated/asn-list.csv
    check_exists_summary config/filters/generated/ip-blacklist.csv
    check_exists_summary config/filters/generated/ip-whitelist.csv

    echo -e "\nIf everything is green, your instance should be ready."
    echo "Once everything is set up, you can remove the sources folder."
}

# main

if [ "$2" != "--regenerate" ] && [ "$2" != "" ]; then
    echo "Unknown option: $2"
    exit 2
fi

case $1 in
    client)             gen_client $2;;
    client-images)      gen_client_images $2;;
    filters)            gen_filters $2;;
    all)                gen_all $2;;
    *)                  usage; exit 0;;
esac

summary
