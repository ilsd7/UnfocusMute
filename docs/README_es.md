<div align="center">
  <img src="../assets/app-icon.png" alt="Icono de UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>Aplicación ligera que se ejecuta en la bandeja del sistema de Windows y silencia automáticamente juegos y aplicaciones seleccionados cuando pierden el foco.</strong></p>

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

UnfocusMute es una aplicación pequeña y ligera que se ejecuta en la bandeja del sistema de Windows, silencia automáticamente juegos y aplicaciones seleccionados cuando pasan a segundo plano, y restaura el audio cuando vuelven al primer plano.

- Está compilada como aplicación nativa en Rust, así que se ejecuta sin un entorno de ejecución adicional.
- El ejecutable ocupa unos 500 KB.
- No se limita a juegos: también puedes registrar aplicaciones comunes como navegadores, aplicaciones de mensajería, lanzadores y reproductores multimedia.
- El silenciamiento y la restauración solo se aplican a las sesiones que UnfocusMute cambió directamente; las sesiones que tú ya habías silenciado no se modifican.

<p align="center">
  <img src="../assets/screenshot_es.png" width="600" alt="Ventana principal de UnfocusMute">
</p>

---

## Útil cuando

- Sueles alternar con Alt+Tab entre un juego o una aplicación y otras ventanas mientras sigue en ejecución.
- Quieres mantener en silencio una aplicación que no tiene opción de silencio en segundo plano.
- Quieres silenciar solo el audio en segundo plano de una aplicación concreta mientras haces otras cosas.

## Funciones

- Silencia automáticamente las aplicaciones registradas al pasar a segundo plano y restaura el audio cuando vuelven al primer plano.
- Permite elegir una aplicación con sesión de audio, buscarla en `Todos los procesos` o escribir un nombre como `game.exe`.
- Registra una aplicación completa por su nombre de archivo `.exe`, o solo la instancia en ejecución mediante PID.
- Notas por aplicación, estado en tiempo real y pausa / reanudación individual.
- Sigue supervisando desde la bandeja al cerrar la ventana, con acciones para `Abrir` / `Ocultar en la bandeja` / `Pausar` / `Salir`.
- Opciones para `Iniciar minimizado en la bandeja`, `Ejecutar al iniciar sesión en Windows` y `Restaurar el audio silenciado por UnfocusMute al salir`.
- Permite elegir el idioma en el primer inicio y cambiar al instante entre 9 idiomas dentro de la aplicación.

## Descargar y ejecutar

En Windows 10/11, descarga el paquete ZIP y extráelo para ejecutar la aplicación.

