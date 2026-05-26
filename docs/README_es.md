<div align="center">
  <img src="../assets/app-icon.png" alt="Icono de UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>App ligera para la bandeja de Windows que silencia automáticamente juegos y aplicaciones seleccionados cuando pierden el foco.</strong></p>

  <p>
    <a href="../README.md">English</a> · <a href="README_ko.md">한국어</a> · <a href="README_ja.md">日本語</a> · <a href="README_zh-CN.md">简体中文</a> · Español · <a href="README_fr.md">Français</a> · <a href="README_pt.md">Português</a> · <a href="README_hi.md">हिन्दी</a> · <a href="README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=Build&logo=githubactions&logoColor=white" alt="Build status"></a>
    &nbsp;
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    &nbsp;
    <a href="../LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>Totalmente local &nbsp;·&nbsp; Sin acceso a la red &nbsp;·&nbsp; Sin telemetría &nbsp;·&nbsp; No requiere instalación</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Descargar</a>
    · <a href="#uso">Uso</a>
    · <a href="#seguridad-y-privacidad">Privacidad</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute es una app pequeña y ligera para la bandeja de Windows que silencia automáticamente juegos y aplicaciones seleccionados cuando pasan a segundo plano.

No se limita a juegos: también puedes registrar aplicaciones normales como navegadores, apps de mensajería, lanzadores y reproductores multimedia.

<p align="center">
  <img src="../assets/screenshot_es.png" width="600" alt="Ventana principal de UnfocusMute">
</p>

Al estar compilada como app nativa en Rust, se ejecuta sin un runtime adicional. El ejecutable ocupa unos 500 KB.

El silenciamiento y la restauración solo se aplican a las sesiones que UnfocusMute cambió directamente. Las sesiones que ya habías silenciado no se modifican.

---

## Útil cuando

- Sueles usar Alt+Tab para salir de un juego o aplicación mientras sigue abierta.
- Un juego o app no ofrece su propia opción de silenciarse en segundo plano.
- Quieres silenciar solo el juego en segundo plano mientras sigues oyendo el navegador o una llamada.
- Un mismo `.exe` abre varios procesos y necesitas alternar entre gestionar la aplicación completa y controlar un PID concreto.

## Funciones

- Silencia automáticamente las aplicaciones registradas mientras están en segundo plano y restaura el audio al volver al primer plano.
- Registra aplicaciones desde la lista de aplicaciones en ejecución o escribiendo un nombre como `game.exe`.
- Admite entradas por `.exe`, entradas por PID de la instancia actual y `Vista PID`.
- Notas por aplicación, estado de silencio en tiempo real y controles `Pausar` / `Reanudar` por aplicación.
- Funcionamiento residente en bandeja, resumen de estado en bandeja, pausa global, acceso a la carpeta de configuración y protección contra instancias duplicadas.
- El botón de configuración en la esquina inferior izquierda reúne opciones de comportamiento, idioma, carpeta de configuración, Repositorio de GitHub e información de versión.
- Permite elegir idioma en el primer inicio y cambiar después dentro de la app entre English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية.
- La configuración se guarda localmente en `%APPDATA%\UnfocusMute\config.json`.

---

## Descargar y ejecutar

En Windows 10/11, descarga el paquete ZIP y extráelo para ejecutar la app.

