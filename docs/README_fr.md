<div align="center">
  <img src="../assets/app-icon.png" alt="Icône UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>Application légère qui s’exécute dans la zone de notification de Windows et coupe automatiquement le son des jeux et applications choisis lorsqu’ils perdent le focus.</strong></p>

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

<br>

UnfocusMute est une petite application légère qui s’exécute dans la zone de notification de Windows, coupe automatiquement le son des jeux et applications choisis lorsqu’ils passent en arrière-plan, puis le rétablit lorsqu’ils reviennent au premier plan.

- Compilée en application native Rust, elle s’exécute sans environnement d’exécution séparé.
- L’exécutable fait environ 500 Ko.
- Elle ne se limite pas aux jeux : vous pouvez aussi ajouter des applications courantes comme des navigateurs, des messageries, des lanceurs et des lecteurs multimédias.
- La coupure et le rétablissement du son ne s’appliquent qu’aux sessions qu’UnfocusMute a modifiées lui-même ; les sessions dont vous aviez déjà coupé le son ne sont pas touchées.

<p align="center">
  <img src="../assets/screenshot_fr.png" width="600" alt="Fenêtre principale d’UnfocusMute">
</p>

<br>

## Cas d’utilisation

- Vous laissez tourner un jeu ou une application et passez souvent à une autre fenêtre avec Alt+Tab.
- Vous voulez qu’une application reste silencieuse lorsqu’elle ne propose pas d’option pour couper le son en arrière-plan.
- Vous voulez couper uniquement le son d’une application précise lorsqu’elle est en arrière-plan pendant que vous faites autre chose.

## Télécharger et lancer

Sous Windows 10/11, téléchargez l’archive ZIP puis extrayez-la pour lancer l’application.

