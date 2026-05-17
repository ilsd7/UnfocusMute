# UnfocusMute

[한국어](../README.md) | [English](../README_en.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | Español | [Français](README.fr.md) | [Português](README.pt.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

UnfocusMute es una pequeña app de bandeja para Windows, ligera y portable, que silencia automáticamente juegos o apps seleccionadas cuando quedan en segundo plano.

El ejecutable actual para Windows ocupa unos 676 KB, menos de 1 MB, así que se descarga rápido y mantiene la ligereza propia de una herramienta portable.

Está hecha con Rust y solo controla las sesiones de audio que registras. Cuando una app vuelve al primer plano, UnfocusMute restaura únicamente las sesiones que silenció por sí misma, así que no cambia los silencios que hayas aplicado manualmente.

Resulta especialmente útil cuando sales de un juego con Alt+Tab para usar un navegador, chat o ventana de trabajo. Puedes mantener callada una app en segundo plano sin abrir una y otra vez el mezclador de volumen de Windows.

## Ventajas Clave

- Permite registrar objetivos por `.exe` o por PID individual.
- Agrupa procesos del mismo ejecutable cuando quieres tratarlos como una sola app.
- Permite registrar una instancia concreta con `Ver PID` cuando hace falta.
- Restaura solo las sesiones silenciadas por UnfocusMute y respeta los silencios manuales.
- Funciona como app portable desde un ZIP, sin instalador.

## Útil Para

- Cambiar con Alt+Tab desde juegos o apps a otras ventanas.
- Juegos que necesitan silencio al quedar en segundo plano.
- Juegos que no tienen una opción propia para silenciarse en segundo plano.
- Silenciar audio de un juego en segundo plano sin apagar navegador o app de llamada.
- Gestionar una app por `.exe` o controlar solo un PID específico.

## Funciones

- Silencia automáticamente solo las apps registradas que no están en primer plano.
- Restaura el audio solo para sesiones silenciadas por UnfocusMute.
- Añade objetivos desde la lista de apps en ejecución o escribiendo un ejecutable como `game.exe`.
- Soporta registro agrupado por `.exe` y registro por PID.
- Funcionamiento en bandeja, pausa, acceso a la carpeta de configuración y protección contra doble ejecución.
- Selección de idioma en el primer inicio y cambio entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी y العربية.
- Configuración local en `%APPDATA%\UnfocusMute\config.json`.

## Por Qué Rust

UnfocusMute es una utilidad pequeña que permanece en segundo plano, así que importan el arranque rápido, el uso de memoria y una distribución sencilla. El ejecutable nativo en Rust no necesita runtime aparte y se integra directamente con las API Windows CoreAudio sin cargar frameworks residentes innecesarios.

## Seguridad y Privacidad

UnfocusMute funciona con un enfoque local. Solo guarda nombres de procesos registrados, PID opcionales, idioma de la UI, posición de ventana y preferencias de inicio en un archivo de configuración local.

La detección de sesiones de audio y el control de silencio se procesan en tu PC con las API Windows CoreAudio. No incluye red, cuentas, telemetría, analíticas, reportes de fallos, logging remoto ni archivos de log separados.

## Descargar y Ejecutar

Descarga el ZIP para Windows 10/11, extráelo y ejecuta `UnfocusMute.exe`. Es portable, así que no hay instalador y no necesitas un runtime aparte, Rust, Visual Studio Build Tools ni MinGW.

## Uso

1. Inicia UnfocusMute.
2. Selecciona un idioma en el primer inicio. English está seleccionado por defecto.
3. Abre el juego o app que quieras gestionar.
4. Actualiza la lista de apps en ejecución, selecciona un elemento y pulsa `Añadir selección`.
5. Los procesos con el mismo `.exe` se agrupan por defecto.
6. Usa `Ver PID` solo cuando necesites registrar una instancia concreta.
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

Crear un ZIP de distribución:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

El paquete se crea en `dist\UnfocusMute-<version>-windows-x64.zip` e incluye el ejecutable, `LICENSE`, `THIRD_PARTY_NOTICES.md`, `README_ko.md` y `README_en.md` en la raíz, además del resto de documentación localizada en `docs`.

## Licencia

Apache License 2.0. Consulta [LICENSE](../LICENSE).

Consulta [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para los avisos de licencia de crates Rust de terceros.
