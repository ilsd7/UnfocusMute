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

<br>

UnfocusMute es una aplicación pequeña y ligera que se ejecuta en la bandeja del sistema de Windows, silencia automáticamente juegos y aplicaciones seleccionados cuando pasan a segundo plano, y restaura el audio cuando vuelven al primer plano.

- Está compilada como aplicación nativa en Rust, así que se ejecuta sin un entorno de ejecución adicional.
- El ejecutable ocupa unos 500 KB.
- No se limita a juegos: también puedes registrar aplicaciones comunes como navegadores, aplicaciones de mensajería, lanzadores y reproductores multimedia.
- El silenciamiento y la restauración solo se aplican a las sesiones que UnfocusMute cambió directamente; las sesiones que tú ya habías silenciado no se modifican.

<p align="center">
  <img src="../assets/screenshot_es.png" width="600" alt="Ventana principal de UnfocusMute">
</p>

<br>

## Útil cuando

- Sueles alternar con Alt+Tab entre un juego o una aplicación y otras ventanas mientras sigue en ejecución.
- Quieres mantener en silencio una aplicación que no tiene opción de silencio en segundo plano.
- Quieres silenciar solo el audio en segundo plano de una aplicación concreta mientras haces otras cosas.

## Descargar y ejecutar

En Windows 10/11, descarga el paquete ZIP y extráelo para ejecutar la aplicación.

