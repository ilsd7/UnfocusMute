<div align="center">
  <img src="../assets/app-icon.png" alt="Icône UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>Application légère pour la zone de notification Windows qui coupe automatiquement le son des jeux et applications choisis lorsqu’ils ne sont plus au premier plan.</strong></p>

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

  <p>Entièrement local &nbsp;·&nbsp; Aucun accès réseau &nbsp;·&nbsp; Aucune télémétrie &nbsp;·&nbsp; Aucune installation requise</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Télécharger</a>
    · <a href="#utilisation">Utilisation</a>
    · <a href="#sécurité-et-confidentialité">Confidentialité</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute est une petite application légère pour la zone de notification Windows qui coupe automatiquement le son des jeux et applications choisis lorsqu’ils passent en arrière-plan.

Elle ne se limite pas aux jeux : vous pouvez aussi enregistrer des applications classiques comme des navigateurs, des messageries, des lanceurs et des lecteurs multimédias.

<p align="center">
  <img src="../assets/screenshot_fr.png" width="600" alt="Fenêtre principale d’UnfocusMute">
</p>

Compilée comme application native Rust, elle s’exécute sans runtime séparé. L’exécutable fait environ 500 Ko.

La mise en sourdine et la restauration du son ne s’appliquent qu’aux sessions qu’UnfocusMute a modifiées lui-même. Les sessions que vous aviez déjà mises en sourdine ne sont pas modifiées.

---

## Utile quand

- Vous laissez un jeu ou une application ouvert et passez souvent à une autre fenêtre avec Alt+Tab.
- Un jeu ou une application ne propose pas sa propre option de sourdine en arrière-plan.
- Vous voulez couper seulement le son d’un jeu en arrière-plan tout en gardant audible un navigateur ou une application d’appel.
- Le même `.exe` lance plusieurs processus et vous devez passer d’une gestion de l’application entière à un PID précis.

## Fonctionnalités

- Met automatiquement en sourdine les applications enregistrées lorsqu’elles sont en arrière-plan et rétablit leur son à leur retour au premier plan.
- Enregistrement depuis la liste des applications avec session audio ou en saisissant un nom comme `game.exe`.
- Prend en charge l’enregistrement par `.exe`, l’enregistrement par PID de l’instance en cours et `Vue PID`.
- Notes par application, état de mise en sourdine en temps réel et commandes `Pause` / `Reprendre` par application.
- Fonctionnement dans la zone de notification, résumé d’état dans la zone de notification, pause globale, accès au dossier de configuration et protection contre les instances multiples.
- Le bouton Paramètres en bas à gauche regroupe les options de comportement, la langue, le dossier de configuration, Dépôt GitHub et les informations de version.
- Choix de la langue au premier démarrage, puis changement immédiat dans l’application entre English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية.
- Les réglages sont stockés localement dans `%APPDATA%\UnfocusMute\config.json`.

---

## Télécharger et lancer

Sous Windows 10/11, téléchargez le paquet ZIP puis extrayez-le pour lancer l’application.

