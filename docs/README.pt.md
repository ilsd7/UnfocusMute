# UnfocusMute

[한국어](../README.md) | [English](../README_en.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Français](README.fr.md) | Português | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

UnfocusMute é um pequeno app leve para a bandeja do Windows que silencia automaticamente jogos ou apps selecionados quando eles ficam em segundo plano.

Criado como app nativo em Rust, ele roda sem runtime separado. O executável atual para Windows tem cerca de 676 KB, menos de 1 MB.

Registre os apps que você quer gerenciar, e o UnfocusMute silencia apenas as sessões de áudio deles enquanto não estão em foco. Quando um app volta ao primeiro plano, o UnfocusMute restaura somente as sessões que ele mesmo silenciou, sem alterar silenciamentos feitos manualmente.

Ele é especialmente útil quando você sai de um jogo com Alt+Tab para usar navegador, chat ou uma janela de trabalho. Dá para controlar o áudio em segundo plano sem abrir o mixer de volume do Windows repetidamente.

## Principais Vantagens

- Automatiza o mudo em segundo plano por jogo ou app, fazendo o áudio acompanhar as mudanças de foco sem ajustes manuais.
- Restaura apenas o áudio que o UnfocusMute alterou, preservando silenciamentos manuais.
- Funciona como ferramenta leve, portátil e local, sem instalação, conta ou conexão de rede.

## Útil Para

- Alternar de jogos ou apps para outras janelas com Alt+Tab.
- Jogos que não oferecem opção própria de silenciar ao ficar em segundo plano.
- Silenciar o áudio de um jogo em segundo plano sem afetar navegador ou app de chamada.
- Alternar entre gerenciar um `.exe` inteiro e controlar um PID específico quando um app abre vários processos.

## Recursos

- Detecção de foco com mudo e restauração automáticos para sessões registradas.
- Adiciona alvos pela lista de apps em execução ou digitando um executável como `game.exe`.
- Registro agrupado por `.exe`, registro por PID e `Mostrar PIDs`.
- Funciona na bandeja, com pausa, atalho para a pasta de configuração e proteção contra múltiplas instâncias.
- Seleção de idioma no primeiro uso e troca imediata entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी e العربية.
- Configuração local em `%APPDATA%\UnfocusMute\config.json`.

## Por Que Rust

UnfocusMute é uma pequena ferramenta que fica em segundo plano, então inicialização rápida, baixo uso de memória e distribuição simples são importantes. O executável nativo em Rust roda sem runtime separado e conversa diretamente com as APIs Windows CoreAudio sem carregar frameworks residentes desnecessários.

## Segurança e Privacidade

UnfocusMute funciona com uma abordagem local. Ele salva apenas nomes de processos registrados, PIDs opcionais, idioma da interface, posição da janela e preferências de inicialização em um arquivo de configuração local.

A detecção de sessões de áudio e o controle de mudo são processados no seu PC pelas APIs Windows CoreAudio. Não há rede, contas, telemetria, analytics, relatório de falhas, log remoto nem arquivos de log separados.

## Baixar e Executar

Baixe o ZIP para Windows 10/11, extraia e execute `UnfocusMute.exe`. O app é portátil, então não há instalador e você não precisa de runtime separado, Rust, Visual Studio Build Tools nem MinGW.

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

O pacote é criado em `dist\UnfocusMute-<version>-windows-x64.zip` e inclui o executável, `LICENSE`, `THIRD_PARTY_NOTICES.md`, `README_ko.md` e `README_en.md` na raiz, além dos outros documentos localizados em `docs`.

## Licença

Apache License 2.0. Consulte [LICENSE](../LICENSE).

Consulte [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para os avisos de licença de crates Rust de terceiros.
