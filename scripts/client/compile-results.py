#!/usr/bin/env python3

import json
import sys
import os
import datetime
import locale

locale.setlocale(locale.LC_TIME, "fr_FR.UTF-8")

# Dossier contenant les informations sur chaque parlementaire en JSON
ACTEURS_DIR = './sources/client/acteurs/'

# Dossier contenant les scrutins publics au format JSON
SCRUTINS_DIR = './sources/client/scrutins/'

# Input : Fichier contenant les scrutins choisis pour le test au format JSON
IN_PICKS_FILE = './config/client/picks.json'

# Input : Fichier contenant les informations sur les organes au format JSON
IN_ORGANES_FILE = './config/client/organes.json'

# Output : Fichier contenant les organes et les parlementaires
OUT_ACTEURS_FILE  = './data/acteurs.json'

# Output : Fichier contenant les scrutins et les votes des parlementaires
OUT_SCRUTINS_FILE  = './data/scrutins.json'

# Output : Informations sur les organes et les parlementaires
out_acteurs = {}

# Output : Informations sur les scrutins et les votes des parlementaires
out_scrutins = []

### Init ###

print("[*] Lecture des scrutins sélectionnés...")

# Input : Scrutins choisis pour le test
with open(IN_PICKS_FILE, 'r') as f:
    in_picks = json.load(f)

print("[*] Lecture des informations sur les organes...")

# Input : Informations sur les organes
with open(IN_ORGANES_FILE, 'r') as f:
    in_organes = json.load(f)

### Fonctions ###

# Trouve un organe dans organes.json à partir de son
# identifiant AN (POXXXXX).
# S'il ne le trouve pas, affiche un warning et renvoie 0.
def find_organe(org_id, alert):
    for orginfo in in_organes:
        if org_id in orginfo["aliases"]:
            return orginfo["id"]
    if alert:
        print("[WARN] Identifiant {} introuvable dans organes.json".format(org_id))
    return 0

# Compte les votes (pours/contres/abstentions) dans une liste donnée
# Retourne une liste d'acteurs (PAXXXXXX) sous forme de tableau
def count_votes(votes_list):
    new_votes_list = []

    if votes_list is None:
        return new_votes_list
    # Gestion du cas [None, None] dans les mises au point
    elif type(votes_list) is list and votes_list[0] is None:
        return new_votes_list
    elif type(votes_list["votant"]) is list:
        for vote in votes_list["votant"]:
            new_votes_list.append(vote["acteurRef"])
    else:
        new_votes_list.append(votes_list["votant"]["acteurRef"])

    return new_votes_list

def count_miseaupoint(deputes_votes, od_miseaupoint):
    fixes_pour = count_votes(scrutin_od["scrutin"]["miseAuPoint"]["pours"])
    fixes_contre = count_votes(scrutin_od["scrutin"]["miseAuPoint"]["contres"])
    fixes_abstention = count_votes(scrutin_od["scrutin"]["miseAuPoint"]["abstentions"])

    deputes_votes = remove_wrong_vote(deputes_votes, fixes_pour)
    deputes_votes = remove_wrong_vote(deputes_votes, fixes_contre)
    deputes_votes = remove_wrong_vote(deputes_votes, fixes_abstention)

    deputes_votes["pour"] = add_fixed_vote(deputes_votes["pour"], fixes_pour)
    deputes_votes["contre"] = add_fixed_vote(deputes_votes["contre"], fixes_contre)
    deputes_votes["abstention"] = add_fixed_vote(deputes_votes["abstention"], fixes_abstention)
    
    return deputes_votes

def add_fixed_vote(deputes_votes, fixes_list):
    for fix in fixes_list:
        deputes_votes.append(fix)
    return deputes_votes

def remove_wrong_vote(deputes_votes, fixes_list):
    for fix in fixes_list:
        if fix in deputes_votes["pour"]:
            deputes_votes["pour"].remove(fix)
        if fix in deputes_votes["contre"]:
            deputes_votes["contre"].remove(fix)
        if fix in deputes_votes["abstention"]:
            deputes_votes["abstention"].remove(fix)
    return deputes_votes

# Vérifie que les groupes politiques ayant des alias ne soient pas comptés en double
def check_aliases(votes, org_id, scrutin_odv):
    #org_aliases = []
    #for orginfo in in_organes:
    #    if orginfo["id"] == org_id:
    #        org_aliases = orginfo["aliases"]
    #        break

    # En cas de conflit d'alias:
    # On ne retient que le groupe avec le plus grand nombre de parlementaires
    for voteinfo in scrutin_odv:
        if find_organe(voteinfo["organeRef"], True) == org_id:
            if voteinfo["nombreMembresGroupe"] > votes["nombreMembresGroupe"]:
                # Renvoie False pour supprimer le vote de l'alias
                return False
    return True

### Main ###

print("[*] Lecture des {} scrutins...".format(len(in_picks)))

