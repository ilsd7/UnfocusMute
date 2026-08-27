<div align="center">
  <img src="../assets/app-icon.png" alt="Icône UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>Application légère qui s’exécute dans la zone de notification de Windows et coupe automatiquement le son des jeux et applications choisis lorsqu’ils ne sont plus au premier plan.</strong></p>

  <p>
    <a href="../README.md">English</a> · <a href="README_ko.md">한국어</a> · <a href="README_ja.md">日本語</a> · <a href="README_zh-CN.md">简体中文</a> · <a href="README_es.md">Español</a> · Français · <a href="README_pt.md">Português</a> · <a href="README_hi.md">हिन्दी</a> · <a href="README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=Build&logo=githubactions&logoColor=white" alt="Build status"></a>
    &nbsp;
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    &nbsp;
    <a href="../LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>Fonctionnement 100 % local &nbsp;·&nbsp; Aucun accès réseau &nbsp;·&nbsp; Aucune télémétrie &nbsp;·&nbsp; Aucune installation requise</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Télécharger</a>
    · <a href="#utilisation">Utilisation</a>
    · <a href="#sécurité-et-confidentialité">Confidentialité</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute est une petite application Windows légère qui s’exécute dans la zone de notification, coupe automatiquement le son des jeux et applications choisis lorsqu’ils passent en arrière-plan, puis le réactive lorsqu’ils reviennent au premier plan.

- Compilée en application native Rust, elle s’exécute sans environnement d’exécution séparé.
- L’exécutable fait environ 600 Ko.
- Elle ne se limite pas aux jeux : vous pouvez aussi ajouter des applications courantes comme des navigateurs, des messageries, des lanceurs et des lecteurs multimédias.
- Seul le son coupé par UnfocusMute est réactivé automatiquement ; les applications que vous avez vous-même mises en sourdine le restent.

---

<p align="center">
  <img src="../assets/screenshot_fr.png" width="600" alt="Fenêtre principale d’UnfocusMute">
</p>

---

## Cas d’utilisation

- Vous laissez tourner un jeu ou une application et passez souvent à une autre fenêtre avec Alt+Tab.
- Vous voulez qu’une application reste silencieuse lorsqu’elle ne propose pas d’option pour couper le son en arrière-plan.
- Vous voulez couper uniquement le son d’une application précise lorsqu’elle est en arrière-plan pendant que vous faites autre chose.

## Télécharger et lancer

Sous Windows 10/11, téléchargez l’archive ZIP puis extrayez-la pour lancer l’application.
La version minimale prise en charge est Windows 10 version 1703.

