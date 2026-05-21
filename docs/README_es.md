<div align="center">
  <img src="../assets/app-icon.png" alt="Icono de UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>App de bandeja para Windows ligera, portátil y completamente local que silencia juegos y apps seleccionados cuando pierden el foco.<br>Solo restaura el audio que ella misma silenció: sin red ni logs.</strong></p>

  <p>
    <a href="../README.md">한국어</a> · <a href="README_en.md">English</a> · <a href="README_ja.md">日本語</a> · <a href="README_zh-CN.md">简体中文</a> · Español · <a href="README_fr.md">Français</a> · <a href="README_pt.md">Português</a> · <a href="README_hi.md">हिन्दी</a> · <a href="README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=CI&logo=githubactions&logoColor=white" alt="CI status"></a>
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    <a href="../LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>Funcionamiento completamente local &nbsp;·&nbsp; Sin conexiones de red &nbsp;·&nbsp; Sin archivos de log &nbsp;·&nbsp; No requiere permisos de administrador</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Descargar</a>
    · <a href="#uso">Uso</a>
    · <a href="#seguridad-y-privacidad">Privacidad</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute es una app pequeña y ligera para la bandeja de Windows que silencia automáticamente solo el sonido del juego o la app seleccionada cuando pasa a segundo plano. No sirve solo para juegos: también puedes registrar navegadores, apps de chat, launchers, reproductores multimedia y otras apps que aparezcan como sesiones de audio de Windows.

Al estar compilada como app nativa en Rust, se ejecuta sin un runtime adicional. El ejecutable actual ocupa unos 489 KB, menos de 1 MB.

<p align="center">
  <img src="../assets/screenshot_es.png" alt="Ventana de UnfocusMute">
</p>

El silencio y la restauración solo se aplican a las sesiones que UnfocusMute cambió directamente. Las sesiones que ya habías silenciado se dejan intactas.

---

## Útil cuando

- Sueles usar Alt+Tab para salir de un juego o app mientras sigue abierta.
- Un juego o app no ofrece su propia opción de silenciarse en segundo plano.
- Quieres silenciar solo el juego en segundo plano mientras sigues oyendo el navegador o una llamada.
- Un mismo `.exe` abre varios procesos y necesitas alternar entre gestionar toda la app y controlar un PID concreto.

## Funciones

- Silencia automáticamente las apps registradas mientras están en segundo plano y restaura el audio al volver al primer plano.
- Añade objetivos desde la lista de apps en ejecución o escribiendo un nombre como `game.exe`.
- Admite objetivos por `.exe`, objetivos por PID de la instancia actual y `Ver PID`.
- Notas por app, `Excluir del silencio automático` e `Incluir en el silencio automático` por app.
- Permanencia en bandeja, pausa global, acceso a la carpeta de configuración y protección contra instancias duplicadas.
- Permite elegir idioma en el primer inicio y cambiar después dentro de la app entre English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية.
- La configuración se guarda localmente en `%APPDATA%\UnfocusMute\config.json`.

---

## Descargar y ejecutar

En Windows 10/11, descarga el paquete ZIP y extráelo para ejecutar la app.

