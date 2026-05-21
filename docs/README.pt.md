<div align="center">
  <img src="../assets/app-icon.png" alt="Ícone do UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>App leve e portátil para a bandeja do Windows que silencia apps selecionados em segundo plano.<br>Restaura apenas o áudio que ele mesmo alterou.</strong></p>

  <p>
    <a href="../README.md">한국어</a> · <a href="../README_en.md">English</a> · <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · Português · <a href="README.hi.md">हिन्दी</a> · <a href="README.ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=CI&logo=githubactions&logoColor=white" alt="CI status"></a>
    <img src="https://img.shields.io/badge/version-2.1.1-0D96F6?style=flat-square" alt="Version 2.1.1">
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    <img src="https://img.shields.io/badge/portable-yes-2E7D32?style=flat-square" alt="Portable app">
    <img src="https://img.shields.io/badge/Rust-native-B7410E?style=flat-square&logo=rust&logoColor=white" alt="Rust native app">
    <img src="https://img.shields.io/badge/binary-~491KB-5E35B1?style=flat-square" alt="Executable size about 491KB">
    <img src="https://img.shields.io/badge/telemetry-none-455A64?style=flat-square" alt="No telemetry">
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Baixar</a>
    · <a href="#uso">Uso</a>
    · <a href="#segurança-e-privacidade">Privacidade</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute é um app pequeno e leve para a bandeja do Windows que silencia automaticamente apenas o som do jogo ou app selecionado quando ele vai para segundo plano. Ele não é só para jogos: navegadores, mensageiros, launchers, players de mídia e outros apps também podem ser registrados se aparecerem como sessões de áudio do Windows.

Criado como app nativo em Rust, ele roda sem runtime separado. O executável atual para Windows tem cerca de 491 KB, menos de 1 MB.

<p align="center">
  <img src="../assets/screenshot_pt.png" alt="Janela do UnfocusMute">
</p>

UnfocusMute silencia a sessão de áudio de um app registrado somente enquanto esse app não está em primeiro plano. Quando ele volta ao primeiro plano, o UnfocusMute restaura apenas as sessões que ele próprio silenciou, sem mexer nos estados alterados manualmente por você.

Ele é especialmente útil quando você deixa um jogo aberto e alterna com Alt+Tab entre navegador, chat ou janelas de trabalho. Assim, dá para desligar só o áudio em segundo plano sem abrir o mixer de volume do Windows toda hora.

---

## Baixar e executar

No Windows 10/11, baixe o pacote ZIP e extraia para executar o app.

