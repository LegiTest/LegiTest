#!/bin/bash

ERRORMSG="The tools wget, curl, tar and unzip are required for this script to work. Please install them through your package manager."

SCRIPTS_FOLDER="scripts/filters"
EXTRACT_FOLDER="sources/filters"
CUSTOM_FOLDER="config/filters/custom"
DATA_FOLDER="config/filters/generated"

mkdir -p $DATA_FOLDER $CUSTOM_FOLDER

# Checking dependencies
which wget 2>/dev/null || echo $ERRMSG
which curl 2>/dev/null || echo $ERRMSG
which unzip 2>/dev/null || echo $ERRMSG
which tar 2>/dev/null || echo $ERRMSG

rm -rf $EXTRACT_FOLDER
mkdir -p $EXTRACT_FOLDER

# Downloading IP ranges list
wget -O $EXTRACT_FOLDER/IP2LOCATION-LITE-DB1.CSV.ZIP https://download.ip2location.com/lite/IP2LOCATION-LITE-DB1.CSV.ZIP
unzip -jd $EXTRACT_FOLDER/ $EXTRACT_FOLDER/IP2LOCATION-LITE-DB1.CSV.ZIP "IP2LOCATION-LITE-DB1.CSV"

# Removing non-French ranges, marked as 1000
grep -iE '"(FR|NC|PF|BL|MF|TF|WF|GP|GF|RE|YT|PM)"' $EXTRACT_FOLDER/IP2LOCATION-LITE-DB1.CSV | cut -d',' -f1,2 | tr -d \" > $DATA_FOLDER/ip-whitelist.csv

# Downloading ASN list
export $(grep -v '^#' .env | xargs) && wget -O $EXTRACT_FOLDER/IP2LOCATION-LITE-ASN.CSV.ZIP "https://www.ip2location.com/download/?token=${IP2LOCATION_TOKEN}&file=DBASNLITE"
unzip -jd $EXTRACT_FOLDER/ $EXTRACT_FOLDER/IP2LOCATION-LITE-ASN.CSV.ZIP IP2LOCATION-LITE-ASN.CSV
cat $EXTRACT_FOLDER/IP2LOCATION-LITE-ASN.CSV | cut -d',' -f1,2,4 | sed 's/"-"/"0"/g' | tr -d \" > $DATA_FOLDER/asn-list.csv

echo "Downloading Tor exit nodes list"
# Downloading Tor exit nodes, marked as 1001 
curl -s https://check.torproject.org/exit-addresses | grep "ExitAddress" | cut -d' ' -f2 > $EXTRACT_FOLDER/tor-exit.txt
$SCRIPTS_FOLDER/ip2dec.sh $EXTRACT_FOLDER/tor-exit.txt | sed 's/$/,1001/g' > $DATA_FOLDER/ip-blacklist.csv

echo "Downloading Spamhaus DROP list"
# Downloading Spamhaus DROP list, marked as 1002
curl -s https://www.spamhaus.org/drop/drop.txt | tail -n+5 | cut -d' ' -f1 > $EXTRACT_FOLDER/drop.txt
$SCRIPTS_FOLDER/ip2dec.sh $EXTRACT_FOLDER/drop.txt | sed 's/$/,1002/g' >> $DATA_FOLDER/ip-blacklist.csv

echo "Downloading Spamhaus EDROP list"
# Downloading Spamhaus EDROP list, marked as 1003
curl -s https://www.spamhaus.org/drop/edrop.txt | tail -n+5 | cut -d' ' -f1 > $EXTRACT_FOLDER/edrop.txt
$SCRIPTS_FOLDER/ip2dec.sh $EXTRACT_FOLDER/edrop.txt | sed 's/$/,1003/g' >> $DATA_FOLDER/ip-blacklist.csv

echo "Downloading Spamhaus ASN list"
# Downloading Spamhaus ASN blocklist, marked as 1100
curl -s https://www.spamhaus.org/drop/asndrop.txt | tail -n+5 | cut -d' ' -f1 | sed 's/^AS//g' | sed 's/$/,1100/g' > $DATA_FOLDER/asn-blacklist.csv

echo "Appending custom lists"
# Appending custom lists
cat $CUSTOM_FOLDER/asn-blacklist.csv >> $DATA_FOLDER/asn-blacklist.csv
cat $CUSTOM_FOLDER/asn-list.csv >> $DATA_FOLDER/asn-list.csv
cat $CUSTOM_FOLDER/ip-whitelist.csv >> $DATA_FOLDER/ip-whitelist.csv
cat $CUSTOM_FOLDER/ip-blacklist.csv >> $DATA_FOLDER/ip-blacklist.csv

echo "Data gathering completed. Please check the log above for errors."