| Paquete más reciente |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Suma de comprobación SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notas de la versión](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Después de extraer el ZIP, mueve la carpeta `UnfocusMute-windows-x64` a la ubicación donde quieras guardar la aplicación y ejecuta `UnfocusMute-v<version>.exe` dentro de esa carpeta.

UnfocusMute es una aplicación independiente que no requiere instalación. Tampoco necesitas instalar Rust, Visual Studio Build Tools, MinGW ni otras herramientas de desarrollo.

> **Nota:** Como los certificados de firma de código tienen un costo, la aplicación se distribuye actualmente sin firma de código de Windows. En el primer inicio puede aparecer un aviso de Windows SmartScreen o de "editor desconocido". Si quieres comprobar la integridad del archivo por tu cuenta, consulta [Transparencia y verificación de archivos de la versión](#transparencia-y-verificación-de-archivos-de-la-versión).

<br>

## Antes de usar

UnfocusMute funciona con los nombres de proceso, la información de la ventana en primer plano y las sesiones CoreAudio que proporciona Windows. Por eso, si un controlador, un ajuste de permisos o una herramienta de seguridad limita el acceso a las sesiones, el control de silencio puede no funcionar correctamente.

Recomendamos mantener UnfocusMute en ejecución en la bandeja del sistema en lugar de cerrarlo. Si una aplicación registrada se cierra mientras está silenciada, su último estado de silencio puede conservarse. Mientras UnfocusMute siga ejecutándose, restaurará el sonido automáticamente cuando vuelvas a abrir la aplicación y la traigas al primer plano. Si también cierras UnfocusMute, es posible que la aplicación se quede sin sonido; en ese caso, reactívalo manualmente desde el `Mezclador de volumen` de Windows.

**Comportamiento al registrar por PID:** Windows no siempre proporciona el mismo PID para una sesión de audio y para la ventana en primer plano. Para compensarlo, UnfocusMute considera que la aplicación ha vuelto al primer plano cuando el nombre del ejecutable (`.exe`) asociado al PID registrado coincide con el nombre del ejecutable de la ventana activa.

Por eso, si hay varias instancias del mismo `.exe` ejecutándose a la vez, no siempre es posible distinguir perfectamente una instancia concreta. En ese caso, el sonido puede restaurarse aunque otra instancia esté en primer plano.

**Compatibilidad con anti-cheat:** UnfocusMute no inyecta código en juegos, no lee memoria del juego, no intercepta la entrada del usuario ni modifica archivos del juego. Solo usa la información de procesos/ventana en primer plano de Windows y los controles de silencio de sesiones CoreAudio, por lo que está diseñado para evitar conflictos con la mayoría de los sistemas anti-cheat, aunque no se puede garantizar compatibilidad con todos.

<br>

## Uso

1. Abre UnfocusMute.
2. Elige el idioma. Recomendamos dejar las opciones con sus valores predeterminados.
3. Inicia el juego o la aplicación que quieres registrar.
4. Selecciona una aplicación de la lista o escribe su nombre `.exe` exacto y, después, haz clic en `Registrar`. Si la aplicación aún no ha creado una sesión de audio, cambia a `Todos los procesos` para ver todos los procesos en ejecución.
5. Si necesitas registrar solo un PID concreto, haz clic en `Vista PID` y elige la entrada correspondiente. Las entradas por PID solo se aplican a la instancia que está en ejecución; si la aplicación se reinicia con otro PID, tendrás que registrarla de nuevo.
6. Haz clic con el botón derecho en una aplicación registrada para editar su nota o usar `Pausar` solo en esa aplicación.
7. Haz clic en el estado `Supervisando` de la parte superior para pausar o reanudar toda la supervisión.
8. Abre `Configuración` para cambiar las opciones, incluido el comportamiento al cerrar la ventana.
9. De forma predeterminada, al cerrar la ventana UnfocusMute sigue ejecutándose en la bandeja. Para salir por completo, haz clic con el botón derecho en el icono de la bandeja y elige `Salir`. Puedes cambiar el comportamiento del botón de cierre en `Configuración`.

<br>

## Notas para aplicaciones registradas

Si el nombre del proceso no basta para recordar de qué aplicación se trata, haz clic con el botón derecho en la aplicación registrada y elige `Editar nota`. La nota aparece encima del nombre del proceso en la lista de aplicaciones registradas y no afecta a la forma en que UnfocusMute identifica la aplicación.

Es útil cuando un mismo lanzador de juegos abre varios procesos, o cuando el nombre del ejecutable no deja claro para qué sirve.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - cliente del servidor de pruebas`

Las notas se guardan localmente junto con el resto de la configuración en `%APPDATA%\UnfocusMute\config.json`.

<br>

## Encontrar el nombre del ejecutable

Si no sabes qué nombre registrar, busca el ejecutable terminado en `.exe` en el Administrador de tareas.

1. Inicia primero la aplicación que quieres registrar.
2. Usa `Alt`+`Tab` o `Windows`+`Tab` para volver al escritorio de Windows.
3. Pulsa `Ctrl`+`Shift`+`Esc` para abrir el Administrador de tareas.
4. Ordena la lista de procesos por `CPU` y busca la aplicación que acabas de iniciar.
5. Haz clic con el botón derecho en ese elemento y abre `Propiedades`.
6. Busca el nombre del ejecutable terminado en `.exe`, como `game.exe`, y regístralo en UnfocusMute.

<br>

## Solución de problemas

Si una aplicación no aparece en la lista, o si el registro por PID no se comporta como esperabas, revisa primero [Antes de usar](#antes-de-usar) y [Encontrar el nombre del ejecutable](#encontrar-el-nombre-del-ejecutable).

Si el indicador de estado de la parte superior cambia a `Requiere atención`, haz clic en `Detalles` para ver el mensaje de error detallado.

Si el problema continúa, abre una incidencia en GitHub.

Si sospechas que el problema es una vulnerabilidad de seguridad, no publiques los detalles en una incidencia pública. Usa el proceso de notificación privada y consulta [SECURITY.md](../SECURITY.md) para más información.

<br>

## Archivo de configuración

Si necesitas revisar directamente el archivo de configuración o hacer una copia de seguridad, haz clic en `Abrir carpeta de configuración` en Configuración. Se abrirá el Explorador de archivos en la carpeta `%APPDATA%\UnfocusMute`, donde se guarda la configuración.

Puedes editar el archivo de configuración directamente, pero si su formato no es válido y no se puede leer, se guarda una copia como `config.invalid-<timestamp>.json`. Si el problema se detecta durante el inicio de la aplicación, la configuración se restaura a los valores predeterminados; si se detecta mientras la aplicación está en ejecución, se crea un nuevo archivo de configuración a partir de los ajustes actuales.

<br>

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
- Posición y tamaño de la ventana

Esta información no se envía a ningún sitio.

Si activas el inicio automático al iniciar sesión en Windows, la ruta del ejecutable actual también se guarda en el valor `UnfocusMute` de `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

### Información que no guarda

UnfocusMute no guarda historial de uso, registros de actividad, registros de errores, datos de audio, títulos de ventanas, pulsaciones de teclas ni ninguna información que no aparezca arriba en "Información que guarda".

### Cómo eliminar la aplicación y sus datos

Para eliminar todos los archivos relacionados con la aplicación, borra la carpeta `UnfocusMute-windows-x64` y después borra `%APPDATA%\UnfocusMute`.

Si alguna vez activaste el inicio automático, borra también el valor `UnfocusMute` en `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

<br>

## Transparencia y verificación de archivos de la versión

UnfocusMute está diseñado para usarse de forma segura en entornos habituales, por lo que la mayoría de los usuarios no necesitan realizar los pasos de verificación que se indican a continuación. Si prefieres no confiar únicamente en el desarrollador o das especial importancia a la seguridad de la cadena de suministro de software, puedes usar este procedimiento público para verificar el origen y la integridad de los archivos descargados.

### Por qué se necesita una verificación independiente

Aunque revises el código fuente del repositorio y determines que es seguro, no puedes dar por hecho que los archivos de una versión publicada en GitHub se crearon realmente a partir de ese código. Si se compromete la cuenta del desarrollador o se abusa de los permisos de publicación, podrían distribuirse archivos ajenos al código fuente publicado.

La comparación de hashes SHA-256 permite confirmar que un archivo descargado coincide con la suma de comprobación publicada, pero no demuestra qué código fuente y entorno de compilación lo generaron.

Para abordar de forma transparente estos riesgos de la cadena de suministro, UnfocusMute publica un método que permite verificar directamente que un archivo de una versión publicada en GitHub es un artefacto oficial generado por GitHub Actions a partir del commit al que apunta la etiqueta de esa versión.

<details>
<summary>Mostrar los pasos de verificación</summary>

Primero instala [GitHub CLI](https://cli.github.com/), indica la versión que descargaste y ejecuta el comando en PowerShell.

```powershell
$version = Read-Host "Introduce la versión publicada (por ejemplo, v1.3.5)"
$sourceRef = "refs/tags/$version"
$workflow = "ilsd7/UnfocusMute/.github/workflows/release.yml"

gh attestation verify .\UnfocusMute-windows-x64.zip `
  -R ilsd7/UnfocusMute `
  --source-ref $sourceRef `
  --signer-workflow $workflow
```

Este comando se conecta al servicio de atestación de GitHub y comprueba que el SHA-256 del ZIP local coincide con el valor registrado en la procedencia de compilación firmada por GitHub Actions.

También puedes comparar el ZIP con el hash SHA-256 publicado en la versión.

```powershell
$expectedHash = ((Get-Content .\UnfocusMute-windows-x64.zip.sha256 -TotalCount 1) -split '\s+')[0]
$actualHash = (Get-FileHash .\UnfocusMute-windows-x64.zip -Algorithm SHA256).Hash

if ($actualHash -ne $expectedHash) {
  throw "La verificación SHA-256 ha fallado."
}

"SHA-256 verificado: $actualHash"
```

Si la verificación se completa correctamente, puedes confirmar que el ZIP descargado fue generado para la etiqueta especificada por el flujo de trabajo de GitHub Actions indicado y que coincide con el hash registrado en su atestación.

Esto no demuestra que el código fuente sea seguro, que todo el entorno de GitHub esté intacto ni que la compilación pueda reproducirse byte por byte en otro equipo.

</details>

<br>

## Compilar desde el código fuente

El objetivo recomendado para las versiones publicadas es `x86_64-pc-windows-msvc`.

Requisitos:

- Rust stable
- Visual Studio Build Tools 2022 o Visual Studio 2022
- Windows 10/11 SDK

<details>
<summary>Mostrar comandos de compilación y empaquetado</summary>

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

</details>

<br>

## Licencia

Apache License 2.0. Consulta [LICENSE](../LICENSE) para más detalles.

Consulta [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para los avisos de licencias de crates de Rust de terceros.