| Paquete más reciente |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Suma de comprobación SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notas de la versión](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Después de extraer el ZIP, mueve la carpeta `UnfocusMute-windows-x64` a la ubicación donde quieras guardar la aplicación y ejecuta `UnfocusMute-v<version>.exe` dentro de esa carpeta.

UnfocusMute es una aplicación independiente que no requiere instalación. Tampoco necesitas instalar Rust, Visual Studio Build Tools, MinGW ni otras herramientas de desarrollo.

> **Nota:** Como los certificados de firma de código tienen un costo, la aplicación se distribuye actualmente sin firma de código de Windows. En el primer inicio puede aparecer un aviso de Windows SmartScreen o de "editor desconocido". Si quieres comprobar la integridad del archivo por tu cuenta, consulta [Verificar archivos de la versión](#verificar-archivos-de-la-versión).

---

## Antes de usar

UnfocusMute funciona con los nombres de proceso, la información de la ventana en primer plano y las sesiones CoreAudio que proporciona Windows. Por eso, si un controlador, un ajuste de permisos o una herramienta de seguridad limita el acceso a las sesiones, el control de silencio puede no funcionar correctamente.

La lista predeterminada muestra solo las aplicaciones que tienen una sesión de audio en ese momento. Si una aplicación aún no ha creado una sesión de audio, cambia a `Todos los procesos` para buscarla entre los procesos `.exe` en ejecución. Usa `Solo sesiones de audio` para volver a la lista filtrada.

**Comportamiento al registrar por PID:** Windows no siempre proporciona el mismo PID para una sesión de audio y para la ventana en primer plano. Para compensarlo, UnfocusMute considera que la aplicación ha vuelto al primer plano cuando el nombre del ejecutable (`.exe`) asociado al PID registrado coincide con el nombre del ejecutable de la ventana activa.

Por eso, si hay varias instancias del mismo `.exe` ejecutándose a la vez, no siempre es posible distinguir perfectamente una instancia concreta. En ese caso, el sonido puede restaurarse aunque otra instancia esté en primer plano.

**Compatibilidad con anti-cheat:** UnfocusMute no inyecta código en juegos, no lee memoria del juego, no intercepta la entrada del usuario ni modifica archivos del juego. Solo usa la información de procesos/ventana en primer plano de Windows y los controles de silencio de sesiones CoreAudio, por lo que está diseñado para evitar conflictos con la mayoría de los sistemas anti-cheat, aunque no se puede garantizar compatibilidad con todos.

---

## Uso

1. Abre UnfocusMute.
2. Elige un idioma en la pantalla que aparece en el primer inicio. El idioma predeterminado es inglés.
3. Inicia el juego o la aplicación que quieres registrar.
4. Selecciona una aplicación de la lista o búscala en `Buscar proceso`, y luego haz clic en `Registrar`. Si la aplicación aún no ha creado una sesión de audio, cambia a `Todos los procesos` para revisar la lista de todos los procesos en ejecución. Usa `Solo sesiones de audio` para volver a la lista filtrada. Si no aparece en la lista, escribe manualmente el nombre del archivo `.exe`.
5. Si necesitas registrar solo un PID concreto, haz clic en `Vista PID` y elige la entrada correspondiente. Las entradas por PID solo se aplican a la instancia que está en ejecución; si la aplicación se reinicia con otro PID, tendrás que registrarla de nuevo.
6. Haz clic con el botón derecho en una aplicación registrada para editar su nota o usar `Pausar` solo en esa aplicación.
7. Abre `Configuración` desde la esquina inferior izquierda para cambiar opciones de comportamiento.
8. Al cerrar la ventana, la aplicación permanece en la bandeja y sigue supervisando las aplicaciones registradas. Usa `Salir` para cerrarla por completo.

---

## Notas para aplicaciones registradas

Si el nombre del proceso no basta para recordar de qué aplicación se trata, haz clic con el botón derecho en la aplicación registrada y elige `Editar nota`. La nota aparece encima del nombre del proceso en la lista de aplicaciones registradas y no afecta a la forma en que UnfocusMute identifica la aplicación.

Es útil cuando un mismo lanzador de juegos abre varios procesos, o cuando el nombre del ejecutable no deja claro para qué sirve.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - cliente del servidor de pruebas`

Las notas se guardan localmente junto con el resto de la configuración en `%APPDATA%\UnfocusMute\config.json`.

---

## Encontrar el nombre del ejecutable

Si no sabes qué nombre registrar, busca el ejecutable terminado en `.exe` en el Administrador de tareas.

1. Inicia primero la aplicación que quieres registrar.
2. Usa `Alt`+`Tab` o `Windows`+`Tab` para volver al escritorio de Windows.
3. Pulsa `Ctrl`+`Shift`+`Esc` para abrir el Administrador de tareas.
4. Ordena la lista de procesos por `CPU` y busca la aplicación que acabas de iniciar.
5. Haz clic con el botón derecho en ese elemento y abre `Propiedades`.
6. Busca el nombre del ejecutable terminado en `.exe`, como `game.exe`, y regístralo en UnfocusMute.

---

## Solución de problemas

Si una aplicación no aparece en la lista, o si el registro por PID no se comporta como esperabas, revisa primero [Antes de usar](#antes-de-usar) y [Encontrar el nombre del ejecutable](#encontrar-el-nombre-del-ejecutable).

Si el indicador de estado de la parte superior cambia a `Requiere atención`, haz clic en `Detalles` para ver el mensaje de error detallado.

Si el problema continúa, abre una incidencia en GitHub.

Si sospechas que el problema es una vulnerabilidad de seguridad, no publiques los detalles en una incidencia pública. Usa el proceso de notificación privada y consulta [SECURITY.md](../SECURITY.md) para más información.

---

## Archivo de configuración

Si necesitas revisar directamente el archivo de configuración o hacer una copia de seguridad, haz clic en `Abrir carpeta de configuración` en Configuración. Se abrirá el Explorador de archivos en la carpeta `%APPDATA%\UnfocusMute`, donde se guarda la configuración.

Puedes editar el archivo de configuración directamente, pero si su formato no es válido y no se puede leer, se guarda una copia como `config.invalid-<timestamp>.json`. Si el problema se detecta durante el inicio de la aplicación, la configuración se restaura a los valores predeterminados; si se detecta mientras la aplicación está en ejecución, se crea un nuevo archivo de configuración a partir de los ajustes actuales.

---

## Seguridad y privacidad

UnfocusMute se ejecuta de forma completamente local. Funciona con normalidad aunque no tengas conexión a internet y no requiere permisos de administrador. Tampoco realiza solicitudes de red automáticas, ni usa telemetría, ni envía informes de fallos ni registros a servicios remotos, ni recopila datos.

Como excepción, el repositorio de GitHub del proyecto solo se abre en tu navegador predeterminado cuando pulsas el botón `GitHub` en Configuración.

La detección de sesiones de audio y el control de silencio solo usan las API CoreAudio de Windows. UnfocusMute no inyecta código en los procesos de destino, no lee su memoria ni intercepta la entrada del usuario.

### Información que guarda

UnfocusMute solo guarda en `%APPDATA%\UnfocusMute\config.json` la configuración necesaria para funcionar.

- Nombres de procesos registrados
- PID registrados directamente
- Último estado de silencio de las aplicaciones registradas
- Notas que escribas
- Idioma y configuración elegidos
- Posición de la ventana

Esta información no se envía a ningún sitio.

Si activas el inicio automático al iniciar sesión en Windows, la ruta del ejecutable actual también se guarda en el valor `UnfocusMute` de `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

### Información que no guarda

UnfocusMute no guarda historial de uso, registros de actividad, registros de errores, datos de audio, títulos de ventanas, pulsaciones de teclas ni ninguna información que no aparezca arriba en "Información que guarda".

### Cómo eliminar la aplicación y sus datos

Para eliminar todos los archivos relacionados con la aplicación, borra la carpeta `UnfocusMute-windows-x64` y después borra `%APPDATA%\UnfocusMute`.

Si alguna vez activaste el inicio automático, borra también el valor `UnfocusMute` en `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

---

## Verificar archivos de la versión

No debes asumir que los archivos subidos a GitHub Releases siempre coinciden con el código fuente publicado en el repositorio.

Si se hace un uso indebido de los permisos para publicar versiones o una cuenta se ve comprometida, podrían subirse archivos compilados a partir de otro código o archivos alterados.

Por transparencia, UnfocusMute ofrece una forma de verificar que los archivos subidos a GitHub Releases son artefactos oficiales generados por GitHub Actions a partir del código fuente de la etiqueta correspondiente en este repositorio.

El ZIP de la versión y el archivo de suma de comprobación SHA-256 se generan automáticamente con GitHub Actions, y cada archivo se entrega con una atestación de procedencia de la compilación (attestation).

Los comandos de abajo permiten comprobar que el ZIP descargado fue generado por la compilación oficial de este repositorio.

```powershell
gh attestation verify .\UnfocusMute-windows-x64.zip -R ilsd7/UnfocusMute
gh attestation verify .\UnfocusMute-windows-x64.zip.sha256 -R ilsd7/UnfocusMute
```

---

## Compilar desde el código fuente

El objetivo recomendado para las versiones publicadas es `x86_64-pc-windows-msvc`.

Requisitos:

- Rust stable
- Visual Studio Build Tools 2022 o Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

La compilación en modo release está configurada para reducir el tamaño del binario. El perfil de release de `Cargo.toml` elimina símbolos, activa LTO, usa una sola unidad de generación de código (codegen unit), establece `panic = "abort"` y optimiza por tamaño.

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

El resultado se crea en `dist\UnfocusMute-windows-x64.zip`, junto con el archivo de verificación SHA-256 `dist\UnfocusMute-windows-x64.zip.sha256`. El ZIP incluye el ejecutable con versión (`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md` y los documentos README `.txt` de cada idioma de la carpeta `docs`.

---

## Licencia

Apache License 2.0. Consulta [LICENSE](../LICENSE) para más detalles.

Consulta [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para los avisos de licencias de crates de Rust de terceros.
