#!/bin/bash

# jq and wget are necessary for this script to work

LEGISLATURE='15'
DEPUTES_LIST=`cat data/acteurs.json | jq -r .deputes[].id`

# Download deputes images if missing

mkdir -p static/assets/img/deputes

for depute in $DEPUTES_LIST; do
    DEPUTE_ID=`echo $depute | grep -oE '[0-9]*'`
    if [[ ! -f "static/assets/img/deputes/$DEPUTE_ID.jpg" ]]; then
        echo "Picture for depute $DEPUTE_ID does not exist, downloading."
        wget -O "static/assets/img/deputes/$DEPUTE_ID.jpg" https://www2.assemblee-nationale.fr/static/tribun/$LEGISLATURE/photos/$DEPUTE_ID.jpg
    fi
done
