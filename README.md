# QuelParti.fr

![QuelParti.fr](/static/assets/img/banner/banner.png)

Choisissez votre coalition.

[Faites le test.](https://quelparti.fr)

Consultez la [méthodologie](METHODOLOGY.md).

*Les procédures d'installation et de configuration sont en cours d'amélioration, voir le ticket #1. Pour le moment, référez-vous au code et n'hésitez pas à documenter derrière vous.*

### Installation

Ce projet compile en [Rust](https://rustup.rs/) stable avec Cargo.

Sur une distribution basée sur Debian, les paquets wget, curl, unzip, jq, python3 et locales sont nécessaires pour le bon fonctionnement des scripts. Le paquet libpq5 est également nécessaire pour assurer la connexion à la base de données.

### Configuration

Les fichiers suivants doivent être présents :
- `config/server/config.toml` : Configuration du serveur, contenant les secrets et mots de passe.
- `config/server/platforms.json` : Configuration du serveur, contenant la liste des hôtes enregistrés dans le logiciel autorisés à recevoir des participations, avec toutes leurs caractéristiques (le logiciel a été pensé pour être multihôte).
- `config/client/organes.json` : Liste manuellement définie des groupes politiques et de leurs caractéristiques.
- `config/client/picks.json` : Liste manuellement définie des questions et arguments.
- `config/filters/custom/*` : Listes personnalisées pour le filtrage IP (facultatif).

À partir de ces fichiers, le script `setup.sh` peut générer le reste de la configuration.

### Déploiement

Il est fortement recommandé de déployer QuelParti derrière un reverse-proxy comme Nginx afin de contrôler plus facilement le routage des requêtes (notamment bloquer la route `/internal` et servir certains fichiers dans `static/assets/txt/`) et de servir directement le dossier `static/` via le reverse-proxy.

Une instance de [PostgreSQL](https://www.postgresql.org/) est nécessaire pour faire fonctionner QuelParti. Le support d'autres SGBD n'est pas envisagé mais peut facilement être implémenté à l'aide de l'ORM utilisé, [Diesel](https://diesel.rs/).

Une instance de [Fail2ban](https://fail2ban.org/wiki/index.php/Main_Page) peut également être mise en place afin de limiter efficacement les tentatives de déni de service.

### Contribuer

Consultez le [guide de contribution](CONTRIBUTING.md).

### Sécurité

La politique de sécurité de ce logiciel est consultable sur la page [SECURITY.md](SECURITY.md).

### Licence

[Licence AGPLv3](LICENSE).

Voir les [crédits](CREDITS.md).
