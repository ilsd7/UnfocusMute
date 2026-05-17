# UnfocusMute

[한국어](../README.md) | [English](README.en.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Français](README.fr.md) | Português | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

UnfocusMute é um pequeno app leve para a bandeja do Windows que silencia automaticamente jogos ou apps selecionados quando eles ficam em segundo plano.

Criado em Rust e distribuído como app portátil, ele controla apenas as sessões de áudio que você registra. Quando um app volta ao primeiro plano, o UnfocusMute restaura somente as sessões que ele mesmo silenciou, sem alterar silenciamentos feitos manualmente.

## Útil Para

- Alternar entre um jogo, navegador, chat ou janela de trabalho
- Manter um app em segundo plano quieto sem abrir o mixer de volume do Windows
- Gerenciar apps parecidos com navegadores que usam vários processos com o mesmo `.exe`
- Usar uma ferramenta leve feita em Rust que roda direto de um ZIP, sem instalador

## Recursos

- Silencia automaticamente apenas apps registrados que não estão em foco
- Restaura o áudio somente das sessões silenciadas pelo UnfocusMute
- Adiciona alvos pela lista de apps em execução ou digitando um executável como `game.exe`
- Agrupa processos `.exe` duplicados por padrão, útil para navegadores com muitos PIDs
- Permite registrar por PID com `Mostrar PIDs`
- Funciona na bandeja, com pausa, atalho para o arquivo de configuração e proteção contra múltiplas instâncias
- Seleção de idioma no primeiro uso e troca imediata entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी e العربية
- Configuração local em `%APPDATA%\UnfocusMute\config.json`
- Sem rede, contas, telemetria ou logs separados do app

## Baixar e Executar

Baixe o ZIP para Windows, extraia e execute `UnfocusMute.exe`. O app é portátil, então não há instalador e você não precisa de runtime separado, Rust, Visual Studio Build Tools nem MinGW.

## Uso

1. Abra o UnfocusMute.
2. Selecione um idioma no primeiro uso. English vem selecionado por padrão.
3. Inicie o jogo ou app que você quer gerenciar.
4. Atualize a lista de apps em execução, selecione um item e clique em `Adicionar seleção`.
5. Entradas com o mesmo `.exe` são agrupadas por padrão.
6. Use `Mostrar PIDs` somente quando precisar registrar uma instância específica.
7. Fechar a janela mantém o UnfocusMute rodando na bandeja. Use `Sair` para encerrar completamente.

## Padrões

No primeiro uso, você pode escolher se o UnfocusMute inicia automaticamente ao entrar no Windows. Em novas configurações, o início automático fica desativado, enquanto iniciar minimizado na bandeja e desmutar apps ao sair ficam ativados.

Clique em `Idioma` no app para trocar imediatamente entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी e العربية. A escolha é salva automaticamente.

## Build para Desenvolvedores

O alvo recomendado para release é `x86_64-pc-windows-msvc`.

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

Criar um ZIP de distribuição:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

O pacote é criado em `dist\UnfocusMute-<version>-windows-x64.zip` e inclui o executável e `LICENSE`.

## Privacidade

UnfocusMute salva apenas nomes de processos registrados, PIDs opcionais, idioma da interface, posição da janela e preferências de inicialização em um arquivo de configuração local. A detecção de sessões de áudio e o controle de mudo são processados localmente pelas APIs Windows CoreAudio.

## Licença

Apache License 2.0. Consulte [LICENSE](../LICENSE).
