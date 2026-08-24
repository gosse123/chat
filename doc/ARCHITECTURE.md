# Architecture — TchatLAN

## 1. Objectif de ce document

Décrire en détail la conception technique du projet : les composants, leurs responsabilités, la manière dont ils communiquent entre eux, et les choix techniques justifiés. Ce document sert de référence pendant tout le développement.

## 2. Principes de conception

1. **Séparation logique métier / interface** : comme dans Toolé, la logique réseau et applicative (`core`) ne doit jamais dépendre directement de l'interface utilisateur. La communication se fait via un trait (par exemple `trait ChatUI`), ce qui permet de tester `core` sans lancer d'interface graphique, et de changer d'interface plus tard sans toucher au cœur du système.
2. **Concurrence structurée** : chaque tâche de fond (écoute multicast, écoute des connexions QUIC entrantes, gestion d'un pair connecté) tourne dans sa propre tâche Tokio (`tokio::spawn`), avec un `JoinHandle` conservé dans une structure centrale pour permettre l'arrêt propre et le suivi de l'état.
3. **Chiffrement systématique** : aucune donnée ne transite en clair sur le réseau. Toutes les connexions de pair à pair utilisent QUIC avec TLS 1.3, comme pour Toolé.
4. **Résilience** : la déconnexion ou l'absence d'un pair ne doit jamais faire planter l'application ; chaque erreur réseau est gérée localement et remontée sous forme d'événement à l'interface.
5. **Simplicité du protocole** : les messages échangés doivent rester simples à sérialiser/désérialiser et faciles à faire évoluer (ajout de nouveaux types de messages sans casser la compatibilité).

## 3. Composants principaux

### 3.1 Module de découverte (Discovery)

Responsable de faire connaître la présence de l'application sur le réseau local et de détecter les autres instances.

- Envoie périodiquement un message d'annonce ("Hello") sur une adresse multicast dédiée.
- Écoute en permanence cette même adresse pour détecter les annonces des autres pairs.
- Maintient une liste des pairs connus, avec un horodatage de dernière annonce reçue.
- Retire automatiquement un pair de la liste si aucune annonce n'est reçue pendant une durée définie (timeout).

### 3.2 Module de transport (Transport)

Responsable de l'établissement et du maintien des connexions directes entre pairs.

- Ouvre un point d'écoute QUIC local pour accepter les connexions entrantes.
- Initie une connexion QUIC sortante vers un pair détecté par le module de découverte.
- Gère l'envoi et la réception des flux (streams) QUIC associés à chaque conversation.
- Répercute les événements de connexion/déconnexion vers le reste de l'application.

### 3.3 Module de protocole applicatif (Protocol)

Définit le format des messages échangés une fois la connexion établie (voir `PROTOCOLE.md` pour le détail complet). Ce module fait la sérialisation/désérialisation et la validation des messages reçus.

### 3.4 Module de persistance (Storage)

Enregistre localement l'historique des conversations dans une base SQLite, via `sqlx`. Chaque message reçu ou envoyé est stocké avec son expéditeur, son destinataire, son contenu et son horodatage.

### 3.5 Module d'interface (TUI)

Interface terminal construite avec `ratatui`. Affiche la liste des pairs connectés, la conversation active, et une zone de saisie. Ce module ne fait aucune logique réseau : il consomme les événements exposés par `core` via le trait d'abstraction et transmet les actions de l'utilisateur (envoyer un message, sélectionner un pair) au reste du système.

### 3.6 Module de gestion des identités

Attribue à chaque instance un identifiant unique (UUID généré au premier lancement, stocké localement) et un nom d'affichage choisi par l'utilisateur. Cet identifiant est inclus dans les messages d'annonce et permet de distinguer les pairs même en cas de changement d'adresse IP.

## 4. Déroulement du fonctionnement, étape par étape

1. **Démarrage** : l'application génère ou charge son identifiant unique, initialise la base de données locale, puis démarre en parallèle le module de découverte et le point d'écoute QUIC.
2. **Découverte** : le module de découverte commence à émettre des annonces multicast et à écouter celles des autres instances. Dès qu'un nouveau pair est détecté, il est ajouté à la liste et affiché dans l'interface.
3. **Connexion** : lorsque l'utilisateur sélectionne un pair pour lui écrire, l'application initie une connexion QUIC vers l'adresse IP annoncée par ce pair. Si une connexion existe déjà, elle est réutilisée.
4. **Échange de messages** : les messages saisis dans l'interface sont sérialisés selon le protocole applicatif, envoyés sur le flux QUIC correspondant, puis stockés localement. Les messages reçus suivent le chemin inverse : réception, désérialisation, stockage, puis affichage.
5. **Déconnexion** : si un pair ne répond plus (timeout de découverte ou fermeture de connexion QUIC), il est marqué comme hors ligne dans l'interface, sans que cela affecte les autres connexions actives.
6. **Arrêt de l'application** : toutes les tâches de fond sont arrêtées proprement via leurs `JoinHandle`, les connexions QUIC sont fermées, et la base de données est synchronisée sur disque.

## 5. Modèle de concurrence

L'application repose sur le runtime asynchrone Tokio. Trois catégories de tâches cohabitent :

- Une tâche unique pour la boucle de découverte multicast (émission + écoute).
- Une tâche unique pour l'écoute des connexions QUIC entrantes.
- Une tâche par pair connecté, dédiée à la lecture des messages entrants sur ce pair.

Toutes ces tâches sont référencées dans une structure centrale (`HashMap<PeerId, JoinHandle<()>>`, à l'image de ce qui a été fait pour la gestion des transferts dans Toolé), ce qui permet d'annuler proprement une tâche liée à un pair qui se déconnecte, sans affecter les autres.

## 6. Sécurité

- Toutes les connexions de transport utilisent QUIC avec TLS 1.3 : les données sont chiffrées de bout en bout entre deux pairs directement connectés.
- Comme il n'y a pas d'autorité de certification centrale sur un réseau local, chaque instance génère un certificat auto-signé au premier démarrage.
- Pour éviter les attaques de type usurpation d'identité, l'empreinte du certificat de chaque pair peut être associée à son identifiant unique lors de la première connexion (confiance à la première utilisation, sur le modèle de SSH).
- Aucune donnée n'est envoyée à un serveur tiers : toutes les communications restent locales au réseau.

## 7. Gestion des erreurs et reconnexion

- Une perte de connexion QUIC avec un pair déclenche une tentative de reconnexion automatique tant que ce pair continue d'apparaître dans les annonces de découverte.
- Les erreurs de sérialisation ou de protocole sur un message reçu entraînent le rejet de ce seul message (avec journalisation), sans fermer la connexion.
- Les erreurs réseau (socket indisponible, adresse déjà utilisée) sont remontées à l'interface sous forme de message d'erreur explicite, jamais sous forme de plantage silencieux.

## 8. Choix techniques justifiés

- **QUIC plutôt que TCP brut ou WebSocket** : QUIC apporte le chiffrement natif (TLS 1.3 intégré), le multiplexage de flux sans tête-de-ligne bloquante, et une reprise de connexion plus rapide — les mêmes avantages qui ont motivé son choix pour Toolé.
- **Multicast plutôt que broadcast UDP** : le multicast limite le trafic aux machines réellement intéressées (celles qui rejoignent le groupe multicast), ce qui est plus propre qu'un broadcast qui atteint toutes les machines du sous-réseau, y compris celles qui n'exécutent pas l'application.
- **SQLite plutôt que PostgreSQL** : le projet étant décentralisé (chaque instance tourne sur sa propre machine sans serveur central), une base embarquée locale comme SQLite est suffisante et évite la complexité de déployer et maintenir un serveur PostgreSQL sur chaque poste.
- **ratatui plutôt qu'une interface graphique** : cohérent avec l'objectif d'une application 100% terminal, légère, scriptable, et sans dépendance à un environnement graphique.

## 9. Limites connues et points d'attention

- Le multicast est bloqué par certains réseaux Wi-Fi d'entreprise ou certains routeurs mal configurés ; prévoir un mode de secours (saisie manuelle d'une adresse IP) pour ces cas.
- Sans autorité de certification centrale, la vérification d'identité repose sur la confiance à la première connexion ; ce n'est pas une garantie absolue contre un attaquant déjà présent sur le réseau au moment de la première rencontre.
- Le passage à l'échelle (beaucoup de pairs simultanés) n'est pas un objectif du projet ; l'architecture cible un usage entre quelques machines sur un réseau domestique ou de petit bureau.