| Paquete más reciente |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Archivo SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notas de la versión](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Mueve la carpeta extraída `UnfocusMute-windows-x64` a la ubicación donde quieras guardar la app y ejecuta `UnfocusMute-v<version>.exe` dentro de esa carpeta. Es una app independiente que no requiere instalación, y no necesitas Rust, Visual Studio Build Tools, MinGW ni otras herramientas de desarrollo.

> **Nota:** Como los certificados de firma de código tienen coste, la app se distribuye actualmente sin firma de código de Windows. En el primer inicio puede aparecer Windows SmartScreen o un aviso de "editor desconocido". Si quieres comprobar la integridad del archivo por tu cuenta, consulta la sección de verificación de archivos de la versión más abajo.

## Antes de usar

UnfocusMute funciona con los nombres de proceso, la información de la ventana en primer plano y las sesiones CoreAudio que proporciona Windows. Si una app aún no ha creado una sesión de audio, o si un controlador, permiso o herramienta de seguridad limita el acceso a la sesión, la lista o el control de silencio pueden estar limitados.

**Comportamiento al registrar por PID:** Windows no siempre proporciona el mismo PID para una sesión de audio y para la ventana en primer plano. Para compensarlo, UnfocusMute considera que la app volvió al primer plano cuando el nombre `.exe` del PID registrado coincide con el nombre `.exe` de la ventana activa.

Por eso, si hay varias instancias del mismo `.exe` ejecutándose a la vez, un PID concreto no siempre puede separarse perfectamente. En ese caso, el sonido puede restaurarse aunque otra instancia esté en primer plano.

**Compatibilidad con anti-cheat:** UnfocusMute no inyecta código en juegos, no lee memoria del juego, no intercepta entradas y no modifica archivos del juego. Solo usa la información de procesos/ventana en primer plano de Windows y los controles de silencio de sesiones CoreAudio, por lo que se espera que funcione sin problemas con la mayoría de sistemas anti-cheat, aunque no se puede garantizar compatibilidad con todos.

## Uso

1. Abre UnfocusMute.
2. Elige un idioma en la pantalla que aparece en el primer inicio. El idioma predeterminado es inglés.
3. Inicia el juego o la app que quieres silenciar cuando esté en segundo plano.
4. Abre la lista `Buscar proceso` o escribe una búsqueda, selecciona un elemento y pulsa `Registrar`.
5. Si necesitas registrar solo un PID concreto, pulsa `Vista PID` y elige el elemento individual. Las entradas por PID solo se aplican a la instancia que está en ejecución; si la app se reinicia con otro PID, selecciónala de nuevo.
6. Haz clic derecho en una app registrada para editar su nota o usar `Pausar`.
7. Abre `Configuración` desde la esquina inferior izquierda para cambiar opciones de comportamiento.
8. Al cerrar la ventana, la app permanece en la bandeja y sigue supervisando las aplicaciones registradas. Usa `Salir` para cerrarla por completo.

## Usar notas en apps registradas

Si el nombre del proceso no basta para recordar qué app es, haz clic derecho en la app registrada y elige `Editar nota`. La nota aparece encima del nombre del proceso en la lista de apps registradas y no afecta a la detección de apps.

Es útil cuando un mismo lanzador de juegos abre varios procesos, o cuando el nombre del ejecutable no deja claro para qué sirve.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - cliente del servidor de pruebas`

Las notas se guardan localmente junto con el resto de la configuración en `%APPDATA%\UnfocusMute\config.json`.

## Encontrar el nombre del ejecutable

Si no sabes qué nombre registrar, busca el ejecutable terminado en `.exe` en el Administrador de tareas.

1. Inicia primero la app que quieres registrar.
2. Usa `Alt`+`Tab` o `Windows`+`Tab` para salir de la pantalla del juego y volver a Windows.
3. Pulsa `Ctrl`+`Shift`+`Esc` para abrir el Administrador de tareas.
4. Ordena la lista de procesos por `CPU` y busca la app que acabas de iniciar.
5. Haz clic derecho en ese elemento y abre `Propiedades`.
6. Busca el nombre del ejecutable terminado en `.exe`, como `game.exe`, y añádelo a UnfocusMute.

---

## Seguridad y privacidad

UnfocusMute se ejecuta de forma completamente local. Funciona con normalidad aunque no tengas conexión a internet y no realiza solicitudes de red automáticas, no usa telemetría, no envía informes de fallos, no hace registro remoto ni recopila datos. Tampoco requiere permisos de administrador.

Como excepción, si pulsas el botón Repositorio de GitHub en Configuración, se abre el repositorio de GitHub del proyecto en tu navegador predeterminado.

**Lo que guarda:** Nombres de procesos registrados, PID opcionales, las notas que escribas, el idioma elegido, la posición de la ventana, las opciones de inicio y el estado de restauración de las apps silenciadas por UnfocusMute.
Estos ajustes de la app se guardan en `%APPDATA%\UnfocusMute\config.json` y no se envían a ningún sitio externo. Si activas el inicio automático al iniciar sesión en Windows, la ruta del ejecutable actual también se guarda en el valor `UnfocusMute` de `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

Para eliminar por completo cualquier rastro de la app, borra la carpeta de la app y después borra `%APPDATA%\UnfocusMute`. Si alguna vez activaste el inicio automático, borra también el valor del registro indicado arriba.

**Lo que no guarda:** No crea archivos de log de la app ni conserva historial de actividad entre sesiones.

La detección de sesiones de audio y el control de silencio solo usan las API CoreAudio de Windows, y UnfocusMute no inyecta código en procesos de juegos ni lee su memoria.

---

## Verificar archivos de la versión

No debes asumir que los archivos subidos a GitHub Releases siempre coinciden con el código fuente publicado en el repositorio.

Si se abusa de los permisos de publicación o una cuenta se ve comprometida, podrían subirse a una release archivos compilados desde otro código o archivos modificados.

Por transparencia, UnfocusMute ofrece una forma de verificar que los archivos subidos a GitHub Releases son artefactos oficiales generados por GitHub Actions a partir del código fuente de este repositorio en la etiqueta correspondiente.

El ZIP de la versión y el archivo de suma SHA-256 se generan automáticamente con GitHub Actions, y cada archivo se entrega con una atestación de procedencia de compilación (attestation).

Los comandos de abajo permiten comprobar que el ZIP descargado fue generado por la compilación oficial de este repositorio.

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

La compilación de release está configurada para reducir el tamaño del binario. El perfil de release de `Cargo.toml` elimina símbolos, activa LTO, usa una sola codegen unit, establece `panic = "abort"` y optimiza por tamaño.

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

El resultado se crea en `dist\UnfocusMute-windows-x64.zip`, junto con el archivo de verificación SHA-256 `dist\UnfocusMute-windows-x64.zip.sha256`. El ZIP incluye el ejecutable con versión (`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md` y los README `.txt` localizados por idioma bajo `docs`.

---

## Licencia

Apache License 2.0. Consulta [LICENSE](../LICENSE) para más detalles.

Consulta [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para los avisos de licencias de crates Rust de terceros.
