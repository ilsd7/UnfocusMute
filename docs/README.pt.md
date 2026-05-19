<div align="center">
  <img src="../assets/app-icon.png" alt="Ícone do UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>App leve e portátil para a bandeja do Windows que silencia apps em segundo plano e restaura só o áudio que alterou.</strong></p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases">Baixar</a>
    · <a href="#uso">Uso</a>
    · <a href="#segurança-e-privacidade">Privacidade</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>

  <p>
    <code>Windows 10/11</code>
    <code>Portable</code>
    <code>Rust Native</code>
    <code>~516KB</code>
    <code>No telemetry</code>
  </p>

  <p>
    <a href="../README.md">한국어</a> · <a href="../README_en.md">English</a> · <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · Português · <a href="README.hi.md">हिन्दी</a> · <a href="README.ar.md">العربية</a>
  </p>
</div>

UnfocusMute é um app leve e portátil para a bandeja do Windows que silencia apenas o áudio dos jogos ou apps que você escolher quando eles ficam em segundo plano.

Criado como app nativo em Rust, ele roda sem runtime separado. O executável atual para Windows tem cerca de 516 KB, menos de 1 MB.

<p align="center">
  <img src="../assets/screenshot.png" alt="Janela do UnfocusMute" width="760">
</p>

Registre os apps que você quer gerenciar, e o UnfocusMute silencia apenas as sessões de áudio deles enquanto não estão em foco. Quando um app volta ao primeiro plano, o UnfocusMute restaura somente as sessões que ele mesmo silenciou, sem alterar os silenciamentos feitos manualmente.

Ele é especialmente útil quando você sai de um jogo com Alt+Tab para usar navegador, chat ou uma janela de trabalho. Dá para controlar o áudio em segundo plano sem abrir o mixer de volume do Windows repetidamente.

---

## Baixar e executar

Baixe o ZIP para Windows 10/11 em [GitHub Releases](https://github.com/ilsd7/UnfocusMute/releases), extraia e execute `UnfocusMute.exe`. O app é portátil, então não há instalador e você não precisa de runtime separado, Rust, Visual Studio Build Tools nem MinGW.

## Uso

1. Abra o UnfocusMute.
2. Selecione um idioma no primeiro uso. 한국어 vem selecionado por padrão.
3. Inicie o jogo ou app que você quer gerenciar.
4. Atualize a lista de apps em execução, selecione um item e clique em `Adicionar seleção`.
5. Use `Mostrar PIDs` somente quando precisar registrar uma instância específica. Um alvo registrado por PID vale apenas para a instância em execução; se o app reiniciar com outro PID, selecione-o novamente.
6. Fechar a janela mantém o UnfocusMute rodando na bandeja. Use `Sair` para encerrar completamente.

## Encontrar o nome do executável do jogo

Se você não souber o que digitar, confira no Gerenciador de Tarefas o nome do executável que termina em `.exe`.

1. Abra o jogo primeiro.
2. Use `Alt`+`Tab` ou `Windows`+`Tab` para sair da tela do jogo e voltar ao Windows.
3. Pressione `Ctrl`+`Shift`+`Esc` para abrir o Gerenciador de Tarefas.
4. Ordene a lista de processos por `CPU` para encontrar o jogo que acabou de abrir.
5. Clique com o botão direito no jogo e abra `Propriedades`.
6. Encontre o nome do executável terminado em `.exe`, como `game.exe`, e adicione ao UnfocusMute.

---

## Vantagens

- Automatiza o silenciamento em segundo plano por jogo ou app, fazendo o áudio acompanhar as mudanças de foco sem ajustes manuais.
- Restaura apenas o áudio que o UnfocusMute alterou, preservando silenciamentos manuais.
- Funciona como ferramenta leve, portátil e local, sem instalação, conta ou conexão de rede.

## Útil para

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

---

## Antes de usar

O UnfocusMute não injeta código em jogos, não lê memória do jogo, não intercepta entradas e não modifica arquivos do jogo. Como ele usa apenas informações de processos/janela em primeiro plano do Windows e controles de mudo de sessões CoreAudio, espera-se que não cause problemas com a maioria dos sistemas anticheat, mas a compatibilidade com todos não pode ser garantida.

## Segurança e privacidade

UnfocusMute funciona com uma abordagem local. Ele salva apenas nomes de processos registrados, PIDs opcionais, idioma da interface, posição da janela e preferências de inicialização em um arquivo de configuração local.

A detecção de sessões de áudio e o controle de mudo são processados no seu PC pelas APIs Windows CoreAudio. Não há rede, contas, telemetria, analytics, relatório de falhas nem log remoto, e o app também não cria arquivos de log separados.

## Configuração inicial

No primeiro uso, você pode escolher se o UnfocusMute inicia automaticamente ao entrar no Windows. Em novas configurações, o início automático fica desativado, enquanto iniciar minimizado na bandeja e desmutar apps ao sair ficam ativados.

Clique em `Idioma` no app para trocar imediatamente entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी e العربية. A escolha é salva automaticamente.

Use `Abrir pasta de configuração` no app se precisar verificar o arquivo de configuração diretamente ou gerenciar backups.

---

## Compilar a partir do código-fonte

O alvo recomendado para release é `x86_64-pc-windows-msvc`.

Requisitos:

- Rust stable
- Visual Studio Build Tools 2022 ou Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

As builds de release são configuradas para reduzir o tamanho final. O release profile em `Cargo.toml` remove símbolos, ativa LTO, usa uma única codegen unit, define `panic = "abort"` e otimiza para tamanho.

Executável:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

Criar um ZIP de distribuição:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Atualizar avisos de licença de terceiros:

```powershell
cargo about generate --frozen --fail -o THIRD_PARTY_NOTICES.md about.hbs
```

O pacote é criado em `dist\UnfocusMute-<version>-windows-x64.zip` e inclui o executável, `LICENSE`, `THIRD_PARTY_NOTICES.md`, `README_ko.md` e `README_en.md` na raiz, além dos outros documentos localizados em `docs`.

---

## Licença

Apache License 2.0. Consulte [LICENSE](../LICENSE).

Consulte [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para os avisos de licença de crates Rust de terceiros.
