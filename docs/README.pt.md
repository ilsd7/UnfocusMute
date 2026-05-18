# UnfocusMute

[한국어](../README.md) | [English](../README_en.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Français](README.fr.md) | Português | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

<p align="center">
  <img src="../assets/screenshot.png" alt="Janela do UnfocusMute" width="760">
</p>

UnfocusMute é um app leve e portátil para a bandeja do Windows que silencia apenas o áudio dos jogos ou apps que você escolher quando eles ficam em segundo plano.

Criado como app nativo em Rust, ele roda sem runtime separado. O executável atual para Windows tem cerca de 533 KB, menos de 1 MB.

Registre os apps que você quer gerenciar, e o UnfocusMute silencia apenas as sessões de áudio deles enquanto não estão em foco. Quando um app volta ao primeiro plano, o UnfocusMute restaura somente as sessões que ele mesmo silenciou, sem alterar os silenciamentos feitos manualmente.

Ele é especialmente útil quando você sai de um jogo com Alt+Tab para usar navegador, chat ou uma janela de trabalho. Dá para controlar o áudio em segundo plano sem abrir o mixer de volume do Windows repetidamente.

## Principais Vantagens

- Automatiza o silenciamento em segundo plano por jogo ou app, fazendo o áudio acompanhar as mudanças de foco sem ajustes manuais.
- Restaura apenas o áudio que o UnfocusMute alterou, preservando silenciamentos manuais.
- Funciona como ferramenta leve, portátil e local, sem instalação, conta ou conexão de rede.

## Útil Para

- Alternar de jogos ou apps para outras janelas com Alt+Tab.
- Jogos que não oferecem opção própria de silenciar ao ficar em segundo plano.
- Silenciar o áudio de um jogo em segundo plano sem afetar navegador ou app de chamada.
- Alternar entre gerenciar um `.exe` inteiro e controlar um PID específico quando um app abre vários processos.

## Recursos

- Silencia automaticamente as sessões de áudio dos apps registrados enquanto eles estão em segundo plano e restaura o áudio quando voltam ao primeiro plano.
- Permite adicionar alvos pela lista de apps em execução ou digitando um executável como `game.exe`.
- Oferece alvos por `.exe`, alvos por PID da instância atual e `Mostrar PIDs`.
- Fica na bandeja, permite pausar, abrir a pasta de configuração e evita múltiplas instâncias.
- Permite escolher o idioma no primeiro uso e depois alternar no app entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी e العربية.
- Configuração local em `%APPDATA%\UnfocusMute\config.json`.

## Compatibilidade com Anticheat

O UnfocusMute não injeta código em jogos, não lê memória do jogo, não intercepta entradas e não modifica arquivos do jogo. Como ele usa apenas informações de processos/janela em primeiro plano do Windows e controles de mudo de sessões CoreAudio, espera-se que não cause problemas com a maioria dos sistemas anticheat, mas a compatibilidade com todos não pode ser garantida.

## Por Que Rust

UnfocusMute é uma pequena ferramenta que fica em segundo plano, então inicialização rápida, baixo uso de memória e distribuição simples são importantes. O executável nativo em Rust roda sem runtime separado e conversa diretamente com as APIs Windows CoreAudio sem carregar frameworks residentes desnecessários.

## Segurança e Privacidade

UnfocusMute funciona com uma abordagem local. Ele salva apenas nomes de processos registrados, PIDs opcionais, idioma da interface, posição da janela e preferências de inicialização em um arquivo de configuração local.

A detecção de sessões de áudio e o controle de mudo são processados no seu PC pelas APIs Windows CoreAudio. Não há rede, contas, telemetria, analytics, relatório de falhas nem log remoto, e o app também não cria arquivos de log separados.

## Baixar e Executar

Baixe o ZIP para Windows 10/11, extraia e execute `UnfocusMute.exe`. O app é portátil, então não há instalador e você não precisa de runtime separado, Rust, Visual Studio Build Tools nem MinGW.

## Uso

1. Abra o UnfocusMute.
2. Selecione um idioma no primeiro uso. English vem selecionado por padrão.
3. Inicie o jogo ou app que você quer gerenciar.
4. Atualize a lista de apps em execução, selecione um item e clique em `Adicionar seleção`.
5. Entradas com o mesmo `.exe` são agrupadas por padrão.
6. Use `Mostrar PIDs` somente quando precisar registrar uma instância específica. Um alvo por PID vale apenas para a instância em execução; se o app reiniciar com outro PID, selecione-o novamente.
7. Fechar a janela mantém o UnfocusMute rodando na bandeja. Use `Sair` para encerrar completamente.

## Encontrar o Nome do Processo do Jogo

Se você não souber o que digitar, abra o jogo e pressione `Ctrl`+`Shift`+`Esc` para abrir o Gerenciador de Tarefas. Ordene a lista de processos por `CPU` para encontrar o jogo ativo com mais facilidade. Clique com o botão direito no jogo, abra `Propriedades` e procure o nome do processo terminado em `.exe`, como `game.exe`.

## Padrões

No primeiro uso, você pode escolher se o UnfocusMute inicia automaticamente ao entrar no Windows. Em novas configurações, o início automático fica desativado, enquanto iniciar minimizado na bandeja e desmutar apps ao sair ficam ativados.

Clique em `Idioma` no app para trocar imediatamente entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी e العربية. A escolha é salva automaticamente.

## Build para Desenvolvedores

O alvo recomendado para release é `x86_64-pc-windows-msvc`.

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

As builds de release são configuradas para reduzir o tamanho final. O release profile em `Cargo.toml` remove símbolos, ativa LTO, usa uma única codegen unit, define `panic = "abort"` e otimiza para tamanho.

Criar um ZIP de distribuição:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

O pacote é criado em `dist\UnfocusMute-<version>-windows-x64.zip` e inclui o executável, `LICENSE`, `THIRD_PARTY_NOTICES.md`, `README_ko.md` e `README_en.md` na raiz, além dos outros documentos localizados em `docs`.

## Licença

Apache License 2.0. Consulte [LICENSE](../LICENSE).

Consulte [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para os avisos de licença de crates Rust de terceiros.
