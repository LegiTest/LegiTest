#!/bin/bash

ERRMSG="Les paquets wget ou unzip ne sont pas installés. Veuillez l'installer ou télécharger manuellement les données nécesaires en consultant ce script."

which wget 2>/dev/null || echo $ERRMSG

which unzip 2>/dev/null || echo $ERRMSG

EXTRACT_FOLDER="sources/client"

mkdir -p ./$EXTRACT_FOLDER

rm -f ./$EXTRACT_FOLDER/scrutins-tmp.zip ./$EXTRACT_FOLDER/acteurs-tmp.zip

# Téléchargement des scrutins
wget -O ./$EXTRACT_FOLDER/scrutins-tmp.zip http://data.assemblee-nationale.fr/static/openData/repository/15/loi/scrutins/Scrutins_XV.json.zip

# Téléchargement des acteurs (députés + organes)
wget -O ./$EXTRACT_FOLDER/acteurs-tmp.zip http://data.assemblee-nationale.fr/static/openData/repository/15/amo/tous_acteurs_mandats_organes_xi_legislature/AMO30_tous_acteurs_tous_mandats_tous_organes_historique.json.zip

# Recréation des dossiers de destination

rm -rf ./$EXTRACT_FOLDER/scrutins ./$EXTRACT_FOLDER/acteurs

mkdir -p ./$EXTRACT_FOLDER/scrutins ./$EXTRACT_FOLDER/acteurs

# Décompression des scrutins et des acteurs

unzip -jd ./$EXTRACT_FOLDER/scrutins ./$EXTRACT_FOLDER/scrutins-tmp.zip "json/*"

unzip -jd ./$EXTRACT_FOLDER/acteurs ./$EXTRACT_FOLDER/acteurs-tmp.zip "json/acteur/*"