# Itère sur la liste des scrutins, combine les descriptifs
# de picks.json avec les données ouvertes de l'AN
for scrutin in in_picks:
    print("[*] --> Lecture du scrutin {}".format(scrutin["id"]))

    new_scrutin = {}

    with open(os.path.join(SCRUTINS_DIR, scrutin["id"]+".json"), 'r') as f:
        scrutin_od = json.load(f)
   
    # Import des descriptions du scrutin dans picks.json
    new_scrutin = scrutin

    # Conversion du format "2019-01-04" en "4 janvier 2019"
    new_scrutin["dateScrutin"] = datetime.datetime.strptime(scrutin_od["scrutin"]["dateScrutin"], '%Y-%m-%d').strftime('%-d %B %Y')

    new_scrutin["nbreVotants"] = int(scrutin_od["scrutin"]["syntheseVote"]["nombreVotants"])

    new_organes = {}

    new_organes["pour"] = []
    new_organes["contre"] = []
    new_organes["abstention"] = []
    
    new_deputes = {}
    
    new_deputes["pour"] = []
    new_deputes["contre"] = []
    new_deputes["abstention"] = []

    # Lecture des votes
    scrutin_odv = scrutin_od["scrutin"]["ventilationVotes"]["organe"]["groupes"]["groupe"]
    for votes in scrutin_odv:

        # Définition et enregistrement du vote du groupe parlementaire
        org_id = find_organe(votes["organeRef"], True)

        if votes["vote"]["positionMajoritaire"] == "pour" and check_aliases(votes, org_id, scrutin_odv):
            new_organes["pour"].append(org_id)
        elif votes["vote"]["positionMajoritaire"] == "contre" and check_aliases(votes, org_id, scrutin_odv):
            new_organes["contre"].append(org_id)
        elif votes["vote"]["positionMajoritaire"] == "abstention" and check_aliases(votes, org_id, scrutin_odv):
            new_organes["abstention"].append(org_id)
        else:
            print("[INFO] Vote de l'alias {} annulé pour le scrutin {}".format(votes["organeRef"], scrutin["id"]))

        # Comptage et inscription des votes individuels
        for depvote in count_votes(votes["vote"]["decompteNominatif"]["pours"]):
            new_deputes["pour"].append(depvote)
        for depvote in count_votes(votes["vote"]["decompteNominatif"]["contres"]):
            new_deputes["contre"].append(depvote)
        for depvote in count_votes(votes["vote"]["decompteNominatif"]["abstentions"]):
            new_deputes["abstention"].append(depvote)

    new_scrutin["organes"] = new_organes
    new_scrutin["deputes"] = new_deputes

    # Lecture des mises au point
    if scrutin_od["scrutin"]["miseAuPoint"] is not None:
        new_scrutin["deputes"] = count_miseaupoint(new_scrutin["deputes"], scrutin_od["scrutin"]["miseAuPoint"])
    
    out_scrutins.append(new_scrutin)


# Liste tous les parlementaires en un seul tableau pour acteurs.json

all_deputes = []

for scr in out_scrutins:
    for dep in scr["deputes"]["pour"]:
        if dep not in all_deputes:
            all_deputes.append(dep)
    for dep in scr["deputes"]["contre"]:
        if dep not in all_deputes:
            all_deputes.append(dep)
    for dep in scr["deputes"]["abstention"]:
        if dep not in all_deputes:
            all_deputes.append(dep)

print("[*] {} parlementaires ont pris part aux votes.".format(len(all_deputes)))

print("[*] Lecture des informations sur les parlementaires...")

# Lecture des informations sur les parlementaires


out_acteurs["organes"] = in_organes

out_acteurs["deputes"] = []

for dep in all_deputes:
    new_dep = {}
    
    with open(os.path.join(ACTEURS_DIR, dep+".json"), 'r') as f:
        dep_od = json.load(f)

    new_dep["id"] = dep

    # Concaténation des prénom et nom
    new_dep["name"] = "{} {}".format(dep_od["acteur"]["etatCivil"]["ident"]["prenom"], dep_od["acteur"]["etatCivil"]["ident"]["nom"])
    
    new_dep["is_president"] = False
    new_dep["is_active"] = True

    for dep_mandat in dep_od["acteur"]["mandats"]["mandat"]:
        # Détermine si le parlementaire est président de groupe
        if dep_mandat["typeOrgane"] == "GP" and dep_mandat["dateFin"] is None and dep_mandat["infosQualite"]["codeQualite"] == "Président":
            new_dep["is_president"] = True

        # Détermine le groupe politique du parlementaire
        if dep_mandat["typeOrgane"] == "GP" and dep_mandat["dateFin"] is None:
            new_dep["organe"] = find_organe(dep_mandat["organes"]["organeRef"], True)

    # Si le parlementaire n'a pas de groupe politique, il n'est plus actif
    # Recherche de son groupe avec des critères moins stricts, puis
    # définition de son statut "is_active" à False
    if "organe" not in new_dep:
        new_dep["is_active"] = False
        for dep_mandat in dep_od["acteur"]["mandats"]["mandat"]:
            if dep_mandat["typeOrgane"] == "GP" and dep_mandat["dateFin"] is not None:
                new_dep["organe"] = find_organe(dep_mandat["organes"]["organeRef"], False)
                if new_dep["organe"] != 0:
                    break

    
    out_acteurs["deputes"].append(new_dep)



print("[*] Écriture des données collectées...")

with open(OUT_ACTEURS_FILE, 'w') as file:
    file.write(json.dumps(out_acteurs, sort_keys=False, indent=4))

with open(OUT_SCRUTINS_FILE, 'w') as file:
    file.write(json.dumps(out_scrutins, sort_keys=False, indent=4))

print("[*] Opération terminée.")

