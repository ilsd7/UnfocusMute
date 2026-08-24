<div align="center">
  <img src="../assets/app-icon.png" alt="Ícone do UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>App leve que fica na bandeja do sistema do Windows e silencia automaticamente jogos e apps selecionados quando perdem o foco.</strong></p>

  <p>
    <a href="../README.md">English</a> · <a href="README_ko.md">한국어</a> · <a href="README_ja.md">日本語</a> · <a href="README_zh-CN.md">简体中文</a> · <a href="README_es.md">Español</a> · <a href="README_fr.md">Français</a> · Português · <a href="README_hi.md">हिन्दी</a> · <a href="README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=Build&logo=githubactions&logoColor=white" alt="Build status"></a>
    &nbsp;
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    &nbsp;
    <a href="../LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>Totalmente local &nbsp;·&nbsp; Sem acesso à rede &nbsp;·&nbsp; Sem telemetria &nbsp;·&nbsp; Não requer instalação</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Baixar</a>
    · <a href="#uso">Uso</a>
    · <a href="#segurança-e-privacidade">Privacidade</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

<br>

UnfocusMute é um app pequeno e leve que fica na bandeja do sistema do Windows, silencia automaticamente jogos e apps selecionados quando ficam em segundo plano e restaura o áudio quando voltam ao primeiro plano.

- Criado como app nativo em Rust, ele roda sem precisar de um runtime separado.
- O executável tem cerca de 500 KB.
- Ele não se limita a jogos: você também pode registrar apps comuns, como navegadores, apps de mensagens, launchers e players de mídia.
- O silenciamento e a restauração se aplicam apenas às sessões que o UnfocusMute alterou diretamente; sessões que você já tinha silenciado não são modificadas.

<p align="center">
  <img src="../assets/screenshot_pt.png" width="600" alt="Janela principal do UnfocusMute">
</p>

<br>

## Útil quando

- Você alterna com frequência para outras janelas com Alt+Tab enquanto um jogo ou app continua aberto.
- Você quer manter um app sem som quando ele não tem opção de silenciar em segundo plano.
- Você quer silenciar apenas o áudio em segundo plano de um app específico enquanto faz outras coisas.

## Baixar e executar

No Windows 10/11, baixe o pacote ZIP, extraia e execute o app.