| Pacote mais recente |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Arquivo SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notas da versão](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Mova a pasta extraída `UnfocusMute-windows-x64` para o local onde você quer manter o app e execute `UnfocusMute-<version>.exe` dentro dela. O app é portátil: não há instalador e você não precisa de runtime separado, Rust, Visual Studio Build Tools nem MinGW.

## Uso

1. Abra o UnfocusMute.
2. Escolha o idioma na tela de primeiro uso. English vem selecionado por padrão.
3. Inicie o jogo ou app que você quer silenciar quando estiver em segundo plano.
4. Abra a lista `Pesquisar processo` ou digite uma busca, escolha um item e clique em `Adicionar seleção`.
5. Se precisar registrar apenas um PID específico, clique em `Mostrar PIDs` e escolha a entrada individual. Alvos por PID valem só para a instância em execução; se o app reiniciar com outro PID, selecione novamente.
6. Clique com o botão direito em um app registrado para editar sua nota ou `Excluir do mudo automático` esse app.
7. Ao fechar a janela, o app continua rodando na bandeja e monitorando. Use `Sair` para encerrar completamente.

## Usar notas em apps registrados

Se o nome do processo não for suficiente para lembrar qual app é, clique com o botão direito no app registrado e escolha `Editar nota`. A nota aparece ao lado do nome do processo na lista de apps registrados e não afeta a regra de detecção.

Isso ajuda quando um mesmo launcher de jogo abre vários processos ou quando o nome do executável não deixa claro para que ele serve.

- `htgame.exe - NTE`
- `chrome.exe (PID 18432) - perfil para tocar música`
- `game.exe (PID 21976) - cliente do servidor de teste`
- `launcher.exe - launcher antes do jogo real`

As notas são salvas localmente junto com o restante das configurações em `%APPDATA%\UnfocusMute\config.json`.

## Encontrar o nome do executável do jogo

Se você não souber qual nome registrar, confira no Gerenciador de Tarefas o executável que termina em `.exe`.

1. Inicie o jogo primeiro.
2. Use `Alt`+`Tab` ou `Windows`+`Tab` para sair da tela do jogo e voltar ao Windows.
3. Pressione `Ctrl`+`Shift`+`Esc` para abrir o Gerenciador de Tarefas.
4. Ordene a lista de processos por `CPU` e encontre o jogo que acabou de iniciar.
5. Clique com o botão direito no jogo e abra `Propriedades`.
6. Veja o nome do executável que termina em `.exe`, como `game.exe`, e adicione-o ao UnfocusMute.

## Antes de usar

UnfocusMute funciona com base nos nomes de processo, nas informações da janela em primeiro plano e nas sessões CoreAudio fornecidas pelo Windows. Se um app ainda não criou uma sessão de áudio, ou se um driver, permissão ou ferramenta de segurança limitar o acesso à sessão, a listagem ou o controle de mudo pode ficar limitado.

Alvos por PID também usam o nome `.exe` da janela em primeiro plano como fallback, porque o Windows nem sempre informa o mesmo PID para a janela ativa e a sessão de áudio. Se houver várias instâncias do mesmo `.exe`, um alvo por PID não consegue separá-las perfeitamente: o som pode ser restaurado quando outra instância do mesmo `.exe` estiver em foco.

UnfocusMute não injeta código em jogos, não lê memória do jogo, não intercepta entrada e não modifica arquivos do jogo. Ele usa apenas informações de processo/janela em primeiro plano do Windows e controles de mudo de sessões CoreAudio, então deve funcionar sem problemas na maioria dos anti-cheats, mas não é possível garantir compatibilidade com todos eles.

## Segurança e privacidade

UnfocusMute funciona com uma abordagem local. Ele salva apenas nomes de processos registrados, PIDs opcionais, idioma da interface, posição da janela e opções de inicialização em um arquivo de configuração local.

A detecção de sessões de áudio e o controle de mudo são processados no seu PC usando as APIs CoreAudio do Windows. Não há solicitações de rede, contas, telemetria, analytics, relatórios de falha nem logs remotos, e o UnfocusMute não cria arquivos de log separados.

---

## Como funciona

UnfocusMute compara os alvos registrados com a janela atualmente em primeiro plano e silencia a sessão de áudio do alvo apenas quando esse app está em segundo plano.

- Se o app alvo estiver em primeiro plano, o estado do áudio não é alterado.
- Se o app alvo estiver em segundo plano, apenas a sessão de áudio desse app é silenciada.
- Quando o app alvo volta ao primeiro plano, o UnfocusMute restaura apenas as sessões que ele silenciou.
- Estados de mudo alterados manualmente no mixer de volume ou por outra ferramenta são preservados.

## Útil quando

- Você costuma usar Alt+Tab para sair de um jogo ou app enquanto ele continua aberto.
- Um jogo ou app não oferece opção própria de silenciar em segundo plano.
- Você quer silenciar só o áudio do jogo em segundo plano enquanto mantém navegador ou chamada audíveis.
- O mesmo `.exe` abre vários processos e você precisa alternar entre gerenciar o app inteiro e controlar um PID específico.

## Recursos

- Silencia automaticamente apps registrados enquanto estão em segundo plano e restaura o áudio quando voltam ao primeiro plano.
- Adiciona alvos pela lista de apps em execução ou digitando um nome como `game.exe`.
- Suporta alvos por `.exe`, alvos por PID da instância atual e `Mostrar PIDs`.
- Notas por app, `Excluir do mudo automático` e `Incluir no mudo automático` por app.
- Execução na bandeja, pausa global, acesso à pasta de configuração e proteção contra instâncias duplicadas.
- Escolha de idioma no primeiro uso e troca imediata dentro do app entre English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية.
- Configurações salvas localmente em `%APPDATA%\UnfocusMute\config.json`.

---

## Compilar a partir do código-fonte

O alvo de release recomendado é `x86_64-pc-windows-msvc`.

Requisitos:

- Rust stable
- Visual Studio Build Tools 2022 ou Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

Builds de release são configurados para reduzir o tamanho do binário. O release profile em `Cargo.toml` remove símbolos, ativa LTO, usa uma única codegen unit, define `panic = "abort"` e otimiza para tamanho.

Executável:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

ZIP de distribuição:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Atualizar avisos de licenças de terceiros:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

O resultado é criado em `dist\UnfocusMute-windows-x64.zip`, com um arquivo de verificação SHA-256 em `dist\UnfocusMute-windows-x64.zip.sha256`. O ZIP inclui o executável com versão (`UnfocusMute-<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md`, os `README_ko.txt` e `README_en.txt` na raiz, além dos outros README `.txt` localizados em `docs`.

---

## Licença

Apache License 2.0. Veja [LICENSE](../LICENSE) para mais detalhes.

Veja [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para os avisos de licenças de crates Rust de terceiros.
