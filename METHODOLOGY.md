# Méthodologie

**Avertissement** : Cette plateforme n'a pas été réalisée par une [entreprise de sondages](https://fr.wikipedia.org/wiki/Entreprise_de_sondages), ni par des experts en sondologie. Par ailleurs, considérer QuelParti.fr comme un sondage constitue probablement un abus de langage ; on parlera donc plutôt d'un *test politique*. Quoi qu'il en soit, il s'agit d'un travail amateur.

## Réalisation du test

### Introduction

QuelParti.fr est une plateforme web dont le but est d'accompagner des personnes qui découvrent la politique vers la construction d'un positionnement politique, à travers une série de question qui permettent d'associer leurs opinions politiques à celles de groupes existants.  
Cette plateforme permet également à des personnes déjà sensibilisées aux enjeux politiques d'apporter un nouveau regard – et peut-être un peu de recul – sur leur orientation politique.  
Inspirée des sites de tests politiques tels que PolitiScales ou 8values, QuelParti.fr se différencie néanmoins sur le principe : les questions posées sont basées sur des faits réels (des votes et des débats qui ont déjà pris place) plutôt que sur des idées, et le résultat correspond à un groupe et des personnalités plutôt que des idéologies.

### Idée générale

À partir de cette finalité, quelques règles ont été établies dans afin d'établir les lignes directrices du projet.

1. Les questions posées dans le test portent uniquement sur des textes débattus et votés dans des institutions parlementaires.
    - Pour la plateforme QuelParti, l'Assemblée nationale a été choisie.
    - La formulation des questions peut varier, mais il ne peut pas s'agir d'une question inventée.
2. Les débats et les votes sur lesdits textes sont publics et accessibles sous forme de données ouvertes.
    - Cela permet une intégration facile des résultats des votes dans le logiciel.
3. Il est indiqué à la personne sondée des arguments "pour" ou "contre" afin de l'aider à se positionner.
    - Chacun de ces arguments sont de taille approximativement équivalente.
    - Ils sont compréhensibles et doivent rester courts (un petit paragraphe).
    - Dans l'idéal, ces arguments sont des citations des personnes qui ont voté la loi, tirées d'un compte-rendu ou d'un enregistrement vidéo de la séance.
    - Les sources sont toujours données.
4. Les résultats présentent la meilleure correspondance avec le groupe politique et la personne la plus proche de l'opinion de la personne sondée, en fonction de ses réponses.
    - Il y a également une vue détaillée permettant de consulter l'intégralité des résultats, avec un affichage du vote de chaque personne et de chaque groupe, sur toutes les questions.
    - Bonus: il peut y avoir un graphique représentant l'opinion des participants et participantes au test.

### Présélection des questions

Avant de sélectionner les sujets du test, il a tout d'abord été nécessaire de choisir une source de données : le choix se porte donc sur les différentes institutions parlementaires françaises. Pour simplifier le travail, il a été préférable de ne choisir qu'une seule organisation : intégrer les résultats des scrutins d'institutions différentes aurait nécessité beaucoup plus de code et de temps de travail.

#### Choix de l'Assemblée nationale

L'Assemblée nationale a finalement été choisie pour sa représentation importante auprès du grand public et son rôle dans le système électoral : l'intérêt du grand public a peut-être plus de chances d'être sollicité si le test porte sur des personnes qu'il élit directement, la chambre haute étant élue par des "grands électeurs".

#### Nombre de questions

Tandis que certains tests politiques dépassent les 50 questions, le nombre de questions dans QuelParti.fr a été limité à 15. Rechercher une question et rédiger des arguments demande beaucoup de temps à consacrer à la veille politique et à la lecture des compte-rendus de la chambre basse. De plus, la lecture des arguments prend un certain temps, augmenter le nombre de questions pourrait décourager les personnes qui répondent au test.

### Critères de sélection

Les critères suivants ont été définis pour décider des questions à inclure dans le test.

Critères obligatoires :
- La question a été traitée dans l'hémicycle de l'Assemblée nationale durant la XVe législature (2017-2022).
- Le vote des députés sur cette question prend la forme d'un **[scrutin public](https://www2.assemblee-nationale.fr/scrutins/liste/%28legislature%29/15)**, dont les résultats sont accessibles depuis les données ouvertes de l'Assemblée nationale.
- Il n'y a pas de consensus global au sein de l'Assemblée sur cette question : les réponses permettent de départager les députés et groupes politiques.
- La question et ses arguments pour / contre correspondent aux critères de publication choisis pour QuelParti :
    - La description, les arguments pour et contre comportent **environ 300 caractères** chacun.
    - Tous les termes sortant du langage commun ou pouvant sembler obscurs sont définis et présentés de manière accessible.
    - Il n'y a qu'un argument pour et un argument contre. Dans certaines circonstances exceptionnelles, il peut y avoir un argument supplémentaire "abstention".