| Paquete más reciente |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Archivo SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notas de la versión](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Mueve la carpeta extraída `UnfocusMute-windows-x64` a la ubicación donde quieras guardar la app y ejecuta `UnfocusMute-<version>.exe` dentro de esa carpeta. Es una app portátil: no hay instalador y no necesitas Rust, Visual Studio Build Tools, MinGW ni otras herramientas de desarrollo.

> **Nota:** Debido al coste de los certificados de firma de código, la app se distribuye actualmente sin firma de código de Windows. En el primer inicio puede aparecer Windows SmartScreen o un aviso de "editor desconocido". Si quieres comprobar la integridad del archivo por tu cuenta, consulta la sección de verificación de archivos de release más abajo.

## Antes de usar

UnfocusMute funciona con los nombres de proceso, la información de la ventana en primer plano y las sesiones CoreAudio que proporciona Windows. Si una app aún no ha creado una sesión de audio, o si un controlador, permiso o programa de seguridad limita el acceso a la sesión, la lista o el control de silencio pueden estar limitados.

Hay un comportamiento que conviene conocer al registrar por PID. Windows no siempre proporciona el mismo PID para una sesión de audio y para la ventana en primer plano. Para compensarlo, UnfocusMute considera que la app volvió al primer plano cuando el nombre `.exe` del PID registrado coincide con el nombre `.exe` de la ventana activa. Por eso, si hay varias instancias del mismo `.exe`, un PID concreto no siempre puede separarse perfectamente, y el sonido puede restaurarse cuando otra instancia está en primer plano.

UnfocusMute no inyecta código en juegos, no lee memoria del juego, no intercepta entradas y no modifica archivos del juego. Solo usa la información de procesos/ventana en primer plano de Windows y los controles de silencio de sesiones CoreAudio, por lo que debería funcionar sin problemas con la mayoría de sistemas anti-cheat, aunque no se puede garantizar compatibilidad con todos.

## Uso

1. Abre UnfocusMute.
2. Elige un idioma en la pantalla de primer inicio. English viene seleccionado de forma predeterminada.
3. Inicia el juego o la app que quieres silenciar cuando esté en segundo plano.
4. Abre la lista `Buscar proceso` o escribe una búsqueda, selecciona un elemento y pulsa `Añadir selección`.
5. Si necesitas registrar solo un PID concreto, pulsa `Ver PID` y elige el elemento individual. Los objetivos por PID solo se aplican a la instancia que está en ejecución; si la app se reinicia con otro PID, selecciónala de nuevo.
6. Haz clic derecho en una app registrada para editar su nota o `Excluir del silencio automático` esa app.
7. Al cerrar la ventana, la app permanece en la bandeja y sigue vigilando. Usa `Salir` para cerrarla por completo.

## Usar notas en apps registradas

Si el nombre del proceso no basta para recordar qué app es, haz clic derecho en la app registrada y elige `Editar nota`. La nota aparece junto al nombre del proceso en la lista de apps registradas y no afecta a la detección del objetivo.

Es útil cuando un mismo launcher abre varios procesos, o cuando el nombre del ejecutable no deja claro para qué sirve.

- `htgame.exe - NTE`
- `chrome.exe (PID 18432) - perfil para reproducir música`
- `game.exe (PID 21976) - cliente del servidor de pruebas`
- `launcher.exe - launcher antes del juego real`

Las notas se guardan localmente junto con el resto de la configuración en `%APPDATA%\UnfocusMute\config.json`.

## Encontrar el nombre del ejecutable del juego

Si no sabes qué nombre registrar, busca el ejecutable terminado en `.exe` en el Administrador de tareas.

1. Inicia primero el juego.
2. Usa `Alt`+`Tab` o `Windows`+`Tab` para salir de la pantalla del juego y volver a Windows.
3. Pulsa `Ctrl`+`Shift`+`Esc` para abrir el Administrador de tareas.
4. Ordena la lista de procesos por `CPU` y busca el juego que acabas de iniciar.
5. Haz clic derecho en el juego y abre `Propiedades`.
6. Busca el nombre del ejecutable terminado en `.exe`, como `game.exe`, y añádelo a UnfocusMute.

---

## Seguridad y privacidad

UnfocusMute es una app completamente local. Todo ocurre dentro de tu PC y funciona con normalidad aunque no tengas conexión a internet.

**Lo que guarda** — Nombres de procesos registrados, PID opcionales, idioma de la interfaz, posición de la ventana y opciones de inicio. Estos datos solo se guardan en `%APPDATA%\UnfocusMute\config.json` y no se envían fuera.

**Lo que no guarda** — No crea archivos de log de la app. Tampoco conserva historial de actividad entre sesiones.

**Lo que no hace** — No hay solicitudes de red, telemetría, informes de fallos ni registro remoto. Tampoco requiere permisos de administrador.

La detección de sesiones de audio y el control de silencio solo usan las API CoreAudio de Windows, y UnfocusMute no inyecta código en procesos de juegos ni lee su memoria.

---

## Verificar archivos de release

Por seguridad, los usuarios deben poder protegerse si un desarrollador distribuye maliciosamente archivos distintos del código publicado en el repositorio, o si los archivos de release se alteran por una cuenta comprometida u otro incidente similar. Para eso hace falta un procedimiento que permita verificar directamente que los archivos subidos a GitHub Releases son builds oficiales que coinciden con el código fuente público.

Por seguridad y transparencia, UnfocusMute ofrece un método de verificación que permite confirmar directamente que los archivos subidos a GitHub Releases son builds oficiales que coinciden con el código fuente de este repositorio.

El ZIP de release y el archivo de checksum SHA-256 se generan con GitHub Actions, el sistema de build automático de GitHub, y ambos archivos se entregan con attestations que prueban su origen.

Los comandos de abajo permiten verificar que el ZIP descargado desde GitHub Releases es idéntico al build oficial de este repositorio.

```powershell
gh attestation verify .\UnfocusMute-windows-x64.zip -R ilsd7/UnfocusMute
gh attestation verify .\UnfocusMute-windows-x64.zip.sha256 -R ilsd7/UnfocusMute
```

---

## Compilar desde el código fuente

El objetivo de release recomendado es `x86_64-pc-windows-msvc`.

Requisitos:

- Rust stable
- Visual Studio Build Tools 2022 o Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

La compilación de release está configurada para reducir el tamaño del binario. El release profile de `Cargo.toml` elimina símbolos, activa LTO, usa una sola codegen unit, establece `panic = "abort"` y optimiza por tamaño.

Ejecutable:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

ZIP de distribución:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Actualizar avisos de licencias de terceros:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

El resultado se crea en `dist\UnfocusMute-windows-x64.zip`, junto con el archivo de verificación SHA-256 `dist\UnfocusMute-windows-x64.zip.sha256`. El ZIP incluye el ejecutable con versión (`UnfocusMute-<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md` y los README `.txt` localizados por idioma bajo `docs`.

---

## Licencia

Apache License 2.0. Consulta [LICENSE](../LICENSE) para más detalles.

Consulta [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para los avisos de licencias de crates Rust de terceros.