| Dernière archive publiée |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Somme de contrôle SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notes de version](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Une fois le ZIP extrait, déplacez le dossier `UnfocusMute-windows-x64` à l’emplacement de votre choix, puis lancez `UnfocusMute-v<version>.exe`. Aucune installation, aucun environnement d’exécution supplémentaire ni aucun outil de développement n’est nécessaire.

> **Remarque :** Pour des raisons de coût, UnfocusMute est distribué sans signature de code Windows. Un avertissement Windows SmartScreen ou « éditeur inconnu » peut donc s’afficher au premier lancement. Pour vérifier vous-même l’origine et l’intégrité du fichier, consultez [Transparence et vérification des fichiers publiés](#transparence-et-vérification-des-fichiers-publiés).

<br>

## Utilisation

1. Lancez UnfocusMute. Au premier démarrage, choisissez la langue et les options de lancement, puis cliquez sur `Commencer`.
2. Lancez le jeu ou l’application à ajouter et faites-lui produire un son pour qu’il apparaisse dans la liste.
3. Revenez dans UnfocusMute, sélectionnez l’application, puis cliquez sur `Ajouter`. L’ajout par nom de `.exe` prend en charge toutes les sessions audio de l’application et reste valable après son redémarrage.
4. Passez à une autre fenêtre avec `Alt`+`Tab`, puis revenez. Le son de l’application ajoutée est coupé lorsqu’elle passe en arrière-plan, puis réactivé lorsqu’elle revient au premier plan.

C’est tout. La surveillance commence dès l’ajout et, avec les réglages par défaut, UnfocusMute continue de fonctionner dans la zone de notification lorsque vous fermez sa fenêtre.

> **L’application n’apparaît pas dans la liste ?** Cliquez sur `Tous les processus` ou saisissez directement le nom exact du `.exe`. Utilisez `Vue PID` si vous souhaitez ajouter uniquement un PID précis en cours d’exécution. Comme le PID change à chaque redémarrage de l’application, il est généralement préférable de l’ajouter par le nom du `.exe`.

### Trouver le nom de l’exécutable

Si vous ne connaissez pas le nom de l’application à ajouter, recherchez le nom exact de son `.exe` dans le Gestionnaire des tâches.

1. Lancez d’abord l’application à ajouter.
2. Si elle s’exécute en plein écran, passez de l’écran du jeu à une autre fenêtre avec `Alt`+`Tab` ou `Windows`+`Tab`.
3. Appuyez sur `Ctrl`+`Shift`+`Esc` pour ouvrir le Gestionnaire des tâches.
4. Dans la liste des processus affichée à l’ouverture, cliquez sur la colonne `Processeur` pour trier les processus du plus actif au moins actif.
5. Repérez vers le haut de la liste l’application que vous venez de lancer, faites un clic droit dessus, puis sélectionnez `Propriétés`.
6. Relevez le nom de l’exécutable se terminant par `.exe`, par exemple `game.exe`, puis ajoutez-le dans UnfocusMute.

### Actions courantes

- Faites un clic droit sur une application ajoutée pour la mettre en pause ou la reprendre, modifier sa note ou la retirer de la liste.
- Cliquez sur l’état `Surveillance en cours` en haut pour suspendre ou reprendre toute la surveillance.
- Dans `Paramètres`, vous pouvez modifier le démarrage automatique et le comportement à la fermeture de la fenêtre.
- Pour quitter complètement l’application, faites un clic droit sur l’icône de la zone de notification, puis sélectionnez `Quitter`.

<br>

## Ajouter une note lorsque le nom d’une application n’est pas clair

Faites un clic droit sur une application ajoutée, puis sélectionnez `Modifier la note` pour afficher une description facile à reconnaître au-dessus du nom du processus. La note sert uniquement à distinguer l’application et n’influence pas le choix de l’application à mettre en sourdine.

- `htgame.exe` → `NTE`
- `game.exe (PID 21976)` → `client du serveur de test`

Les notes sont enregistrées avec les autres réglages dans `%APPDATA%\UnfocusMute\config.json`.

<br>

## Comportements à connaître

**Fonctionnement dans la zone de notification et réactivation du son :** UnfocusMute réactive automatiquement le son d’une application ajoutée lorsqu’elle revient au premier plan ou lorsque vous quittez UnfocusMute. Si l’application se ferme en premier, UnfocusMute veille également à réactiver le son. Toutefois, si UnfocusMute se ferme de manière inattendue, le son peut rester coupé dans Windows. Si une application n’émet aucun son à son prochain lancement, vérifiez son état dans le `Mélangeur de volume` de Windows.

**Ajout par PID :** Windows peut attribuer des PID différents à la session audio et à la fenêtre au premier plan. Ainsi, même après un ajout par PID, UnfocusMute considère que l’application est revenue lorsqu’une fenêtre portant le même nom de `.exe` passe au premier plan et en réactive le son. Ce mode n’est donc pas adapté si vous souhaitez continuer à utiliser une fenêtre du même `.exe` tout en maintenant un PID précis en sourdine. Il peut en revanche être utile pour couper le son d’un seul PID parmi plusieurs appartenant au même `.exe`, tout en conservant le son des autres PID pendant que vous travaillez dans une autre application.

**Compatibilité anti-triche :** UnfocusMute n’injecte pas de code dans les jeux, ne lit pas leur mémoire, n’intercepte pas les entrées et ne modifie pas leurs fichiers. Il utilise uniquement les informations Windows sur les processus et la fenêtre au premier plan, ainsi que les commandes de mise en sourdine de CoreAudio. Il est conçu pour éviter les conflits avec la plupart des systèmes anti-triche, mais la compatibilité avec tous ces systèmes ne peut pas être garantie.

<br>

## Dépannage

Commencez par vérifier les points suivants :

- **L’application n’apparaît pas dans la liste :** Faites-lui produire un son, puis rouvrez la liste. Si elle reste absente, cliquez sur `Tous les processus` ou [recherchez directement le nom de l’exécutable](#trouver-le-nom-de-lexécutable).
- **Le son n’est pas coupé :** Vérifiez que l’état en haut indique `Surveillance en cours` et que l’application ajoutée n’est pas `En pause`. Le fonctionnement peut aussi être perturbé si un pilote, un réglage d’autorisation ou un logiciel de sécurité limite l’accès aux sessions audio de Windows.
- **Le son reste coupé :** Si l’application ajoutée reste muette lorsque vous revenez à sa fenêtre, vérifiez son état dans le `Mélangeur de volume` de Windows.
- **L’ajout par PID ne fonctionne pas comme prévu :** Consultez le [comportement de l’ajout par PID](#comportements-à-connaître).
- **L’état passe à `Attention requise` :** Cliquez sur `Détails`, à côté de l’indicateur d’état, pour consulter l’erreur.

Si le problème persiste, [ouvrez une issue sur GitHub](https://github.com/ilsd7/UnfocusMute/issues/new/choose).

Si vous pensez qu’il s’agit d’une faille de sécurité, ne publiez pas les détails dans une issue publique. Utilisez la procédure de signalement privé et consultez [SECURITY.md](../SECURITY.md) pour plus d’informations.

<br>

## Fichier de configuration

Pour consulter ou sauvegarder directement le fichier de configuration, cliquez sur `Ouvrir le dossier des paramètres` dans Paramètres. L’Explorateur de fichiers ouvre le dossier `%APPDATA%\UnfocusMute`, où les paramètres sont enregistrés.

Vous pouvez également modifier directement `config.json`. Si son format est invalide et ne peut pas être lu, UnfocusMute sauvegarde le fichier d’origine sous `config.invalid-<timestamp>.json`, puis en crée un nouveau à partir des valeurs par défaut ou des réglages actuels de l’application.

<br>

## Sécurité et confidentialité

UnfocusMute s’exécute entièrement en local. L’application fonctionne normalement même sans connexion Internet et ne demande pas de droits administrateur. Elle n’effectue pas non plus de requêtes réseau automatiques, n’utilise pas de télémétrie, n’envoie pas de rapports de plantage, n’effectue aucune journalisation distante et ne collecte pas de données.

La seule exception est le bouton `Dépôt GitHub` dans les paramètres : il ouvre le dépôt GitHub du projet dans votre navigateur par défaut uniquement lorsque vous cliquez dessus.

La détection des sessions audio et le contrôle du son utilisent uniquement les API Windows CoreAudio. UnfocusMute n’injecte pas de code dans les processus ciblés, ne lit pas leur mémoire et n’intercepte pas les entrées.

### Informations stockées

UnfocusMute stocke uniquement les réglages nécessaires à son fonctionnement dans `%APPDATA%\UnfocusMute\config.json`.

- Noms de processus ajoutés
- PID ajoutés directement
- Dernier état de mise en sourdine de chaque application ajoutée
- Notes saisies
- Langue et paramètres choisis
- Position et taille de la fenêtre

Ces informations ne sont envoyées nulle part.

Si vous activez le démarrage automatique à l’ouverture de session Windows, le chemin de l’exécutable actuel est aussi enregistré dans la valeur `UnfocusMute` sous `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

### Informations non stockées

UnfocusMute ne stocke pas l’historique d’utilisation, les journaux d’activité, les journaux d’erreurs, les données audio, les titres de fenêtres, les frappes au clavier ni aucune autre information que celles indiquées ci-dessus dans « Informations stockées ».

### Supprimer complètement UnfocusMute

1. Si vous avez activé `Lancer à l’ouverture de session Windows`, désactivez d’abord cette option dans `Paramètres`.
2. Faites un clic droit sur l’icône de la zone de notification, puis sélectionnez `Quitter`.
3. Supprimez les dossiers `UnfocusMute-windows-x64` et `%APPDATA%\UnfocusMute`.

Si vous avez déjà supprimé l’exécutable et ne pouvez plus désactiver le démarrage automatique, supprimez la valeur `UnfocusMute` sous `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

<br>

## Transparence et vérification des fichiers publiés

UnfocusMute est conçu pour être utilisé en toute sécurité dans les environnements courants. La plupart des utilisateurs n’ont donc pas besoin de suivre les étapes de vérification ci-dessous. Si vous ne souhaitez pas dépendre uniquement de la confiance accordée au développeur ou si vous accordez une importance particulière à la sécurité de la chaîne d’approvisionnement logicielle, vous pouvez suivre cette procédure publique pour vérifier l’origine et l’intégrité des fichiers téléchargés.

### Pourquoi une vérification distincte est nécessaire

Même après avoir examiné le code source du dépôt et l’avoir jugé sûr, vous ne pouvez pas conclure que les fichiers d’une version publiée sur GitHub ont effectivement été produits à partir de ce code. Si le compte du développeur est compromis ou si les droits de publication sont détournés, des fichiers sans rapport avec le code source publié pourraient être distribués.

La comparaison des hachages SHA-256 permet de confirmer qu’un fichier téléchargé correspond à la somme de contrôle publiée, mais elle ne prouve pas à partir de quel code source ni dans quel environnement de compilation il a été produit.

Afin de traiter ces risques de chaîne d’approvisionnement de façon transparente, UnfocusMute publie une méthode permettant de vérifier directement qu’un fichier joint à une version publiée sur GitHub est un artefact officiel généré par GitHub Actions à partir du commit référencé par le tag de cette version.

<details>
<summary>Afficher la procédure de vérification</summary>

Installez d’abord [GitHub CLI](https://cli.github.com/). Remplacez ensuite la valeur de `$version` ci-dessous par le tag de la version à vérifier, puis exécutez l’ensemble du bloc de commandes dans PowerShell.

```powershell
$version = "v1.5.0"
$sourceRef = "refs/tags/$version"
$workflow = "ilsd7/UnfocusMute/.github/workflows/release.yml"

gh attestation verify .\UnfocusMute-windows-x64.zip `
  -R ilsd7/UnfocusMute `
  --source-ref $sourceRef `
  --signer-workflow $workflow
```

Cette commande contacte le service d’attestation de GitHub et vérifie que le SHA-256 du ZIP local correspond à la valeur enregistrée dans la provenance de compilation signée par GitHub Actions.

Vous pouvez également comparer le ZIP au hachage SHA-256 publié avec la version.

```powershell
$expectedHash = ((Get-Content .\UnfocusMute-windows-x64.zip.sha256 -TotalCount 1) -split '\s+')[0]
$actualHash = (Get-FileHash .\UnfocusMute-windows-x64.zip -Algorithm SHA256).Hash

if ($actualHash -ne $expectedHash) {
  throw "La vérification SHA-256 a échoué."
}

"SHA-256 vérifié : $actualHash"
```

Une vérification réussie confirme que le ZIP téléchargé a été généré par le workflow GitHub Actions indiqué pour le tag de version spécifié et qu’il correspond au hachage enregistré dans son attestation.

Elle ne prouve pas que le code source lui-même est sûr, que l’ensemble de l’environnement GitHub est intact, ni que la compilation est reproductible octet par octet sur un autre ordinateur.

</details>

<br>

## Compiler depuis le code source

La cible recommandée pour les versions publiées est `x86_64-pc-windows-msvc`.

Outils nécessaires :

- Rust stable
- Visual Studio Build Tools 2022 ou Visual Studio 2022
- SDK Windows 10/11

`cargo-about` est également nécessaire pour mettre à jour `THIRD_PARTY_NOTICES.md`.

<details>
<summary>Afficher les commandes de compilation et de création du paquet</summary>

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

Les compilations destinées aux versions publiées sont configurées pour réduire la taille du binaire. Le profil `release` de `Cargo.toml` retire les symboles, active LTO, utilise une seule unité de génération de code (`codegen-units = 1`), définit `panic = "abort"` et optimise pour la taille.

Exécutable :

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

ZIP de distribution :

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Lorsque vous devez actualiser les mentions de licence des dépendances tierces :

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

Le résultat est créé dans `dist\UnfocusMute-windows-x64.zip`, avec le fichier de vérification SHA-256 `dist\UnfocusMute-windows-x64.zip.sha256` au même emplacement. Le ZIP contient l’exécutable avec numéro de version (`UnfocusMute-v<version>.exe`), `LICENSE` et `THIRD_PARTY_NOTICES.md`.

</details>

<br>

## Licence

Apache License 2.0. Consultez [LICENSE](../LICENSE) pour les détails.

Consultez [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) pour les mentions de licence des crates Rust tierces.