| Pacote mais recente |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Arquivo de verificação SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notas da versão](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Depois de extrair o ZIP, mova a pasta `UnfocusMute-windows-x64` para o local onde você quer manter o app e execute `UnfocusMute-v<version>.exe` dentro dela.

UnfocusMute é um app independente que não requer instalação. Você também não precisa instalar Rust, Visual Studio Build Tools, MinGW nem outras ferramentas de desenvolvimento.

> **Observação:** Como certificados de assinatura de código têm custo, o app atualmente é distribuído sem assinatura de código do Windows. No primeiro uso, podem aparecer avisos do Windows SmartScreen ou de "editor desconhecido". Se quiser verificar a integridade do arquivo por conta própria, consulte [Transparência e verificação dos arquivos de distribuição](#transparência-e-verificação-dos-arquivos-de-distribuição).

<br>

## Antes de usar

UnfocusMute funciona com base nos nomes dos processos, nas informações da janela em primeiro plano e nas sessões CoreAudio fornecidas pelo Windows. Por isso, se um driver, uma configuração de permissão ou uma ferramenta de segurança limitar o acesso às sessões, o controle de silenciamento pode não funcionar corretamente.

Recomendamos manter o UnfocusMute em execução na bandeja do sistema, em vez de encerrá-lo. Se um app registrado for fechado enquanto estiver silenciado, o último estado de silenciamento poderá permanecer. Enquanto o UnfocusMute estiver em execução, ele restaurará o áudio automaticamente quando você abrir o app novamente e colocá-lo em primeiro plano. Se você também encerrar o UnfocusMute, o app poderá ficar sem som; nesse caso, reative o áudio manualmente no `Mixer de volume` do Windows.

**Comportamento ao registrar por PID:** O Windows nem sempre informa o mesmo PID para uma sessão de áudio e para a janela em primeiro plano. Para compensar isso, o UnfocusMute considera que o app voltou ao primeiro plano quando o nome `.exe` do PID registrado corresponde ao nome `.exe` da janela atual em primeiro plano.

Assim, se houver várias instâncias do mesmo `.exe` em execução ao mesmo tempo, talvez não seja possível distinguir perfeitamente uma instância específica. Nesse caso, o som pode ser restaurado quando outra instância estiver em primeiro plano.

**Compatibilidade com anti-cheat:** UnfocusMute não injeta código em jogos, não lê memória do jogo, não intercepta entradas do usuário nem modifica arquivos do jogo. Ele usa apenas informações de processo/janela em primeiro plano do Windows e controles de silenciamento de sessões CoreAudio, então foi projetado para evitar conflitos com a maioria dos sistemas anti-cheat, mas não é possível garantir compatibilidade com todos eles.

<br>

## Uso

1. Abra o UnfocusMute.
2. Escolha o idioma. Recomendamos manter as opções nos valores padrão.
3. Inicie o jogo ou app que você quer registrar.
4. Selecione um app na lista ou digite o nome `.exe` exato e clique em `Registrar`. Se o app ainda não tiver criado uma sessão de áudio, mude para `Todos os processos` para ver todos os processos em execução.
5. Se precisar registrar apenas um PID específico, clique em `Ver por PID` e escolha a entrada individual. Registros por PID valem apenas para a instância em execução; se o app reiniciar com outro PID, registre-o novamente.
6. Clique com o botão direito em um app registrado para editar sua nota ou usar `Pausar` apenas nesse app.
7. Clique no status `Monitorando` na parte superior para pausar ou retomar todo o monitoramento.
8. Abra `Configurações` para alterar as opções, inclusive o comportamento ao fechar a janela.
9. Por padrão, fechar a janela mantém o UnfocusMute rodando na bandeja do sistema. Para sair completamente, clique com o botão direito no ícone da bandeja e escolha `Sair`. Você pode alterar o comportamento do botão de fechar em `Configurações`.

<br>

## Usar notas para apps registrados

Se o nome do processo não for suficiente para identificar de qual app se trata, clique com o botão direito no app registrado e escolha `Editar nota`. A nota aparece acima do nome do processo na lista de apps registrados e não afeta a forma como o app é identificado.

Isso ajuda quando um mesmo launcher de jogo abre vários processos ou quando o nome do executável não deixa claro para que ele serve.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - cliente do servidor de teste`

As notas são salvas localmente junto com o restante das configurações em `%APPDATA%\UnfocusMute\config.json`.

<br>

## Encontrar o nome do executável

Se você não souber qual nome registrar, confira no Gerenciador de Tarefas o nome do executável que termina em `.exe`.

1. Inicie primeiro o app que você quer registrar.
2. Use `Alt`+`Tab` ou `Windows`+`Tab` para voltar à área de trabalho do Windows.
3. Pressione `Ctrl`+`Shift`+`Esc` para abrir o Gerenciador de Tarefas.
4. Ordene a lista de processos por `CPU` e encontre o app que acabou de iniciar.
5. Clique com o botão direito nesse item e abra `Propriedades`.
6. Veja o nome do executável que termina em `.exe`, como `game.exe`, e registre-o no UnfocusMute.

<br>

## Solução de problemas

Se um app não aparecer na lista, ou se o registro por PID não funcionar como esperado, confira primeiro [Antes de usar](#antes-de-usar) e [Encontrar o nome do executável](#encontrar-o-nome-do-executável).

Se o status no topo mudar para `Atenção`, clique em `Detalhes` para ver a mensagem de erro detalhada.

Se o problema continuar, abra uma issue no GitHub.

Se você suspeitar de uma vulnerabilidade de segurança, não publique detalhes em uma issue pública. Use o canal de relato privado e consulte [SECURITY.md](../SECURITY.md) para mais informações.

<br>

## Arquivo de configuração

Se precisar verificar ou fazer backup do arquivo de configuração diretamente, clique em `Abrir pasta de configurações` em Configurações. O Explorador de Arquivos abre a pasta `%APPDATA%\UnfocusMute`, onde as configurações são salvas.

Você pode editar o arquivo de configuração diretamente, mas se o formato estiver inválido e não puder ser lido, ele será salvo como backup em `config.invalid-<timestamp>.json`. Se o problema for encontrado durante a inicialização do app, as configurações serão restauradas para os padrões; se for encontrado enquanto o app estiver em execução, um novo arquivo de configuração será criado com base nas configurações atuais do app.

<br>

## Segurança e privacidade

O UnfocusMute é totalmente local. Funciona normalmente mesmo sem conexão com a internet e não exige permissões de administrador. Também não faz solicitações de rede automáticas, não usa telemetria, não envia relatórios de falha, não registra logs remotamente nem coleta dados.

Como exceção, o repositório GitHub deste projeto só é aberto no navegador padrão quando você clica no botão `Repositório GitHub` em Configurações.

A detecção de sessões de áudio e o controle de silenciamento usam apenas as APIs CoreAudio do Windows. O UnfocusMute não injeta código nos processos de destino, não lê a memória deles nem intercepta entrada do usuário.

### Informações salvas

O UnfocusMute salva apenas as configurações necessárias para funcionar em `%APPDATA%\UnfocusMute\config.json`.

- Nomes de processos registrados
- PIDs registrados diretamente
- Último estado de silenciamento dos apps registrados
- Notas que você escrever
- Idioma e configurações escolhidos
- Posição e tamanho da janela

Essas informações não são enviadas para nenhum lugar.

Se você ativar o início automático ao fazer login no Windows, o caminho do executável atual também será salvo no valor `UnfocusMute` em `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

### Informações que não são salvas

O UnfocusMute não salva histórico de uso, logs de atividade, logs de erro, dados de áudio, títulos de janelas, teclas digitadas nem qualquer informação que não esteja listada acima em "Informações salvas".

### Como remover

Para remover todos os arquivos relacionados ao app, apague a pasta `UnfocusMute-windows-x64` e depois apague `%APPDATA%\UnfocusMute`.

Se você já ativou o início automático, apague também o valor `UnfocusMute` em `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

<br>

## Transparência e verificação dos arquivos de distribuição

O UnfocusMute foi projetado para ser usado com segurança em ambientes comuns, portanto a maioria dos usuários não precisa seguir as etapas de verificação abaixo. Se você não quiser confiar apenas no desenvolvedor ou der especial importância à segurança da cadeia de fornecimento de software, poderá usar este procedimento público para verificar a origem e a integridade dos arquivos baixados.

### Por que uma verificação separada é necessária

Mesmo que você revise o código-fonte do repositório e conclua que ele é seguro, não é possível presumir que os arquivos de uma versão publicada no GitHub foram realmente criados a partir desse código. Se a conta do desenvolvedor for comprometida ou as permissões de publicação forem usadas indevidamente, arquivos sem relação com o código-fonte publicado poderão ser distribuídos.

A comparação de hashes SHA-256 confirma se um arquivo baixado corresponde à soma de verificação publicada, mas não comprova qual código-fonte e qual ambiente de compilação o produziram.

Para tratar esses riscos da cadeia de fornecimento com transparência, o UnfocusMute publica um método que permite verificar diretamente se um arquivo de uma versão publicada no GitHub é um artefato oficial gerado pelo GitHub Actions a partir do commit indicado pela tag dessa versão.

<details>
<summary>Mostrar as etapas de verificação</summary>

Primeiro, instale o [GitHub CLI](https://cli.github.com/), informe a versão que você baixou e execute o comando no PowerShell.

```powershell
$version = Read-Host "Digite a versão publicada (por exemplo, v1.3.5)"
$sourceRef = "refs/tags/$version"
$workflow = "ilsd7/UnfocusMute/.github/workflows/release.yml"

gh attestation verify .\UnfocusMute-windows-x64.zip `
  -R ilsd7/UnfocusMute `
  --source-ref $sourceRef `
  --signer-workflow $workflow
```

Esse comando acessa o serviço de atestação do GitHub e verifica se o SHA-256 do ZIP local corresponde ao valor registrado na procedência da compilação assinada pelo GitHub Actions.

Você também pode comparar o ZIP com o hash SHA-256 publicado na versão.

```powershell
$expectedHash = ((Get-Content .\UnfocusMute-windows-x64.zip.sha256 -TotalCount 1) -split '\s+')[0]
$actualHash = (Get-FileHash .\UnfocusMute-windows-x64.zip -Algorithm SHA256).Hash

if ($actualHash -ne $expectedHash) {
  throw "A verificação SHA-256 falhou."
}

"SHA-256 verificado: $actualHash"
```

Uma verificação bem-sucedida confirma que o ZIP baixado foi gerado para a tag especificada pelo workflow do GitHub Actions indicado e corresponde ao hash registrado em sua atestação.

Isso não prova que o código-fonte em si é seguro, que todo o ambiente do GitHub está íntegro ou que a compilação possa ser reproduzida byte por byte em outro computador.

</details>

<br>

## Compilar a partir do código-fonte

O alvo recomendado para as versões publicadas é `x86_64-pc-windows-msvc`.

Requisitos:

- Rust stable
- Visual Studio Build Tools 2022 ou Visual Studio 2022
- Windows 10/11 SDK

<details>
<summary>Mostrar comandos de compilação e empacotamento</summary>

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

As compilações de release são configuradas para reduzir o tamanho do binário. O perfil de release em `Cargo.toml` remove símbolos, ativa LTO, usa uma única unidade de geração de código (codegen unit), define `panic = "abort"` e otimiza para tamanho.

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

O resultado é criado em `dist\UnfocusMute-windows-x64.zip`, com um arquivo de verificação SHA-256 em `dist\UnfocusMute-windows-x64.zip.sha256`. O ZIP inclui o executável com versão (`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md` e os arquivos README `.txt` de cada idioma na pasta `docs`.

</details>

<br>

## Licença

Apache License 2.0. Veja [LICENSE](../LICENSE) para mais detalhes.

Veja [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para os avisos de licenças de crates Rust de terceiros.
