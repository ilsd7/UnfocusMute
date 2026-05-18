# UnfocusMute

[한국어](../README.md) | [English](../README_en.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | Español | [Français](README.fr.md) | [Português](README.pt.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

<p align="center">
  <img src="../assets/screenshot.png" alt="Ventana de UnfocusMute" width="760">
</p>

UnfocusMute es una app ligera y portátil para la bandeja de Windows que silencia solo el audio de los juegos o aplicaciones que elijas cuando pasan a segundo plano.

Al estar compilada como app nativa en Rust, se ejecuta sin un runtime adicional. El ejecutable actual para Windows ocupa unos 533 KB, menos de 1 MB.

Registra las apps que quieres gestionar, y UnfocusMute solo silencia sus sesiones de audio mientras no están en primer plano. Cuando una app vuelve al primer plano, UnfocusMute restaura únicamente las sesiones que silenció por sí misma, sin cambiar los silencios que hayas aplicado manualmente.

Resulta especialmente útil cuando sales de un juego con Alt+Tab para usar un navegador, chat o ventana de trabajo. Puedes controlar el audio en segundo plano sin abrir una y otra vez el mezclador de volumen de Windows.

## Ventajas Clave

- Automatiza el silencio en segundo plano por juego o app, para que el audio siga los cambios de foco sin ajustes manuales.
- Restaura solo el audio que cambió UnfocusMute y respeta los silencios manuales.
- Funciona como herramienta ligera, portátil y local, sin instalación, cuenta ni conexión de red.

## Útil Para

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

## Compatibilidad con Anticheat

UnfocusMute no inyecta código en juegos, no lee memoria del juego, no intercepta la entrada y no modifica archivos del juego. Como solo usa información de procesos/ventana en primer plano de Windows y controles de silencio de sesiones CoreAudio, se espera que no cause problemas con la mayoría de sistemas anticheat, pero no se puede garantizar la compatibilidad con todos.

## Por Qué Rust

UnfocusMute es una utilidad pequeña que permanece en segundo plano, así que importan el arranque rápido, el uso de memoria y una distribución sencilla. El ejecutable nativo en Rust no necesita runtime aparte y se integra directamente con las API Windows CoreAudio sin cargar frameworks residentes innecesarios.

## Seguridad y Privacidad

UnfocusMute funciona con un enfoque local. Solo guarda nombres de procesos registrados, PID opcionales, idioma de la UI, posición de ventana y preferencias de inicio en un archivo de configuración local.

La detección de sesiones de audio y el control de silencio se procesan en tu PC con las API Windows CoreAudio. No incluye red, cuentas, telemetría, analíticas, reportes de fallos ni logging remoto, y tampoco crea archivos de log de la app.

## Descargar y Ejecutar

Descarga el ZIP para Windows 10/11, extráelo y ejecuta `UnfocusMute.exe`. Es portable, así que no hay instalador y no necesitas un runtime aparte, Rust, Visual Studio Build Tools ni MinGW.

## Uso

1. Inicia UnfocusMute.
2. Selecciona un idioma en el primer inicio. English está seleccionado por defecto.
3. Abre el juego o app que quieras gestionar.
4. Actualiza la lista de apps en ejecución, selecciona un elemento y pulsa `Añadir selección`.
5. Los procesos con el mismo `.exe` se agrupan por defecto.
6. Usa `Ver PID` solo cuando necesites registrar una instancia concreta. Un objetivo PID solo se aplica a la instancia que está en ejecución; si la app se reinicia con otro PID, selecciónalo de nuevo.
7. Al cerrar la ventana, UnfocusMute sigue en la bandeja. Usa `Salir` para cerrarla por completo.

## Valores Predeterminados

En el primer inicio puedes elegir si UnfocusMute se inicia automáticamente al entrar en Windows. La configuración nueva deja desactivado el inicio automático y activa iniciar minimizado en bandeja y quitar silencio al salir.

Pulsa `Idioma` en la app para cambiar inmediatamente entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी y العربية. La selección se guarda automáticamente.

## Compilación para Desarrolladores

El objetivo recomendado para publicar es `x86_64-pc-windows-msvc`.

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

Las builds de release están configuradas para reducir el tamaño de salida. El release profile de `Cargo.toml` elimina símbolos, activa LTO, usa una sola codegen unit, define `panic = "abort"` y optimiza para tamaño.

Crear un ZIP de distribución:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

El paquete se crea en `dist\UnfocusMute-<version>-windows-x64.zip` e incluye el ejecutable, `LICENSE`, `THIRD_PARTY_NOTICES.md`, `README_ko.md` y `README_en.md` en la raíz, además del resto de documentación localizada en `docs`.

## Licencia

Apache License 2.0. Consulta [LICENSE](../LICENSE).

Consulta [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para los avisos de licencia de crates Rust de terceros.
