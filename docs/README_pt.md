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

---

UnfocusMute é um app pequeno e leve que fica na bandeja do sistema do Windows, silencia automaticamente jogos e apps selecionados quando ficam em segundo plano e restaura o áudio quando voltam ao primeiro plano.

- Criado como app nativo em Rust, ele roda sem precisar de um runtime separado.
- O executável tem cerca de 600 KB.
- Ele não se limita a jogos: você também pode registrar apps comuns, como navegadores, apps de mensagens, launchers e players de mídia.
- O silenciamento e a restauração se aplicam apenas às sessões que o UnfocusMute alterou diretamente; sessões que você já tinha silenciado não são modificadas.

---

<p align="center">
  <img src="../assets/screenshot_pt.png" width="600" alt="Janela principal do UnfocusMute">
</p>

---

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

Depois de extrair o ZIP, mova a pasta `UnfocusMute-windows-x64` para o local de sua preferência e execute `UnfocusMute-v<version>.exe`. Não é necessário instalar o app nem ter um runtime adicional ou ferramentas de desenvolvimento.

> **Observação:** Por questões de custo, o UnfocusMute é distribuído sem assinatura de código do Windows. Por isso, na primeira execução, pode aparecer um aviso do Windows SmartScreen ou de "editor desconhecido". Para verificar por conta própria a origem e a integridade do arquivo, consulte [Transparência e verificação dos arquivos de distribuição](#transparência-e-verificação-dos-arquivos-de-distribuição).

<br>

## Uso

1. Abra o UnfocusMute. Na primeira execução, escolha o idioma e as opções de inicialização e clique em `Começar`.
2. Abra o jogo ou app que deseja registrar e reproduza algum som para que ele apareça na lista.
3. Volte ao UnfocusMute, selecione o app e clique em `Registrar`. O registro pelo nome do `.exe` gerencia todas as sessões de áudio do app e continua funcionando mesmo depois que ele é reiniciado.
4. Use `Alt`+`Tab` para ir a outra janela e depois volte. O app registrado será silenciado enquanto estiver em segundo plano, e o som será restaurado quando ele voltar ao primeiro plano.

Pronto. O monitoramento começa assim que o app é registrado e, com as configurações padrão, o UnfocusMute continua funcionando na bandeja mesmo depois que você fecha a janela.

> **O app não aparece na lista?** Clique em `Todos os processos` ou digite diretamente o nome exato do `.exe`. Use `Ver por PID` se quiser registrar somente um PID específico que esteja em execução. Como o PID muda quando o app é reiniciado, na maioria dos casos é melhor registrá-lo pelo nome do `.exe`.

### Encontrar o nome do executável

Se você não souber o nome do app que deseja registrar, confira o nome exato do `.exe` no Gerenciador de Tarefas.

1. Inicie primeiro o app que você quer registrar.
2. Se ele estiver em tela cheia, mude da tela do jogo para outra janela usando `Alt`+`Tab` ou `Windows`+`Tab`.
3. Pressione `Ctrl`+`Shift`+`Esc` para abrir o Gerenciador de Tarefas.
4. Na lista de processos exibida ao abrir o Gerenciador, clique na coluna `CPU` para ordenar do maior para o menor uso.
5. Encontre perto do topo da lista o app que você acabou de abrir, clique nele com o botão direito e selecione `Propriedades`.
6. Confira o nome do executável terminado em `.exe`, como `game.exe`, e registre-o no UnfocusMute.

### Ações comuns

- Clique com o botão direito em um app registrado para pausar ou retomar seu monitoramento, editar a nota ou remover o registro.
- Clique no status `Monitorando`, na parte superior, para pausar ou retomar todo o monitoramento.
- Em `Configurações`, você pode alterar a inicialização automática e o comportamento ao fechar a janela.
- Para sair completamente, clique com o botão direito no ícone da bandeja e selecione `Sair`.

<br>

## Adicionar notas quando o nome do app não for claro

Clique com o botão direito em um app registrado e selecione `Editar nota` para exibir uma descrição fácil de reconhecer acima do nome do processo. A nota serve apenas para diferenciar o app e não afeta a decisão sobre qual áudio deve ser silenciado.

- `htgame.exe` → `NTE`
- `game.exe (PID 21976)` → `cliente do servidor de teste`

As notas são salvas com as outras configurações em `%APPDATA%\UnfocusMute\config.json`.

<br>

## Comportamentos importantes

**Execução na bandeja e restauração do som:** Se um app registrado for fechado enquanto estiver silenciado, o Windows poderá lembrar esse estado. Enquanto o UnfocusMute continuar em execução na bandeja, ele restaurará o som automaticamente quando você abrir o app de novo e colocá-lo em primeiro plano. Se você também tiver encerrado o UnfocusMute e o app continuar sem som, reative o som manualmente no `Mixer de volume` do Windows.

**Registro por PID:** O Windows pode atribuir PIDs diferentes à sessão de áudio e à janela em primeiro plano. Por isso, mesmo que o registro tenha sido feito por PID, o UnfocusMute considera que o app voltou quando uma janela com o mesmo nome de `.exe` passa ao primeiro plano e restaura o som. Isso não é adequado se você quiser continuar usando uma janela do mesmo `.exe` enquanto mantém um PID específico silenciado. Por outro lado, pode ser útil para silenciar somente um entre vários PIDs do mesmo `.exe` enquanto você trabalha em outro app e manter o som dos demais.

**Compatibilidade com anti-cheat:** O UnfocusMute não injeta código nos jogos, não lê a memória deles, não intercepta entradas nem modifica seus arquivos. Ele usa apenas informações do Windows sobre processos e a janela em primeiro plano, além dos controles de silenciamento do CoreAudio. Foi projetado para evitar conflitos com a maioria dos sistemas anti-cheat, mas não é possível garantir compatibilidade com todos eles.

<br>

## Solução de problemas

Confira primeiro estes pontos:

- **O app não aparece na lista:** Reproduza algum som no app e abra a lista novamente. Se ele ainda não aparecer, clique em `Todos os processos` ou [procure diretamente o nome do executável](#encontrar-o-nome-do-executável).
- **O app não é silenciado:** Verifique se o status na parte superior é `Monitorando` e se o app registrado não está `Pausado`. O recurso também pode não funcionar se um driver, uma configuração de permissão ou um programa de segurança limitar o acesso às sessões de áudio do Windows.
- **O som não é restaurado:** Traga o app de volta ao primeiro plano. Se você já tiver encerrado o UnfocusMute, reative o som manualmente no `Mixer de volume` do Windows.
- **O registro por PID não funciona como esperado:** Confira o [comportamento do registro por PID](#comportamentos-importantes).
- **O status muda para `Atenção`:** Clique em `Detalhes`, ao lado do indicador de status, para ver o erro.

Se o problema continuar, [abra uma issue no GitHub](https://github.com/ilsd7/UnfocusMute/issues/new/choose).

Se você suspeitar de uma vulnerabilidade de segurança, não publique detalhes em uma issue pública. Use o canal de relato privado e consulte [SECURITY.md](../SECURITY.md) para mais informações.

<br>

## Arquivo de configuração

Se precisar verificar ou fazer backup do arquivo de configuração diretamente, clique em `Abrir pasta de configurações` em Configurações. O Explorador de Arquivos abre a pasta `%APPDATA%\UnfocusMute`, onde as configurações são salvas.

Você também pode editar o `config.json` diretamente. Se o formato estiver inválido e não puder ser lido, o UnfocusMute salva uma cópia do arquivo original como `config.invalid-<timestamp>.json` e cria um novo com base nos valores padrão ou nas configurações atuais do app.

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

### Remover completamente o UnfocusMute

1. Se você ativou `Iniciar automaticamente ao fazer login no Windows`, desative essa opção primeiro em `Configurações`.
2. Clique com o botão direito no ícone da bandeja e selecione `Sair`.
3. Exclua as pastas `UnfocusMute-windows-x64` e `%APPDATA%\UnfocusMute`.

Se você já tiver excluído o executável e não puder desativar a inicialização automática, exclua o valor `UnfocusMute` em `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

<br>

## Transparência e verificação dos arquivos de distribuição

O UnfocusMute foi projetado para ser usado com segurança em ambientes comuns, portanto a maioria dos usuários não precisa seguir as etapas de verificação abaixo. Se você não quiser depender apenas da confiança no desenvolvedor ou der especial importância à segurança da cadeia de fornecimento de software, poderá usar este procedimento público para verificar a origem e a integridade dos arquivos baixados.

### Por que uma verificação separada é necessária

Mesmo que você revise o código-fonte do repositório e conclua que ele é seguro, não é possível presumir que os arquivos de uma versão publicada no GitHub foram realmente criados a partir desse código. Se a conta do desenvolvedor for comprometida ou as permissões de publicação forem usadas indevidamente, arquivos sem relação com o código-fonte publicado poderão ser distribuídos.

A comparação de hashes SHA-256 confirma se um arquivo baixado corresponde à soma de verificação publicada, mas não comprova qual código-fonte e qual ambiente de compilação o produziram.

Para tratar esses riscos da cadeia de fornecimento com transparência, o UnfocusMute publica um método que permite verificar diretamente se um arquivo de uma versão publicada no GitHub é um artefato oficial gerado pelo GitHub Actions a partir do commit indicado pela tag dessa versão.

<details>
<summary>Mostrar as etapas de verificação</summary>

Primeiro, instale o [GitHub CLI](https://cli.github.com/). Em seguida, altere o valor de `$version` abaixo para a tag da versão que deseja verificar e execute todo o bloco de comandos no PowerShell.

```powershell
$version = "v1.5.0"
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

Uma verificação bem-sucedida confirma que o ZIP baixado foi gerado pelo workflow indicado do GitHub Actions para a tag de versão especificada e corresponde ao hash registrado em sua atestação.

Isso não prova que o código-fonte em si é seguro, que todo o ambiente do GitHub está íntegro ou que a compilação possa ser reproduzida byte por byte em outro computador.

</details>

<br>

## Compilar a partir do código-fonte

O alvo recomendado para as versões publicadas é `x86_64-pc-windows-msvc`.

Ferramentas necessárias:

- Rust stable
- Visual Studio Build Tools 2022 ou Visual Studio 2022
- Windows 10/11 SDK

Você também precisa do `cargo-about` para atualizar o `THIRD_PARTY_NOTICES.md`.

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

Quando precisar atualizar os avisos de licenças de terceiros:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

O resultado é criado em `dist\UnfocusMute-windows-x64.zip`, com um arquivo de verificação SHA-256 em `dist\UnfocusMute-windows-x64.zip.sha256`. O ZIP inclui o executável com versão (`UnfocusMute-v<version>.exe`), `LICENSE` e `THIRD_PARTY_NOTICES.md`.

</details>

<br>

## Licença

Apache License 2.0. Veja [LICENSE](../LICENSE) para mais detalhes.

Veja [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para os avisos de licenças de crates Rust de terceiros.