| Paquet le plus récent |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Fichier SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notes de version](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Une fois le ZIP extrait, déplacez le dossier `UnfocusMute-windows-x64` à l’emplacement où vous souhaitez conserver l’application, puis lancez `UnfocusMute-v<version>.exe` depuis ce dossier.

UnfocusMute est une application autonome qui ne nécessite aucune installation. Vous n’avez pas non plus besoin d’installer Rust, Visual Studio Build Tools, MinGW ni d’autres outils de développement.

> **Remarque :** Comme les certificats de signature de code ont un coût, l’application est actuellement distribuée sans signature de code Windows. Windows SmartScreen ou un avertissement d’éditeur inconnu peut apparaître au premier lancement. Pour vérifier vous-même l’intégrité du fichier, consultez la section de vérification des fichiers de release ci-dessous.

## À savoir avant utilisation

UnfocusMute s’appuie sur les noms de processus, les informations de fenêtre au premier plan et les sessions CoreAudio fournies par Windows. Si une application n’a pas encore créé de session audio, ou si un pilote, un réglage d’autorisation ou un logiciel de sécurité limite l’accès aux sessions, l’affichage dans la liste ou le contrôle de sourdine peut être limité.

**Comportement de l’enregistrement par PID :** Windows ne fournit pas toujours le même PID pour une session audio et pour la fenêtre au premier plan. Pour compenser cela, UnfocusMute considère que l’application est revenue au premier plan lorsque le nom `.exe` du PID enregistré correspond au nom `.exe` de la fenêtre actuellement active.

Si plusieurs instances du même `.exe` sont ouvertes en même temps, un PID précis ne peut donc pas toujours être séparé parfaitement. Dans ce cas, le son peut être rétabli lorsqu’une autre instance est au premier plan.

**Compatibilité anti-triche :** UnfocusMute n’injecte pas de code dans les jeux, ne lit pas la mémoire du jeu, n’intercepte pas les entrées et ne modifie pas les fichiers du jeu. Il utilise seulement les informations de processus/fenêtre au premier plan de Windows et les commandes de sourdine des sessions CoreAudio. Il devrait donc fonctionner sans problème avec la plupart des systèmes anti-triche, mais la compatibilité avec tous les systèmes anti-triche ne peut pas être garantie.

## Utilisation

1. Lancez UnfocusMute.
2. Choisissez la langue dans l’écran affiché au premier démarrage. La langue par défaut est l’anglais.
3. Lancez le jeu ou l’application à mettre en sourdine lorsqu’il passe en arrière-plan.
4. Ouvrez la liste `Rechercher un processus` ou saisissez un terme de recherche, choisissez une application avec session audio, puis cliquez sur `Enregistrer`. Les applications qui n’ont pas encore créé de session audio peuvent ne pas apparaître ; dans ce cas, saisissez le nom `.exe` manuellement.
5. Si vous devez enregistrer seulement un PID précis, cliquez sur `Vue PID` et choisissez l’entrée concernée. Une entrée par PID n’est valable que pour l’instance actuellement ouverte ; si l’application redémarre avec un autre PID, sélectionnez-la à nouveau.
6. Faites un clic droit sur une application enregistrée pour modifier sa note ou utiliser `Pause`.
7. Ouvrez `Paramètres` en bas à gauche pour modifier le comportement.
8. Fermer la fenêtre laisse l’application dans la zone de notification, où elle continue de surveiller les applications enregistrées. Cliquez sur `Quitter` pour l’arrêter complètement.

## Utiliser les notes des applications enregistrées

Si le nom du processus ne suffit pas à reconnaître l’application, faites un clic droit sur l’application enregistrée et choisissez `Modifier la note`. La note s’affiche au-dessus du nom du processus dans la liste, sans influencer la détection de l’application.

C’est utile lorsqu’un même lanceur de jeu ouvre plusieurs processus, ou lorsqu’un nom d’exécutable n’indique pas clairement son rôle.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - client du serveur de test`

Les notes sont enregistrées localement avec les autres réglages dans `%APPDATA%\UnfocusMute\config.json`.

## Trouver le nom de l’exécutable

Si vous ne savez pas quel nom enregistrer, vérifiez dans le Gestionnaire des tâches le nom de l’exécutable qui se termine par `.exe`.

1. Lancez d’abord l’application à enregistrer.
2. Utilisez `Alt`+`Tab` ou `Windows`+`Tab` pour quitter l’écran du jeu et revenir à Windows.
3. Appuyez sur `Ctrl`+`Shift`+`Esc` pour ouvrir le Gestionnaire des tâches.
4. Triez la liste des processus par `CPU` afin de retrouver l’application que vous venez de lancer.
5. Faites un clic droit sur cet élément et ouvrez `Propriétés`.
6. Relevez le nom de l’exécutable se terminant par `.exe`, par exemple `game.exe`, puis ajoutez-le à UnfocusMute.

---

## Sécurité et confidentialité

UnfocusMute fonctionne entièrement en local. L’application marche normalement même sans connexion internet et n’effectue pas de requêtes réseau automatiques, n’utilise pas de télémétrie, n’envoie pas de rapports de crash, ne fait pas de journalisation distante et ne collecte pas de données. Elle ne demande pas non plus de droits administrateur.

Par exception, le dépôt GitHub du projet ne s’ouvre dans votre navigateur par défaut que lorsque vous cliquez sur le bouton `Dépôt GitHub` dans Paramètres.

**Ce qui est stocké :** Les noms de processus enregistrés, les PID que vous avez enregistrés directement, le dernier état de mise en sourdine des applications enregistrées, les notes que vous écrivez, la langue et les réglages choisis, ainsi que la position de la fenêtre.

Ces valeurs sont enregistrées dans `%APPDATA%\UnfocusMute\config.json` et ne sont envoyées nulle part.

Si vous activez le démarrage automatique à l’ouverture de session Windows, le chemin de l’exécutable actuel est aussi enregistré dans la valeur `UnfocusMute` sous `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

Pour supprimer tous les fichiers liés à l’application, supprimez le dossier de l’application puis `%APPDATA%\UnfocusMute`.
Si vous avez déjà activé le démarrage automatique, supprimez aussi la valeur de registre indiquée ci-dessus.

**Ce qui n’est pas stocké :** Historique d’utilisation, journaux d’activité, données audio, titres de fenêtres, frappes au clavier ou toute autre information qui n’est pas indiquée ci-dessus dans « Ce qui est stocké ».

La détection des sessions audio et le contrôle de la sourdine utilisent uniquement les API Windows CoreAudio, et UnfocusMute n’injecte pas de code dans les processus de jeux ni ne lit leur mémoire.

---

## Vérifier les fichiers de release

Il ne faut pas supposer que les fichiers mis en ligne sur GitHub Releases correspondent toujours au code source publié dans le dépôt.

Si les droits de publication sont détournés ou si un compte est compromis, des fichiers compilés depuis un autre code ou des fichiers modifiés pourraient être ajoutés à une release.

Par transparence, UnfocusMute fournit une méthode permettant de vérifier que les fichiers mis en ligne sur GitHub Releases sont bien des artefacts officiels générés par GitHub Actions à partir du code source de ce dépôt pour le tag correspondant.

Le fichier ZIP de release et le fichier de somme de contrôle SHA-256 sont générés automatiquement par GitHub Actions, et chaque fichier est fourni avec une attestation de provenance de build.

Les commandes ci-dessous permettent de vérifier que le ZIP téléchargé a été généré par la compilation officielle de ce dépôt.

```powershell
gh attestation verify .\UnfocusMute-windows-x64.zip -R ilsd7/UnfocusMute
gh attestation verify .\UnfocusMute-windows-x64.zip.sha256 -R ilsd7/UnfocusMute
```

---

## Compiler depuis le code source

La cible de release recommandée est `x86_64-pc-windows-msvc`.

Prérequis :

- Rust stable
- Visual Studio Build Tools 2022 ou Visual Studio 2022
- SDK Windows 10/11

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

Les builds de release sont configurés pour réduire la taille du binaire. Le profil de release de `Cargo.toml` retire les symboles, active LTO, utilise une seule codegen unit, définit `panic = "abort"` et optimise pour la taille.

Exécutable :

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

ZIP de distribution :

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Actualiser les avis de licences tierces :

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

Le résultat est créé dans `dist\UnfocusMute-windows-x64.zip`, avec le fichier de vérification SHA-256 `dist\UnfocusMute-windows-x64.zip.sha256` au même emplacement. Le ZIP contient l’exécutable avec version (`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md` et les README `.txt` localisés par langue dans `docs`.

---

## Licence

Apache License 2.0. Consultez [LICENSE](../LICENSE) pour les détails.

Consultez [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) pour les avis de licences des crates Rust tierces.