- Toutes les sources officielles sont citées : le dossier législatif, le scrutin public, le compte rendu, l'amendement le cas échéant.
- Le taux de participation au scrutin s'élève à 15 % minimum.

Critères secondaires :
- Les résultats permettent de départager des groupes qui votent souvent ensemble :
    - LAREM-MODEM
    - LFI-SOC-GDR (LFI-SOC, SOC-GDR ou LFI-GDR)
- Le taux de participation au scrutin s'élève à 30 % minimum.
- Il s'agit d'une question particulièrement "polémique" : qui relève d'une problématique largement médiatisée ou relayée, qui a suscité d'importants mouvements sociaux (critère assez subjectif)
- Le thème de la question est peu soulevé par les autres questions, et il n'existe pas de question portant sur un sujet similaire.

Les critères secondaires ne sont pas tous respectés et représentent plutôt des "bonus" qui motiveraient la sélection d'une question. Il n'y a pas de seuil particulier de critères secondaires à respecter, et même une question respectant tous ces critères pourrait ne pas avoir été incluse, parce qu'il n'y a que 15 questions dans le test de QuelParti.fr et que faute de temps, il n'a pas été possible de considérer tous les scrutins publics de la XVe législature.

### Rédaction d'une question

Voici la méthodologie appliquée pour la rédaction d'une question :

- Les termes et le contexte général sont définis dans la description.
- Les arguments pour / contre sont directement tirés de l'un des compte-rendus de séance de l'Assemblée nationale :
    1. Tout d'abord sous la forme de plusieurs paragraphes, des citations entières de députés. Les explications de vote sont très utiles à cette fin ;
    2. Les formulations agressives, condescendantes, les arguments *ad hominem*, *ad passiones*, ou les propos extrêmes (lorsqu'ils n'illustrent pas directement l'argument en lui-même) sont retirés de l'argumentation, afin de valoriser l'argument de fond plutôt que l'émotion momentanée du député ;
    3. Les phrases sont raccourcies autant que possible, en préservant l'argument de fond, pour former un argument final d'une taille de 300 caractères environ.
- La description présente le contexte et la nature de la question, sans nécessairement détailler s'il s'agit d'un amendement, d'une proposition de loi ou encore d'une motion de rejet préalable.
- Une question finale conclut l'argumentaire, permettant de recentrer le débat sur ses trois issues possibles : pour, contre ou abstention.
- Il n'est pas précisé de quel groupe politique émane les argumentaires ou la question.

### Tri des groupes parlementaires

Cette section concerne le script `compile-results.py` qui lit les données ouvertes de l'Assemblée nationale, le fichier contenant les arguments `picks.json`, et génère les deux fichiers de données `acteurs.json` et `scrutins.json`.

Les groupes politiques de l'Assemblée nationale sont en perpétuelle évolution : certains groupes disparaissent, d'autres se forment, certains se réforment... et ce, même en cours de législature.
Pour que les résultats soient représentatifs avec la constitution actuelle des groupes politiques, les anciens groupes ont été fusionnés avec les groupes actuels.

Voici la méthodologie suivie :

- Seuls les groupes qui existent encore en 2021 ont été ajoutés à la liste publique.
- Les groupes ayant été dissous ou renommés entre-temps ont été rattachés aux groupes existants.
    - Ainsi, sur un scrutin public enregistré avant la dissolution du groupe MODEM, les membres de MODEM sont considérés comme membres du groupe DEM.
    - Le groupe de rattachement des groupes dissous est déterminé par le groupe d'origine de la majorité des parlementaires du groupe.
    - Liste des groupes rattachés :
        - [Mouvement démocrate et apparentés](https://fr.wikipedia.org/wiki/Groupe_Mouvement_d%C3%A9mocrate_et_d%C3%A9mocrates_apparent%C3%A9s) (MODEM) -> Mouvement Démocrate et Démocrates apparentés (DEM)
        - [Écologie Démocratie Solidarité](https://fr.wikipedia.org/wiki/Groupe_%C3%89cologie_d%C3%A9mocratie_solidarit%C3%A9) (EDS) -> La République en Marche (LREM)
        - [UDI, Agir et Indépendants](https://fr.wikipedia.org/wiki/Groupe_UDI_et_indépendants) (UDI-A-I) -> UDI et Indépendants (UDI-I)
        - [Les Constructifs : républicains, UDI, indépendants](https://fr.wikipedia.org/wiki/Groupe_UDI_et_indépendants) (LC) -> UDI et Indépendants (UDI-I)
        - [Nouvelle Gauche](https://fr.wikipedia.org/wiki/Groupe_socialiste_(Assembl%C3%A9e_nationale)) (NG) -> Socialistes et apparentés (SOC)
- Les députés non-inscrits n'ont pas été associés à un groupe parlementaire, seules leurs voix individuelles ont été comptées.