| Dernière archive publiée |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Somme de contrôle SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notes de version](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Une fois le ZIP extrait, déplacez le dossier `UnfocusMute-windows-x64` à l’emplacement où vous souhaitez conserver l’application, puis lancez `UnfocusMute-v<version>.exe` depuis ce dossier.

UnfocusMute est une application autonome qui ne nécessite aucune installation. Vous n’avez pas non plus besoin d’installer Rust, Visual Studio Build Tools, MinGW ni d’autres outils de développement.

> **Remarque :** Comme les certificats de signature de code ont un coût, l’application est actuellement distribuée sans signature de code Windows. Un avertissement Windows SmartScreen ou « éditeur inconnu » peut s’afficher au premier lancement. Pour vérifier vous-même l’intégrité du fichier, consultez [Transparence et vérification des fichiers publiés](#transparence-et-vérification-des-fichiers-publiés).

<br>

## À savoir avant utilisation

UnfocusMute s’appuie sur les noms de processus, les informations sur la fenêtre au premier plan et les sessions CoreAudio fournies par Windows. Si un pilote, un réglage d’autorisation ou un logiciel de sécurité limite l’accès aux sessions, le contrôle du son peut ne pas fonctionner correctement.

Nous vous recommandons de laisser UnfocusMute actif dans la zone de notification plutôt que de le fermer. Si une application ajoutée se ferme alors que son son est coupé, ce dernier état peut être conservé. Tant qu’UnfocusMute reste actif, il rétablit automatiquement le son lorsque vous relancez l’application et la ramenez au premier plan. Si vous quittez également UnfocusMute, l’application risque de ne plus émettre de son ; dans ce cas, réactivez manuellement le son dans le `Mélangeur de volume` de Windows.

**Comportement de l’ajout par PID :** Windows ne fournit pas toujours le même PID pour une session audio et pour la fenêtre au premier plan. Pour compenser cela, UnfocusMute considère que l’application est revenue au premier plan lorsque le nom de l’exécutable (`.exe`) associé au PID ajouté correspond à celui de la fenêtre actuellement active.

Si plusieurs instances du même `.exe` sont ouvertes en même temps, il n’est donc pas toujours possible d’isoler parfaitement une instance précise. Dans ce cas, le son peut être rétabli lorsqu’une autre instance est au premier plan.

**Compatibilité anti-triche :** UnfocusMute n’injecte pas de code dans les jeux, ne lit pas la mémoire du jeu, n’intercepte pas les entrées utilisateur et ne modifie pas les fichiers du jeu. Il utilise seulement les informations Windows sur les processus et la fenêtre au premier plan, ainsi que les commandes de coupure du son des sessions CoreAudio. Il est donc conçu pour éviter les conflits avec la plupart des systèmes anti-triche, mais la compatibilité avec tous ces systèmes ne peut pas être garantie.

<br>

## Utilisation

1. Lancez UnfocusMute.
2. Choisissez la langue. Nous vous recommandons de conserver les réglages par défaut.
3. Lancez le jeu ou l’application à ajouter.
4. Sélectionnez une application dans la liste ou saisissez le nom exact de son fichier `.exe`, puis cliquez sur `Ajouter`. Si l’application n’a pas encore créé de session audio, passez à `Tous les processus` pour parcourir tous les processus en cours d’exécution.
5. Si vous devez ajouter seulement un PID précis, cliquez sur `Vue PID` et choisissez l’entrée concernée. Un ajout par PID n’est valable que pour l’instance actuellement ouverte ; si l’application redémarre avec un autre PID, ajoutez-la à nouveau.
6. Faites un clic droit sur une application ajoutée pour modifier sa note ou utiliser `Mettre en pause` uniquement pour cette application.
7. Cliquez sur l’état `Surveillance en cours` en haut pour suspendre ou reprendre toute la surveillance.
8. Ouvrez `Paramètres` pour modifier les options, notamment le comportement à la fermeture de la fenêtre.
9. Par défaut, fermer la fenêtre laisse UnfocusMute actif dans la zone de notification. Pour quitter complètement l’application, faites un clic droit sur son icône dans la zone de notification, puis choisissez `Quitter`. Vous pouvez modifier le comportement du bouton de fermeture dans `Paramètres`.

<br>

## Utiliser les notes des applications ajoutées

Si le nom du processus ne suffit pas à reconnaître l’application, faites un clic droit sur l’application ajoutée et choisissez `Modifier la note`. La note s’affiche au-dessus du nom du processus dans la liste, sans changer la manière dont l’application est identifiée.

C’est utile lorsqu’un même lanceur de jeu ouvre plusieurs processus, ou lorsqu’un nom d’exécutable n’indique pas clairement son rôle.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - client du serveur de test`

Les notes sont enregistrées localement avec les autres réglages dans `%APPDATA%\UnfocusMute\config.json`.

<br>

## Trouver le nom de l’exécutable

Si vous ne savez pas quel nom ajouter, ouvrez le Gestionnaire des tâches et cherchez le nom du fichier exécutable de l’application, celui qui se termine par `.exe`.

1. Lancez d’abord l’application à ajouter.
2. Utilisez `Alt`+`Tab` ou `Windows`+`Tab` pour revenir au bureau Windows.
3. Appuyez sur `Ctrl`+`Shift`+`Esc` pour ouvrir le Gestionnaire des tâches.
4. Triez la liste des processus par `CPU` afin de retrouver l’application que vous venez de lancer.
5. Faites un clic droit sur cet élément et ouvrez `Propriétés`.
6. Relevez le nom de l’exécutable se terminant par `.exe`, par exemple `game.exe`, puis ajoutez-le à UnfocusMute.

<br>

## Dépannage

Si une application n’apparaît pas dans la liste, ou si l’ajout par PID ne se comporte pas comme prévu, consultez d’abord [À savoir avant utilisation](#à-savoir-avant-utilisation) et [Trouver le nom de l’exécutable](#trouver-le-nom-de-lexécutable).

Si l’état en haut de la fenêtre passe à `Attention requise`, cliquez sur `Détails` pour consulter le message d’erreur détaillé.

Si le problème persiste, ouvrez une issue sur GitHub.

Si vous pensez qu’il s’agit d’une faille de sécurité, ne publiez pas les détails dans une issue publique. Utilisez la procédure de signalement privé et consultez [SECURITY.md](../SECURITY.md) pour plus d’informations.

<br>

## Fichier de configuration

Pour consulter ou sauvegarder directement le fichier de configuration, cliquez sur `Ouvrir le dossier des paramètres` dans Paramètres. L’Explorateur de fichiers ouvre le dossier `%APPDATA%\UnfocusMute`, où les paramètres sont enregistrés.

Vous pouvez modifier directement le fichier de configuration, mais si son format est invalide et qu’il ne peut pas être lu, il est sauvegardé sous `config.invalid-<timestamp>.json`. Si le problème est détecté au démarrage de l’application, les paramètres sont restaurés aux valeurs par défaut ; s’il est détecté pendant l’exécution, un nouveau fichier de configuration est créé à partir des paramètres actuels de l’application.

<br>

## Sécurité et confidentialité

UnfocusMute s’exécute entièrement en local. L’application fonctionne normalement même sans connexion Internet et ne demande pas de droits administrateur. Elle n’effectue pas non plus de requêtes réseau automatiques, n’utilise pas de télémétrie, n’envoie pas de rapports de plantage, n’effectue aucune journalisation distante et ne collecte pas de données.

La seule exception est le bouton `Dépôt GitHub` dans les paramètres : il ouvre le dépôt GitHub du projet dans votre navigateur par défaut uniquement lorsque vous cliquez dessus.

La détection des sessions audio et le contrôle du son utilisent uniquement les API Windows CoreAudio. UnfocusMute n’injecte pas de code dans les processus ciblés, ne lit pas leur mémoire et n’intercepte pas les entrées.

### Informations stockées

UnfocusMute stocke uniquement les réglages nécessaires à son fonctionnement dans `%APPDATA%\UnfocusMute\config.json`.

- Noms de processus ajoutés
- PID ajoutés directement
- Dernier état de coupure du son des applications ajoutées
- Notes saisies
- Langue et paramètres choisis
- Position et taille de la fenêtre

Ces informations ne sont envoyées nulle part.

Si vous activez le démarrage automatique à l’ouverture de session Windows, le chemin de l’exécutable actuel est aussi enregistré dans la valeur `UnfocusMute` sous `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

### Informations non stockées

UnfocusMute ne stocke pas l’historique d’utilisation, les journaux d’activité, les journaux d’erreurs, les données audio, les titres de fenêtres, les frappes au clavier ni aucune autre information que celles indiquées ci-dessus dans « Informations stockées ».

### Suppression

Pour supprimer tous les fichiers liés à l’application, supprimez le dossier `UnfocusMute-windows-x64`, puis le dossier `%APPDATA%\UnfocusMute`.

Si vous avez déjà activé le démarrage automatique, supprimez aussi la valeur `UnfocusMute` sous `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

<br>

## Transparence et vérification des fichiers publiés

UnfocusMute est conçu pour être utilisé en toute sécurité dans les environnements courants. La plupart des utilisateurs n’ont donc pas besoin de suivre les étapes de vérification ci-dessous. Si vous ne souhaitez pas dépendre uniquement de la confiance accordée au développeur ou si vous accordez une importance particulière à la sécurité de la chaîne d’approvisionnement logicielle, vous pouvez suivre cette procédure publique pour vérifier l’origine et l’intégrité des fichiers téléchargés.

### Pourquoi une vérification distincte est nécessaire

Même après avoir examiné le code source du dépôt et l’avoir jugé sûr, vous ne pouvez pas conclure que les fichiers d’une version publiée sur GitHub ont effectivement été produits à partir de ce code. Si le compte du développeur est compromis ou si les droits de publication sont détournés, des fichiers sans rapport avec le code source publié pourraient être distribués.

La comparaison des hachages SHA-256 permet de confirmer qu’un fichier téléchargé correspond à la somme de contrôle publiée, mais elle ne prouve pas à partir de quel code source ni dans quel environnement de compilation il a été produit.

Afin de traiter ces risques de chaîne d’approvisionnement de façon transparente, UnfocusMute publie une méthode permettant de vérifier directement qu’un fichier joint à une version publiée sur GitHub est un artefact officiel généré par GitHub Actions à partir du commit référencé par le tag de cette version.

<details>
<summary>Afficher la procédure de vérification</summary>

Installez d’abord [GitHub CLI](https://cli.github.com/). Exécutez ensuite les commandes ci-dessous dans PowerShell et saisissez la version publiée lorsque vous y êtes invité.

```powershell
$version = Read-Host "Saisissez la version publiée (par exemple, v1.5.0)"
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

Prérequis :

- Rust stable
- Visual Studio Build Tools 2022 ou Visual Studio 2022
- SDK Windows 10/11

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

Actualiser les mentions de licence des dépendances tierces :

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

Le résultat est créé dans `dist\UnfocusMute-windows-x64.zip`, avec le fichier de vérification SHA-256 `dist\UnfocusMute-windows-x64.zip.sha256` au même emplacement. Le ZIP contient l’exécutable avec numéro de version (`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md` et les README traduits au format `.txt` du dossier `docs`.

</details>

<br>

## Licence

Apache License 2.0. Consultez [LICENSE](../LICENSE) pour les détails.

Consultez [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) pour les mentions de licence des crates Rust tierces.
