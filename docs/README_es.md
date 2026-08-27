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
- El ejecutable ocupa unos 600 KB.
- No se limita a juegos: también puedes registrar aplicaciones comunes como navegadores, aplicaciones de mensajería, lanzadores y reproductores multimedia.
- El silenciamiento y la restauración solo se aplican a las sesiones que UnfocusMute cambió directamente; las sesiones que tú ya habías silenciado no se modifican.

---

<p align="center">
  <img src="../assets/screenshot_es.png" width="600" alt="Ventana principal de UnfocusMute">
</p>

---

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

Después de extraer el ZIP, mueve la carpeta `UnfocusMute-windows-x64` a la ubicación que prefieras y ejecuta `UnfocusMute-v<version>.exe`. No requiere instalación, un entorno de ejecución adicional ni herramientas de desarrollo.

> **Nota:** Por motivos de costo, UnfocusMute se distribuye sin firma de código de Windows. Por eso, la primera vez que lo ejecutes puede aparecer una advertencia de SmartScreen o de "editor desconocido". Para comprobar por tu cuenta el origen y la integridad del archivo, consulta [Transparencia y verificación de archivos de la versión](#transparencia-y-verificación-de-archivos-de-la-versión).

<br>

## Uso

1. Abre UnfocusMute. La primera vez, elige el idioma y las opciones de inicio, y haz clic en `Comenzar`.
2. Abre el juego o la aplicación que quieras registrar y reproduce algún sonido para que aparezca en la lista.
3. Vuelve a UnfocusMute, selecciona la aplicación y haz clic en `Registrar`. Si la registras por el nombre del `.exe`, se administrarán todas sus sesiones de audio y seguirá funcionando aunque reinicies la aplicación.
4. Cambia a otra ventana con `Alt`+`Tab` y vuelve. La aplicación registrada se silenciará mientras esté en segundo plano y recuperará el sonido al volver al primer plano.

Eso es todo. La supervisión comienza en cuanto registras la aplicación y, con la configuración predeterminada, UnfocusMute sigue funcionando en la bandeja aunque cierres la ventana.

> **¿La aplicación no aparece en la lista?** Haz clic en `Todos los procesos` o escribe directamente el nombre exacto del `.exe`. Usa `Vista PID` si solo quieres registrar un PID concreto que esté en ejecución. Como el PID cambia al reiniciar la aplicación, en la mayoría de los casos conviene registrarla por el nombre del `.exe`.

### Encontrar el nombre del ejecutable

Si no sabes el nombre de la aplicación que quieres registrar, consulta el nombre exacto de su `.exe` en el Administrador de tareas.

1. Inicia primero la aplicación que quieres registrar.
2. Si se está ejecutando a pantalla completa, cambia de la pantalla del juego a otra ventana con `Alt`+`Tab` o `Windows`+`Tab`.
3. Pulsa `Ctrl`+`Shift`+`Esc` para abrir el Administrador de tareas.
4. En la lista de procesos que aparece al abrirlo, haz clic en la columna `CPU` para ordenar de mayor a menor uso.
5. Busca cerca de la parte superior la aplicación que acabas de abrir, haz clic con el botón derecho y selecciona `Propiedades`.
6. Comprueba el nombre del ejecutable terminado en `.exe`, como `game.exe`, y regístralo en UnfocusMute.

### Acciones habituales

- Haz clic con el botón derecho en una aplicación registrada para pausarla o reanudarla, editar su nota o quitarla del registro.
- Haz clic en el estado `Supervisando` de la parte superior para pausar o reanudar toda la supervisión.
- En `Configuración` puedes cambiar el inicio automático y lo que ocurre al cerrar la ventana.
- Para salir por completo, haz clic con el botón derecho en el icono de la bandeja y selecciona `Salir`.

<br>

## Añadir notas cuando el nombre de una aplicación no es claro

Haz clic con el botón derecho en una aplicación registrada y selecciona `Editar nota` para añadir una descripción fácil de reconocer encima del nombre del proceso. La nota solo sirve para distinguir la aplicación y no afecta a la forma en que UnfocusMute decide qué debe silenciar.

- `htgame.exe` → `NTE`
- `game.exe (PID 21976)` → `cliente del servidor de pruebas`

Las notas se guardan con el resto de la configuración en `%APPDATA%\UnfocusMute\config.json`.

<br>

## Comportamientos importantes

**Ejecución en la bandeja y restauración del sonido:** Si una aplicación registrada se cierra mientras está silenciada, Windows puede recordar ese estado. Si UnfocusMute sigue ejecutándose en la bandeja, restaurará el sonido automáticamente cuando vuelvas a abrir la aplicación y la traigas al primer plano. Si también cerraste UnfocusMute y la aplicación sigue sin sonido, quita el silencio manualmente desde el `Mezclador de volumen` de Windows.

**Registro por PID:** Windows puede asignar PID distintos a la sesión de audio y a la ventana en primer plano. Por eso, aunque registres una aplicación por PID, UnfocusMute considera que ha vuelto cuando una ventana con el mismo nombre de `.exe` pasa al primer plano y restaura su sonido. Esto no resulta adecuado si quieres seguir usando una ventana del mismo `.exe` mientras mantienes silenciado un PID concreto. En cambio, puede ser útil para silenciar solo uno de varios PID del mismo `.exe` mientras trabajas en otra aplicación y mantener el sonido de los demás.

**Compatibilidad con sistemas anti-cheat:** UnfocusMute no inyecta código en los juegos, no lee su memoria, no intercepta la entrada ni modifica sus archivos. Solo usa la información de procesos y ventanas en primer plano de Windows y los controles de silencio de CoreAudio. Está diseñado para evitar conflictos con la mayoría de los sistemas anti-cheat, aunque no se puede garantizar la compatibilidad con todos.

<br>

## Solución de problemas

Comprueba primero lo siguiente:

- **La aplicación no aparece en la lista:** Reproduce algún sonido en la aplicación y vuelve a abrir la lista. Si sigue sin aparecer, haz clic en `Todos los procesos` o [busca directamente el nombre del ejecutable](#encontrar-el-nombre-del-ejecutable).
- **No se silencia:** Comprueba que el estado superior sea `Supervisando` y que la aplicación registrada no esté `En pausa`. También puede fallar si un controlador, la configuración de permisos o un programa de seguridad limita el acceso a las sesiones de audio de Windows.
- **El sonido no se restaura:** Vuelve a traer la aplicación al primer plano. Si ya cerraste UnfocusMute, quita el silencio manualmente desde el `Mezclador de volumen` de Windows.
- **El registro por PID no funciona como esperabas:** Consulta el [comportamiento del registro por PID](#comportamientos-importantes).
- **El estado cambia a `Requiere atención`:** Haz clic en `Detalles`, junto al indicador de estado, para ver el error.

Si el problema continúa, [abre una incidencia en GitHub](https://github.com/ilsd7/UnfocusMute/issues/new/choose).

Si sospechas que el problema es una vulnerabilidad de seguridad, no publiques los detalles en una incidencia pública. Usa el proceso de notificación privada y consulta [SECURITY.md](../SECURITY.md) para más información.

<br>

## Archivo de configuración

Si necesitas revisar directamente el archivo de configuración o hacer una copia de seguridad, haz clic en `Abrir carpeta de configuración` en Configuración. Se abrirá el Explorador de archivos en la carpeta `%APPDATA%\UnfocusMute`, donde se guarda la configuración.

También puedes editar `config.json` directamente. Si el formato no es válido y no se puede leer, UnfocusMute guarda una copia del archivo original como `config.invalid-<timestamp>.json` y crea uno nuevo a partir de los valores predeterminados o de la configuración actual de la aplicación.

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

### Eliminar UnfocusMute por completo

1. Si activaste `Ejecutar al iniciar sesión en Windows`, desactívalo primero desde `Configuración`.
2. Haz clic con el botón derecho en el icono de la bandeja y selecciona `Salir`.
3. Elimina la carpeta `UnfocusMute-windows-x64` y la carpeta `%APPDATA%\UnfocusMute`.

Si ya eliminaste el ejecutable y no puedes desactivar el inicio automático, elimina el valor `UnfocusMute` de `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

<br>

## Transparencia y verificación de archivos de la versión

UnfocusMute está diseñado para usarse de forma segura en entornos habituales, por lo que la mayoría de los usuarios no necesitan realizar los pasos de verificación que se indican a continuación. Si no quieres depender únicamente de la confianza en el desarrollador o das especial importancia a la seguridad de la cadena de suministro de software, puedes usar este procedimiento público para verificar el origen y la integridad de los archivos descargados.

### Por qué se necesita una verificación independiente

Aunque revises el código fuente del repositorio y determines que es seguro, no puedes dar por hecho que los archivos de una versión publicada en GitHub se crearon realmente a partir de ese código. Si se compromete la cuenta del desarrollador o se abusa de los permisos de publicación, podrían distribuirse archivos ajenos al código fuente publicado.

La comparación de hashes SHA-256 permite confirmar que un archivo descargado coincide con la suma de comprobación publicada, pero no demuestra qué código fuente y entorno de compilación lo generaron.

Para abordar de forma transparente estos riesgos de la cadena de suministro, UnfocusMute publica un método que permite verificar directamente que un archivo de una versión publicada en GitHub es un artefacto oficial generado por GitHub Actions a partir del commit al que apunta la etiqueta de esa versión.

<details>
<summary>Mostrar los pasos de verificación</summary>

Primero instala [GitHub CLI](https://cli.github.com/). Después, cambia el valor de `$version` que aparece abajo por la etiqueta de la versión que quieras verificar y ejecuta todo el bloque de comandos en PowerShell.

```powershell
$version = "v1.5.0"
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

Si la verificación se completa correctamente, puedes confirmar que el ZIP descargado fue generado por el flujo de trabajo de GitHub Actions indicado para la etiqueta de versión especificada y que coincide con el hash registrado en su atestación.

Esto no demuestra que el código fuente sea seguro, que todo el entorno de GitHub esté intacto ni que la compilación pueda reproducirse byte por byte en otro equipo.

</details>

<br>

## Compilar desde el código fuente

El objetivo recomendado para las versiones publicadas es `x86_64-pc-windows-msvc`.

Herramientas necesarias:

- Rust stable
- Visual Studio Build Tools 2022 o Visual Studio 2022
- Windows 10/11 SDK

También necesitas `cargo-about` si quieres actualizar `THIRD_PARTY_NOTICES.md`.

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

Cuando necesites actualizar los avisos de licencias de terceros:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

El resultado se crea en `dist\UnfocusMute-windows-x64.zip`, junto con el archivo de verificación SHA-256 `dist\UnfocusMute-windows-x64.zip.sha256`. El ZIP incluye el ejecutable con versión (`UnfocusMute-v<version>.exe`), `LICENSE` y `THIRD_PARTY_NOTICES.md`.

</details>

<br>

## Licencia

Apache License 2.0. Consulta [LICENSE](../LICENSE) para más detalles.

Consulta [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para los avisos de licencias de crates de Rust de terceros.
