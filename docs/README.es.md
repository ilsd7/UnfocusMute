# UnfocusMute

[한국어](../README.md) | [English](README.en.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | Español | [Français](README.fr.md) | [Português](README.pt.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

UnfocusMute es una pequeña app de bandeja para Windows, ligera y portable, que silencia automáticamente juegos o apps seleccionadas cuando quedan en segundo plano.

Está hecha con Rust y solo controla las sesiones de audio que registras. Cuando una app vuelve al primer plano, UnfocusMute restaura únicamente las sesiones que silenció por sí misma, así que no cambia los silencios que hayas aplicado manualmente.

## Útil Para

- Cambiar entre un juego, navegador, chat o ventana de trabajo
- Mantener callada una app en segundo plano sin abrir el mezclador de volumen de Windows
- Gestionar apps tipo navegador que usan varios procesos con el mismo `.exe`
- Usar una herramienta ligera hecha con Rust que se ejecuta desde un ZIP, sin instalador

## Funciones

- Silencia automáticamente solo las apps registradas que no están en primer plano
- Restaura el audio solo para sesiones silenciadas por UnfocusMute
- Añade objetivos desde la lista de apps en ejecución o escribiendo un ejecutable como `game.exe`
- Agrupa por defecto procesos `.exe` duplicados, útil para navegadores con muchos PID
- Permite registrar un PID específico con `Ver PID`
- Funcionamiento en bandeja, pausa, acceso al archivo de configuración y protección contra doble ejecución
- Selección de idioma en el primer inicio y cambio entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी y العربية
- Configuración local en `%APPDATA%\UnfocusMute\config.json`
- Sin red, cuentas, telemetría ni logs separados de la app

## Descargar y Ejecutar

Descarga el ZIP para Windows, extráelo y ejecuta `UnfocusMute.exe`. Es portable, así que no hay instalador y no necesitas un runtime aparte, Rust, Visual Studio Build Tools ni MinGW.

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

El paquete se crea en `dist\UnfocusMute-<version>-windows-x64.zip` e incluye el ejecutable y `LICENSE`.

## Privacidad

UnfocusMute guarda solo nombres de procesos registrados, PID opcionales, idioma de la UI, posición de ventana y preferencias de inicio en un archivo de configuración local. La detección de sesiones de audio y el control de silencio se procesan localmente con las API Windows CoreAudio.

## Licencia

Apache License 2.0. Consulta [LICENSE](../LICENSE).
