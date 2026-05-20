<div align="center">
  <img src="../assets/app-icon.png" alt="Icono de UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>App ligera y portátil para la bandeja de Windows que silencia apps en segundo plano y restaura solo el audio que cambió.</strong></p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases">Descargar</a>
    · <a href="#uso">Uso</a>
    · <a href="#seguridad-y-privacidad">Privacidad</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>

  <p>
    <code>Windows 10/11</code>
    <code>Portable</code>
    <code>Rust Native</code>
    <code>~477KB</code>
    <code>No telemetry</code>
  </p>

  <p>
    <a href="../README.md">한국어</a> · <a href="../README_en.md">English</a> · <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">简体中文</a> · Español · <a href="README.fr.md">Français</a> · <a href="README.pt.md">Português</a> · <a href="README.hi.md">हिन्दी</a> · <a href="README.ar.md">العربية</a>
  </p>
</div>

UnfocusMute es una app ligera y portátil para la bandeja de Windows que silencia solo el audio de los juegos o aplicaciones que elijas cuando pasan a segundo plano.

Al estar compilada como app nativa en Rust, se ejecuta sin un runtime adicional. El ejecutable actual para Windows ocupa unos 477 KB, menos de 1 MB.

<p align="center">
  <img src="../assets/screenshot.png" alt="Ventana de UnfocusMute" width="760" loading="lazy" decoding="async">
</p>

Registra las apps que quieres gestionar, y UnfocusMute solo silencia sus sesiones de audio mientras no están en primer plano. Cuando una app vuelve al primer plano, UnfocusMute restaura únicamente las sesiones que silenció por sí misma, sin cambiar los silencios que hayas aplicado manualmente.

Resulta especialmente útil cuando sales de un juego con Alt+Tab para usar un navegador, chat o ventana de trabajo. Puedes controlar el audio en segundo plano sin abrir una y otra vez el mezclador de volumen de Windows.

---

## Descargar y ejecutar

Descarga el ZIP para Windows 10/11 desde [GitHub Releases](https://github.com/ilsd7/UnfocusMute/releases), extráelo y ejecuta `UnfocusMute.exe`. Es portable, así que no hay instalador y no necesitas un runtime aparte, Rust, Visual Studio Build Tools ni MinGW.

## Uso

1. Inicia UnfocusMute.
2. Selecciona un idioma en el primer inicio. English está seleccionado por defecto.
3. Abre el juego o app que quieras gestionar.
4. Actualiza la lista de apps en ejecución, selecciona un elemento y pulsa `Añadir selección`.
5. Usa `Ver PID` solo cuando necesites registrar una instancia concreta. Los objetivos registrados por PID solo se aplican a la instancia que está en ejecución; si la app se reinicia con otro PID, selecciónalo de nuevo.
6. Al cerrar la ventana, UnfocusMute sigue en la bandeja. Usa `Salir` para cerrarla por completo.

## Encontrar el nombre del ejecutable del juego

Si no sabes qué nombre escribir, revisa en el Administrador de tareas el nombre del ejecutable que termina en `.exe`.

1. Abre primero el juego.
2. Usa `Alt`+`Tab` o `Windows`+`Tab` para salir de la pantalla del juego y volver a Windows.
3. Pulsa `Ctrl`+`Shift`+`Esc` para abrir el Administrador de tareas.
4. Ordena la lista de procesos por `CPU` para encontrar el juego que acabas de abrir.
5. Haz clic derecho sobre el juego y abre `Propiedades`.
6. Busca el nombre del ejecutable que termina en `.exe`, como `game.exe`, y añádelo a UnfocusMute.

---

## Ventajas

- Automatiza el silencio en segundo plano por juego o app, para que el audio siga los cambios de foco sin ajustes manuales.
- Restaura solo el audio que cambió UnfocusMute y respeta los silencios manuales.
- Funciona como herramienta ligera, portátil y local, sin instalación, cuenta ni conexión de red.

## Útil para

- Cambiar con Alt+Tab desde juegos o apps a otras ventanas.
- Juegos que no tienen una opción propia para silenciarse en segundo plano.
- Silenciar el audio de un juego en segundo plano sin apagar el navegador o la app de llamada.
- Alternar entre gestionar todo un `.exe` y controlar un PID concreto cuando una app abre varios procesos.

## Funciones

- Silencia automáticamente las sesiones de audio de las apps registradas mientras están en segundo plano y las restaura al volver al primer plano.
- Permite añadir objetivos desde la lista de apps en ejecución o escribiendo un ejecutable como `game.exe`.
- Admite objetivos por `.exe`, objetivos por PID de la instancia actual y `Ver PID`.
- Permanece en la bandeja, permite pausar, abrir la carpeta de configuración y evita dobles ejecuciones.
- Permite elegir idioma en el primer inicio y cambiar luego entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी y العربية.
- Configuración local en `%APPDATA%\UnfocusMute\config.json`.

---

## Antes de usar

UnfocusMute no inyecta código en juegos, no lee memoria del juego, no intercepta la entrada y no modifica archivos del juego. Como solo usa información de procesos/ventana en primer plano de Windows y controles de silencio de sesiones CoreAudio, se espera que no cause problemas con la mayoría de sistemas anticheat, pero no se puede garantizar la compatibilidad con todos.

## Seguridad y privacidad

UnfocusMute funciona con un enfoque local. Solo guarda nombres de procesos registrados, PID opcionales, idioma de la UI, posición de ventana y preferencias de inicio en un archivo de configuración local.

La detección de sesiones de audio y el control de silencio se procesan en tu PC con las API Windows CoreAudio. No incluye red, cuentas, telemetría, analíticas, reportes de fallos ni logging remoto, y tampoco crea archivos de log de la app.

## Configuración inicial

En el primer inicio puedes elegir si UnfocusMute se inicia automáticamente al entrar en Windows. La configuración nueva deja desactivado el inicio automático y activa iniciar minimizado en bandeja y quitar silencio al salir.

Pulsa `Idioma` en la app para cambiar inmediatamente entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी y العربية. La selección se guarda automáticamente.

Usa `Abrir carpeta de configuración` en la app si necesitas revisar el archivo de configuración directamente o gestionar copias de seguridad.

---

## Compilar desde el código fuente

El objetivo recomendado para publicar es `x86_64-pc-windows-msvc`.

Requisitos:

- Rust stable
- Visual Studio Build Tools 2022 o Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

Las builds de release están configuradas para reducir el tamaño de salida. El release profile de `Cargo.toml` elimina símbolos, activa LTO, usa una sola codegen unit, define `panic = "abort"` y optimiza para tamaño.

Ejecutable:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

Crear un ZIP de distribución:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Actualizar los avisos de licencia de terceros:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

El paquete se crea en `dist\UnfocusMute-<version>-windows-x64.zip`, junto con un archivo de verificación SHA-256 en `dist\UnfocusMute-<version>-windows-x64.zip.sha256`. El ZIP incluye el ejecutable, `LICENSE`, `THIRD_PARTY_NOTICES.md`, `README_ko.md` y `README_en.md` en la raíz, además del resto de documentación localizada en `docs`.

---

## Licencia

Apache License 2.0. Consulta [LICENSE](../LICENSE).

Consulta [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para los avisos de licencia de crates Rust de terceros.
